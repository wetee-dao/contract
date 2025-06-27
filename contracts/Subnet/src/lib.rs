#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;
mod events;

#[ink::contract]
mod subnet {
    use ink::{prelude::vec::Vec, storage::Mapping, H256, U256};
    use primitives::{ensure, ok_or_err, ListHelper, VecIndex};

    use crate::{datas::*, errors::Error};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Subnet {
        /// parent contract ==> Dao contract/user
        parent_contract: Address,

        /// workers
        workers: Mapping<NodeID, K8sCluster>,
        /// workers list helper
        workers_helper: ListHelper<NodeID>,
        /// user off worker
        worker_of_user: Mapping<Address, NodeID>,

        /// worker mortgage
        worker_mortgages: Mapping<u128, AssetDeposit>,
        /// mortgage list helper
        worker_mortgage_helper: ListHelper<u128>,
        /// node of worker
        mortgage_of_worker: Mapping<NodeID, VecIndex<u128>>,

        /// secret validators
        secrets: Mapping<NodeID, SecretNode>,
        /// secrets list helper
        secrets_helper: ListHelper<NodeID>,
        /// user off secret
        secret_of_user: Mapping<Address, NodeID>,
        /// secret mortgages
        secret_mortgages: Mapping<NodeID, U256>,

        /// subnet epoch
        epoch: u32,
        /// last epoch block
        last_epoch_block: BlockNumber,
        /// run secrets
        runing_secrets: Vec<(NodeID, u32)>,
        /// pending secrets
        pending_secrets: Vec<(NodeID, u32)>,

        /// worker node code TEE version (TEE Signer,TEE signature)
        worker_code: (Vec<u8>, Vec<u8>),
        /// Secret node code TEE version (TEE Signer,TEE signature)
        secret_code: (Vec<u8>, Vec<u8>),

        /// USD of deposit Price
        deposit_prices: Mapping<u8, U256>,
        /// n/1_000_000 of USD
        deposit_ratio: Mapping<u32, U256>,

        /// boot nooes
        boot_nodes: Vec<NodeID>,
    }

    impl Subnet {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let mut net: Subnet = Default::default();
            net.parent_contract = caller;

            net
        }

        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_parent()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        #[ink(message)]
        pub fn boot_nodes(&self) -> Result<Vec<SecretNode>, Error> {
            let mut nodes = Vec::new();
            for id in self.boot_nodes.iter() {
                let node = self.secrets.get(*id).ok_or(Error::NodeNotExist)?;
                nodes.push(node);
            }

            return Ok(nodes);
        }

        #[ink(message)]
        pub fn set_boot_nodes(&mut self, nodes: Vec<NodeID>) -> Result<(), Error> {
            self.ensure_from_parent()?;

            let mut lnodes = nodes;
            lnodes.sort();
            lnodes.dedup();
            
            self.boot_nodes = lnodes;

            Ok(())
        }

        #[ink(message)]
        pub fn workers(&self) -> Result<Vec<K8sCluster>, Error> {
            let id = self.workers_helper.next_id - 1;
            let mut workers = Vec::new();
            for i in 0..id {
                let worker = self.workers.get(i).ok_or(Error::NodeNotExist)?;
                workers.push(worker);
            }

            return Ok(workers);
        }

        #[ink(message)]
        pub fn worker_register(
            &mut self,
            name: Vec<u8>,
            p2p_id: AccountId,
            ip: Ip,
            port: u32,
            level: u8,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let now = self.env().block_number();

            let worker = K8sCluster {
                name: name,
                p2p_id,
                ip: ip,
                port: port,
                level: level,
                owner: caller,
                start_block: now,
                stop_block: None,
                terminal_block: None,
                status: 0,
            };

            let id = self.workers_helper.next_id;
            self.workers.insert(id, &worker);

            self.worker_of_user.insert(caller, &id);
            // self.workers_helper.list.push(id);
            self.workers_helper.next_id += 1;

            Ok(id)
        }

        #[ink(message)]
        pub fn worker_mortgage(
            &mut self,
            id: NodeID,
            cpu: u32,
            mem: u32,
            cvm_cpu: u32,
            cvm_mem: u32,
            disk: u32,
            gpu: u32,
            deposit: U256,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let worker = self.workers.get(id).ok_or(Error::WorkerNotExist)?;

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(worker.status == 0, Error::WorkerStatusNotReady);

            let deposit = AssetDeposit {
                amount: deposit,
                cpu,
                cvm_cpu,
                mem,
                cvm_mem,
                disk,
                gpu,
                deleted: None,
            };

            let mid = self.worker_mortgage_helper.next_id;
            self.worker_mortgages.insert(mid, &deposit);
            self.worker_mortgage_helper.next_id += 1;
            // self.worker_mortgage_helper.list.push(mid);

            let mut index = self.mortgage_of_worker.get(id).unwrap_or_default();
            index.list.push(mid);
            self.mortgage_of_worker.insert(id, &index);

            Ok(mid)
        }

        #[ink(message)]
        pub fn worker_unmortgage(
            &mut self,
            id: NodeID,
            mortgage_id: u128,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let worker = self.workers.get(id).ok_or(Error::WorkerNotExist)?;

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(worker.status == 0, Error::WorkerStatusNotReady);

            let mut index = self.mortgage_of_worker.get(id).unwrap_or_default();
            let i = index
                .list
                .iter()
                .position(|t| t == &mortgage_id)
                .ok_or(Error::WorkerMortgageNotExist)?;
            index.list.swap_remove(i);

            self.mortgage_of_worker.insert(id, &index);

            let mut mortgage = self
                .worker_mortgages
                .get(id)
                .ok_or(Error::WorkerMortgageNotExist)?;
            mortgage.deleted = Some(self.env().block_number());
            self.worker_mortgages.insert(id, &mortgage);

            ok_or_err!(
                self.env().transfer(worker.owner, mortgage.amount),
                Error::TransferFailed
            );

            Ok(mortgage_id)
        }

        #[ink(message)]
        pub fn worker_stop(&mut self, id: NodeID) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let worker = self.workers.get(id).ok_or(Error::WorkerNotExist)?;

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(worker.status == 0, Error::WorkerStatusNotReady);

            let worker = self
                .mortgage_of_worker
                .get(id)
                .ok_or(Error::WorkerMortgageNotExist)?;
            if worker.list.len() > 0 {
                return Err(Error::WorkerIsUseByUser);
            }

            self.mortgage_of_worker.remove(id);

            Ok(id)
        }

        #[ink(message)]
        pub fn secrets(&self) -> Result<Vec<SecretNode>, Error> {
            let id = self.secrets_helper.next_id - 1;
            let mut secrets = Vec::new();
            for i in 0..id {
                let secret = self.secrets.get(i).ok_or(Error::NodeNotExist)?;
                secrets.push(secret);
            }

            return Ok(secrets);
        }

        #[ink(message)]
        pub fn secret_register(
            &mut self,
            name: Vec<u8>,
            validator_id: AccountId,
            p2p_id: AccountId,
            ip: Ip,
            port: u32,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let now = self.env().block_number();

            let node = SecretNode {
                name: name,
                ip: ip,
                port: port,
                owner: caller,
                validator_id,
                p2p_id,
                start_block: now,
                stop_block: None,
                terminal_block: None,
                status: 0,
            };

            let id = self.secrets_helper.next_id;
            self.secrets.insert(id, &node);

            self.secret_of_user.insert(caller, &id);
            // self.secrets_helper.list.push(id);
            self.secrets_helper.next_id += 1;

            Ok(id)
        }

        #[ink(message)]
        pub fn secret_deposit(&mut self, id: NodeID, deposit: U256) -> Result<(), Error> {
            let caller = self.env().caller();
            let node = self.secrets.get(id).ok_or(Error::NodeNotExist)?;

            ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(node.status == 0, Error::WorkerStatusNotReady);

            let mut amount = self.secret_mortgages.get(id).unwrap_or_default();
            amount += deposit;
            self.secret_mortgages.insert(id, &amount);

            Ok(())
        }

        #[ink(message)]
        pub fn secret_join(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_parent()?;

            let mut nodes = self.pending_secrets.clone();
            let mut is_in = false;
            for (_, node) in nodes.iter_mut().enumerate() {
                if node.0 == id {
                    is_in = true;
                    node.1 = 1;
                    break;
                }
            }
            if !is_in {
                nodes.push((id, 1));
            }

            Ok(())
        }

        #[ink(message)]
        pub fn secret_delete(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_parent()?;

            let mut nodes = self.pending_secrets.clone();
            let mut is_in = false;
            for (_, node) in nodes.iter_mut().enumerate() {
                if node.0 == id {
                    is_in = true;
                    *node = (node.0, 0u32);
                    break;
                }
            }
            if !is_in {
                nodes.push((id, 0));
            }

            Ok(())
        }

        #[ink(message)]
        pub fn epoch(&self) -> (u32, BlockNumber, BlockNumber) {
            let now = self.env().block_number();
            (self.epoch, self.last_epoch_block, now)
        }

        #[ink(message)]
        pub fn next_epoch(&mut self) -> Result<(), Error> {
            let now = self.env().block_number();
            let last_epoch = self.last_epoch_block;

            ensure!(now - last_epoch >= 72000u32.into(), Error::EpochNotExpired);

            self.epoch += 1;
            self.last_epoch_block = now;
            self.calc_new_validators();

            Ok(())
        }

        #[ink(message)]
        pub fn next_epoch_with_gov(&mut self) -> Result<(), Error> {
            self.ensure_from_parent()?;
            let now = self.env().block_number();

            self.epoch += 1;
            self.last_epoch_block = now;
            self.calc_new_validators();

            Ok(())
        }

        /// calaculate new validators
        fn calc_new_validators(&mut self) {
            let mut runings = self.runing_secrets.clone();
            let pendings = self.pending_secrets.clone();
            for (_, runing) in runings.iter_mut().enumerate() {
                for (_, pending) in pendings.iter().enumerate() {
                    if runing.0 == pending.0 {
                        *runing = (runing.0, pending.1);
                        break;
                    }
                }
            }
            
            runings.retain(|x| x.1 != 0);
            self.runing_secrets = runings;
            self.pending_secrets = Vec::new();
        }

        /// Gov call only call from contract
        fn ensure_from_parent(&self) -> Result<(), Error> {
            ensure!(
                self.env().caller() == self.parent_contract,
                Error::MustCallByMainContract
            );

            Ok(())
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;

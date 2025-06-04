#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;
mod events;

#[ink::contract]
mod subnet {
    use ink::{prelude::vec::Vec, storage::Mapping, H256, U256};
    use primitives::{ensure, ok_or_err, some_or_err, ListHelper, VecIndex};

    use crate::{datas::*, errors::Error};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Subnet {
        /// parent contract ==> cloud contract
        parent_contract: Address,
        /// workers
        workers: Mapping<NodeID, K8sCluster>,
        /// workers list helper
        workers_helper: ListHelper<NodeID>,
        /// user off worker
        worker_of_user: Mapping<Address, NodeID>,

        /// secrets
        secrets: Mapping<NodeID, SecretNode>,
        /// secrets list helper
        secrets_helper: ListHelper<NodeID>,
        /// user off secret
        secret_of_user: Mapping<Address, NodeID>,
        /// secret mortgages
        secret_mortgages: Mapping<NodeID, U256>,

        /// run secrets
        runing_secrets: Vec<NodeID>,
        /// pending secrets
        pending_secrets: Vec<NodeID>,

        /// worker node code TEE version (TEE Signer,TEE signature)
        dworker_code: (Vec<u8>, Vec<u8>),
        /// Secret node code TEE version (TEE Signer,TEE signature)
        dsecret_code: (Vec<u8>, Vec<u8>),

        /// worker mortgage
        worker_mortgages: Mapping<u128, AssetDeposit>,
        /// mortgage list helper
        worker_mortgage_helper: ListHelper<u128>,
        /// node of worker
        mortgage_of_worker: Mapping<NodeID, VecIndex<u128>>,

        /// USD of deposit Price
        deposit_prices: Mapping<u8, U256>,
        /// n/1_000_000 of USD
        deposit_ratio: Mapping<u32, U256>,
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
        pub fn worker_register(
            &mut self,
            name: Vec<u8>,
            ip: Vec<Ip>,
            port: u32,
            level: u8,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let now = self.env().block_number();

            let worker = K8sCluster {
                name: name,
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
            let worker = some_or_err!(self.workers.get(id), Error::WorkerNotExist);

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
            let worker = some_or_err!(self.workers.get(id), Error::WorkerNotExist);

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(worker.status == 0, Error::WorkerStatusNotReady);

            let mut index = self.mortgage_of_worker.get(id).unwrap_or_default();
            let i = some_or_err!(
                index.list.iter().position(|t| t == &mortgage_id),
                Error::WorkerMortgageNotExist
            );
            index.list.swap_remove(i);

            self.mortgage_of_worker.insert(id, &index);

            let mut mortgage =
                some_or_err!(self.worker_mortgages.get(id), Error::WorkerMortgageNotExist);
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
            let worker = some_or_err!(self.workers.get(id), Error::WorkerNotExist);

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
        pub fn secret_register(
            &mut self,
            name: Vec<u8>,
            ip: Vec<Ip>,
            port: u32,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();
            let now = self.env().block_number();

            let node = SecretNode {
                name: name,
                ip: ip,
                port: port,
                owner: caller,
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
            let node = some_or_err!(self.secrets.get(id), Error::NodeNotExist);

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

            let pending = self.pending_secrets.clone();
            ensure!(pending.contains(&id), Error::SecretNodeAlreadyExists);

            let nodes = self.runing_secrets.clone();
            ensure!(nodes.contains(&id), Error::SecretNodeAlreadyExists);

            self.pending_secrets.push(id);

            Ok(())
        }

        #[ink(message)]
        pub fn secret_delete(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_parent()?;

            let nodes = self.runing_secrets.clone();
            if nodes.contains(&id) {
                let i = nodes.iter().position(|t| t == &id);
                if i.is_some() {
                    self.runing_secrets.swap_remove(i.unwrap());
                }
            }

            let pendding = self.pending_secrets.clone();
            if pendding.contains(&id) {
                let i = pendding.iter().position(|t| t == &id);
                if i.is_some() {
                    self.pending_secrets.swap_remove(i.unwrap());
                }
            }

            Ok(())
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

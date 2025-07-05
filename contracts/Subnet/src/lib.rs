#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;
mod events;

#[ink::contract]
mod subnet {
    use ink::{prelude::vec::Vec, storage::Mapping, H256, U256};
    use primitives::{ensure, ok_or_err};

    use crate::{datas::*, errors::Error};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Subnet {
        /// parent contract ==> Dao contract/user
        parent_contract: Address,

        /// workers
        workers: Workers,
        /// user off worker
        worker_of_user: Mapping<Address, NodeID>,

        /// worker mortgage
        worker_mortgages: WorkerMortgages,

        /// secret validators
        secrets: Secrets,
        /// user off secret
        secret_of_user: Mapping<Address, NodeID>,
        /// secret mortgages
        secret_mortgages: Mapping<NodeID, U256>,

        /// subnet epoch
        epoch: u32,
        /// epoch solt block number
        epoch_solt: u32,
        /// sidechain Multi-sig account
        side_chain_multi_key: Option<Address>,
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
            net.epoch_solt = 72000;

            net
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
        pub fn workers(&self) -> Vec<(u64, K8sCluster)> {
            let workers = self.workers.desc_list(1, 1000);
            return workers;
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

            let id = self.workers.insert(&worker);
            self.worker_of_user.insert(caller, &id);

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
        ) -> Result<u32, Error> {
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

            let mid = self.worker_mortgages.insert(id, &deposit);

            Ok(mid)
        }

        #[ink(message)]
        pub fn worker_unmortgage(&mut self, id: NodeID, mortgage_id: u32) -> Result<u32, Error> {
            let caller = self.env().caller();
            let worker = self.workers.get(id).ok_or(Error::WorkerNotExist)?;

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(worker.status == 0, Error::WorkerStatusNotReady);

            let list = self.worker_mortgages.list(id, 1, 100000);
            let i = list
                .iter()
                .position(|t| t.0 == mortgage_id)
                .ok_or(Error::WorkerMortgageNotExist)?;

            let now = self.env().block_number();
            let mortgage_id = list[i].0;
            let mut mortgage = list[i].1.clone();
            mortgage.deleted = Some(now);

            self.worker_mortgages.update(id, mortgage_id, &mortgage);

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

            let mut list = self.worker_mortgages.list(id, 0, 100);

            list.retain(|x| x.1.deleted == None);
            if list.len() > 0 {
                return Err(Error::WorkerIsUseByUser);
            }

            Ok(id)
        }

        #[ink(message)]
        pub fn secrets(&self) -> Vec<(u64, SecretNode)> {
            let list = self.secrets.desc_list(1, 10000);

            return list;
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
                terminal_block: None,
                status: 0,
            };

            let id = self.secrets.insert(&node);
            self.secret_of_user.insert(caller, &id);

            if id == 0 {
                let mut node = self.runing_secrets.clone();
                node.push((id, 1));
                self.runing_secrets = node;
            }

            Ok(id)
        }

        #[ink(message)]
        pub fn secret_deposit(&mut self, id: NodeID, deposit: U256) -> Result<(), Error> {
            let caller = self.env().caller();
            let node = self.secrets.get(id).ok_or(Error::NodeNotExist)?;

            ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);

            let mut amount = self.secret_mortgages.get(id).unwrap_or_default();
            amount += deposit;
            self.secret_mortgages.insert(id, &amount);

            Ok(())
        }

        #[ink(message)]
        pub fn secret_delete(&mut self, id: NodeID) -> Result<(), Error> {
            let caller: ink::H160 = self.env().caller();
            let mut node = self.secrets.get(id).ok_or(Error::NodeNotExist)?;

            ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);

            let runing = self.runing_secrets.clone();
            for i in runing.iter() {
                if i.0 == id {
                    return Err(Error::NodeIsRunning);
                }
            }

            let m = self.secret_mortgages.get(id);
            if m.is_some() {
                return Err(Error::NodeIsRunning);
            }

            node.terminal_block = Some(self.env().block_number());
            self.secrets.update(id, &node);

            Ok(())
        }

        #[ink(message)]
        pub fn validators(&self) -> Vec<(u64, SecretNode, u32)> {
            let nodes = self.runing_secrets.clone();
            return nodes
                .iter()
                .map(|(id, power)| {
                    let node = self.secrets.get(*id).unwrap();
                    (id.clone(), node.clone(), *power)
                })
                .collect::<Vec<_>>();
        }

        #[ink(message)]
        pub fn get_pending_secrets(&self) -> Vec<(NodeID, u32)> {
            self.pending_secrets.clone()
        }

        #[ink(message)]
        pub fn validator_join(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_parent()?;
            self.secrets.get(id).ok_or(Error::NodeNotExist)?;

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

            self.pending_secrets = nodes;
            Ok(())
        }

        #[ink(message)]
        pub fn validator_delete(&mut self, id: NodeID) -> Result<(), Error> {
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
        pub fn epoch_info(&self) -> EpochInfo {
            let now = self.env().block_number();

            EpochInfo {
                epoch: self.epoch,
                epoch_solt: self.epoch_solt,
                last_epoch_block: self.last_epoch_block,
                now: now,
                side_chain_pub: self.side_chain_multi_key,
            }
        }

        #[ink(message)]
        pub fn set_epoch_solt(&mut self, epoch_solt: u32) {
            self.epoch_solt = epoch_solt;
        }

        #[ink(message)]
        pub fn set_next_epoch(&mut self, _node_id: u64) -> Result<(), Error> {
            let caller = self.env().caller();
            let now = self.env().block_number();
            let last_epoch = self.last_epoch_block;

            // check sidechain key
            let key = self.side_chain_multi_key.clone();
            if key.is_none() {
                self.side_chain_multi_key = Some(caller);
            } else {
                ensure!(
                    caller == key.unwrap(),
                    Error::InvalidSideChainCaller
                );
            }

            // check epoch block time
            ensure!(
                now - last_epoch >= self.epoch_solt.into(),
                Error::EpochNotExpired
            );

            self.epoch += 1;
            self.last_epoch_block = now;
            self.calc_new_validators();
            Ok(())
        }

        #[ink(message)]
        pub fn next_epoch_validators(&self) -> Result<Vec<(u64, SecretNode, u32)>, Error> {
            let now = self.env().block_number();
            let last_epoch = self.last_epoch_block;

            // check epoch block time
            ensure!(
                now - last_epoch >= (self.epoch_solt - 5).into(),
                Error::EpochNotExpired
            );

            let mut runings = self.runing_secrets.clone();
            let pendings = self.pending_secrets.clone();
            for (_, pending) in pendings.iter().enumerate() {
                let mut is_in = false;
                for (i, runing) in runings.iter().enumerate() {
                    if runing.0 == pending.0 {
                        is_in = true;
                        runings[i] = (runing.0, pending.1);
                        break;
                    }
                }
                if !is_in {
                    runings.push((pending.0, pending.1));
                }
            }
            runings.retain(|x| x.1 != 0);

            return Ok(runings
                .iter()
                .map(|(id, power)| {
                    let node = self.secrets.get(*id).unwrap();
                    (id.clone(), node.clone(), *power)
                })
                .collect::<Vec<_>>());
        }

        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_parent()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        /// calaculate new validators
        fn calc_new_validators(&mut self) {
            let mut runings = self.runing_secrets.clone();
            let pendings = self.pending_secrets.clone();
            for (_, pending) in pendings.iter().enumerate() {
                let mut is_in = false;
                for (i, runing) in runings.iter().enumerate() {
                    if runing.0 == pending.0 {
                        is_in = true;
                        runings[i] = (runing.0, pending.1);
                        break;
                    }
                }
                if !is_in {
                    runings.push((pending.0, pending.1));
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

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;

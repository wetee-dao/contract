#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod datas;
mod errors;
mod events;

#[ink::contract]
mod subnet {
    use ink::{prelude::vec::Vec, storage::Mapping, H256, U256};
    use primitives::{ensure, ok_or_err, AssetInfo};

    use crate::{
        datas::{NodeID, *},
        errors::Error,
    };

    /// Subnet Contract Storage
    /// 子网合约存储结构
    /// 
    /// This contract manages the subnet infrastructure including workers, validators,
    /// regions, epochs, and resource pricing.
    /// 该合约管理子网基础设施，包括工作节点、验证者、区域、周期和资源定价。
    #[ink(storage)]
    #[derive(Default)]
    pub struct Subnet {
        /// Governance contract address (DAO contract or user) / 治理合约地址（DAO 合约或用户）
        gov_contract: Address,

        /// Computing regions (divided into different zones to ensure user experience)
        /// 计算区域（分为不同区域以确保用户体验）
        regions: Regions,
        /// Worker nodes storage / 工作节点存储
        workers: Workers,
        /// Worker status: 0=offline, 1=online / 工作节点状态：0=离线，1=在线
        worker_status: Mapping<NodeID, u8>,
        /// Worker owner mapping / 工作节点所有者映射
        owner_of_worker: Mapping<Address, NodeID>,
        /// Worker mint account mapping / 工作节点挖矿账户映射
        mint_of_worker: Mapping<AccountId, NodeID>,
        /// Workers in each region / 每个区域的工作节点
        regions_workers: RegionWorkers,

        /// Worker mortgage records / 工作节点抵押记录
        worker_mortgages: WorkerMortgages,

        /// Secret validator nodes / 密钥验证者节点
        secrets: Secrets,
        /// Secret node owner mapping / 密钥节点所有者映射
        secret_of_user: Mapping<Address, NodeID>,
        /// Secret node mortgages / 密钥节点抵押
        secret_mortgages: Mapping<NodeID, U256>,

        /// Current subnet epoch / 当前子网周期
        epoch: u32,
        /// Epoch slot block number / 周期槽区块号
        epoch_solt: u32,
        /// Sidechain multi-signature account / 侧链多重签名账户
        side_chain_multi_key: Address,
        /// Last epoch block number / 上一个周期的区块号
        last_epoch_block: BlockNumber,
        /// Currently running secret validators / 当前运行的密钥验证者
        runing_secrets: Vec<(NodeID, u32)>,
        /// Pending secret validators for next epoch / 下一个周期的待处理密钥验证者
        pending_secrets: Vec<(NodeID, u32)>,

        /// Worker node code TEE version (TEE Signer, TEE signature) / 工作节点代码 TEE 版本（TEE 签名者，TEE 签名）
        worker_code: (Vec<u8>, Vec<u8>),
        /// Secret node code TEE version (TEE Signer, TEE signature) / 密钥节点代码 TEE 版本（TEE 签名者，TEE 签名）
        secret_code: (Vec<u8>, Vec<u8>),

        /// Deposit prices in USD by level / 按级别的 USD 抵押价格
        deposit_prices: Mapping<u8, U256>,

        /// Next asset ID / 下一个资产 ID
        next_asset_id: u32,

        /// Asset information by asset ID / 按资产 ID 的资产信息
        asset_infos: Mapping<u32, AssetInfo>,

        /// Asset prices (n/1000 of USD) / 资产价格（USD 的 n/1000）
        asset_prices: Mapping<u32, U256>,

        /// Prices for different worker levels / 不同工作节点级别的价格
        level_prices: Mapping<u8, RunPrice>,

        /// Boot nodes for network initialization / 用于网络初始化的引导节点
        boot_nodes: Vec<NodeID>,
    }

    impl Subnet {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut net: Subnet = Default::default();

            net.gov_contract = Self::env().caller();
            net.epoch_solt = 72000;

            net
        }

        /// boot nodes
        #[ink(message)]
        pub fn boot_nodes(&self) -> Result<Vec<SecretNode>, Error> {
            let mut nodes = Vec::new();
            for id in self.boot_nodes.iter() {
                let node = self.secrets.get(*id).ok_or(Error::NodeNotExist)?;
                nodes.push(node);
            }

            return Ok(nodes);
        }

        /// set boot nodes
        #[ink(message)]
        pub fn set_boot_nodes(&mut self, nodes: Vec<NodeID>) -> Result<(), Error> {
            self.ensure_from_gov()?;

            let mut lnodes = nodes;
            lnodes.sort();
            lnodes.dedup();

            self.boot_nodes = lnodes;

            Ok(())
        }

        /// add or update region
        #[ink(message)]
        pub fn set_region(&mut self, name: Vec<u8>) -> Result<(), Error> {
            self.ensure_from_gov()?;
            self.regions.insert(&name);

            Ok(())
        }

        /// get region name
        #[ink(message)]
        pub fn region(&self, id: u32) -> Option<Vec<u8>> {
            self.regions.get(id)
        }

        /// list regions
        pub fn regions(&self) -> Vec<(u32, Vec<u8>)> {
            self.regions.desc_list(None, 1000)
        }

        /// set price for different levels
        #[ink(message)]
        pub fn set_level_price(&mut self, level: u8, price: RunPrice) -> Result<(), Error> {
            self.ensure_from_gov()?;
            self.level_prices.insert(level, &price);
            Ok(())
        }

        /// get price for different levels (USD
        #[ink(message)]
        pub fn level_price(&self, level: u8) -> Option<RunPrice> {
            self.level_prices.get(level)
        }

        /// set asset info
        #[ink(message)]
        pub fn set_asset(&mut self, info: AssetInfo, price: U256) -> Result<(), Error> {
            let id = self.next_asset_id;
            self.next_asset_id += 1;
            self.asset_infos.insert(id, &info);
            self.asset_prices.insert(id, &price);

            Ok(())
        }

        /// get asset info
        #[ink(message)]
        pub fn asset(&self, id: u32) -> Option<(AssetInfo, U256)> {
            let info = self.asset_infos.get(id);
            let price = self.asset_prices.get(id);
            if info.is_none() || price.is_none() {
                return None;
            }
            Some((info.unwrap(), price.unwrap()))
        }

        /// worker info
        #[ink(message)]
        pub fn worker(&self, id: NodeID) -> Option<K8sCluster> {
            let worker_wrap = self.workers.get(id);
            if worker_wrap.is_none() {
                return None;
            }

            let mut worker = worker_wrap.unwrap();
            let status = self.worker_status.get(id).unwrap_or(0);
            worker.status = status;

            Some(worker)
        }

        /// get all workers
        #[ink(message)]
        pub fn workers(&self, start: Option<u64>, size: u64) -> Vec<(u64, K8sCluster)> {
            let workers = self.workers.desc_list(start, size);
            return workers;
        }

        /// get user worker
        #[ink(message)]
        pub fn user_worker(&self, user: Address) -> Option<(u64, K8sCluster)> {
            let id = self.owner_of_worker.get(user);
            if id.is_none() {
                return None;
            }

            let worker = self.workers.get(id.unwrap());
            if worker.is_none() {
                return None;
            }

            return Some((id.unwrap(), worker.unwrap()));
        }

        /// get mint worker
        #[ink(message)]
        pub fn mint_worker(&self, id: AccountId) -> Option<(u64, K8sCluster)> {
            let id = self.mint_of_worker.get(id);
            if id.is_none() {
                return None;
            }

            let worker = self.workers.get(id.unwrap());
            if worker.is_none() {
                return None;
            }

            return Some((id.unwrap(), worker.unwrap()));
        }

        /// register worker
        #[ink(message)]
        pub fn worker_register(
            &mut self,
            name: Vec<u8>,
            p2p_id: AccountId,
            ip: Ip,
            port: u32,
            level: u8,
            region_id: u32,
        ) -> Result<NodeID, Error> {
            let caller = self.env().caller();

            self.regions.get(region_id).ok_or(Error::RegionNotExist)?;

            let worker_id = self.workers.next_id();
            let now = self.env().block_number();
            let worker = K8sCluster {
                name,
                p2p_id,
                ip,
                port,
                level,
                region_id,
                owner: caller,
                start_block: now,
                stop_block: None,
                terminal_block: None,
                status: 0,
            };

            self.workers.insert(&worker);
            self.owner_of_worker.insert(caller, &worker_id);
            self.mint_of_worker.insert(p2p_id, &worker_id);
            self.regions_workers.insert(region_id, &worker_id);

            Ok(worker_id)
        }

        #[ink(message)]
        pub fn worker_update(
            &mut self,
            id: NodeID,
            name: Vec<u8>,
            ip: Ip,
            port: u32,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut worker = self.workers.get(id).ok_or(Error::WorkerNotExist)?;

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);

            worker.name = name;
            worker.ip = ip;
            worker.port = port;

            self.workers.update(id, &worker);

            Ok(())
        }

        /// Mortgage worker
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
                cpu,
                cvm_cpu,
                mem,
                cvm_mem,
                disk,
                gpu,
                amount: deposit,
                deleted: None,
            };

            let mid = self.worker_mortgages.insert(id, &deposit);

            Ok(mid)
        }

        /// Unmortgage worker
        #[ink(message)]
        pub fn worker_unmortgage(
            &mut self,
            worker_id: NodeID,
            mortgage_id: u32,
        ) -> Result<u32, Error> {
            let caller = self.env().caller();
            let worker = self.workers.get(worker_id).ok_or(Error::WorkerNotExist)?;

            ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
            ensure!(worker.status == 0, Error::WorkerStatusNotReady);

            let mut mortgage = self
                .worker_mortgages
                .get(worker_id, mortgage_id)
                .ok_or(Error::WorkerMortgageNotExist)?;
            let now = self.env().block_number();
            mortgage.deleted = Some(now);
            self.worker_mortgages
                .update(worker_id, mortgage_id, &mortgage);

            ok_or_err!(
                self.env().transfer(worker.owner, mortgage.amount),
                Error::TransferFailed
            );

            Ok(mortgage_id)
        }

        /// Start worker
        #[ink(message)]
        pub fn worker_start(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_side_chain()?;

            // update worker status
            self.worker_status.insert(id, &1);

            Ok(())
        }

        /// Stop worker
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

        /// list secrets
        #[ink(message)]
        pub fn secrets(&self) -> Vec<(u64, SecretNode)> {
            let list = self.secrets.desc_list(None, 10000);

            return list;
        }

        /// register secret
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
        pub fn secret_update(
            &mut self,
            id: NodeID,
            name: Vec<u8>,
            ip: Ip,
            port: u32,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let mut node = self.secrets.get(id).ok_or(Error::NodeNotExist)?;

            ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);

            node.name = name;
            node.ip = ip;
            node.port = port;
            self.secrets.update(id, &node);

            Ok(())
        }

        /// deposit secret
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

        /// Delete secret
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

            let peending = self.pending_secrets.clone();
            for i in peending.iter() {
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

        /// Secret nodes that are currently acting as network validation nodes.
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

        /// Modifications to the nodes that will be acting as validation nodes in the next epoch
        #[ink(message)]
        pub fn get_pending_secrets(&self) -> Vec<(NodeID, u32)> {
            self.pending_secrets.clone()
        }

        /// Add nodes to the validation set through governance
        #[ink(message)]
        pub fn validator_join(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_gov()?;
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

        /// delete secret node form pending and runing validator for next epoch
        #[ink(message)]
        pub fn validator_delete(&mut self, id: NodeID) -> Result<(), Error> {
            self.ensure_from_gov()?;

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

        /// get current epoch info
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

        /// set epoch solt
        #[ink(message)]
        pub fn set_epoch_solt(&mut self, epoch_solt: u32) -> Result<(), Error> {
            self.ensure_from_gov()?;
            self.epoch_solt = epoch_solt;
            Ok(())
        }

        /// goto next epoch
        #[ink(message)]
        pub fn set_next_epoch(&mut self, _node_id: u64) -> Result<(), Error> {
            let caller = self.env().caller();
            let now = self.env().block_number();
            let last_epoch = self.last_epoch_block;

            // check sidechain key
            let key = self.side_chain_multi_key.clone();
            if key == Default::default() {
                self.side_chain_multi_key = caller;
            } else {
                ensure!(caller == key, Error::InvalidSideChainCaller);
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

        /// get next epoch validators
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

        /// update contract
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_gov()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        // get side chain key (H160)
        #[ink(message)]
        pub fn side_chain_key(&self) -> Address {
            self.side_chain_multi_key
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

        /// ensure the caller is from side chain
        fn ensure_from_side_chain(&self) -> Result<(), Error> {
            let caller = self.env().caller();
            ensure!(
                caller == self.side_chain_multi_key,
                Error::InvalidSideChainCaller
            );

            Ok(())
        }

        /// Gov call only call from contract
        fn ensure_from_gov(&self) -> Result<(), Error> {
            ensure!(
                self.env().caller() == self.gov_contract,
                Error::MustCallByMainContract
            );

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

// #[cfg(all(test, feature = "e2e-tests"))]
#[cfg(test)]
mod e2e_tests;

//! Subnet 合约 — PolkaVM/wrevive 迁移版
//! 从 inks/Subnet 迁移，使用 wrevive-api + wrevive-macro。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(all(not(test), not(feature = "api")))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<65536> = pvm_bump_allocator::BumpAllocator::new();

mod datas;
mod errors;

use wrevive_api::*;
use wrevive_macro::{list_2d, mapping, revive_contract, storage};

pub use datas::{AssetDeposit, AssetInfo, EpochInfo, Ip, K8sCluster, NodeID, RunPrice, SecretNode};
pub use errors::Error;
pub use primitives::{ensure, ok_or_err};

#[revive_contract]
pub mod subnet {
    use super::*;
    use crate::datas::NodeID;
    use crate::{Error, ensure};

    const GOV_CONTRACT: Storage<Address> = storage!(b"gov_contract");
    const EPOCH_SOLT: Storage<u32> = storage!(b"epoch_solt");
    const EPOCH: Storage<u32> = storage!(b"epoch");
    const LAST_EPOCH_BLOCK: Storage<BlockNumber> = storage!(b"last_epoch_block");
    const SIDE_CHAIN_MULTI_KEY: Storage<Address> = storage!(b"side_chain_multi_key");
    const NEXT_REGION_ID: Storage<u32> = storage!(b"next_region_id");
    const NEXT_WORKER_ID: Storage<u64> = storage!(b"next_worker_id");
    const NEXT_SECRET_ID: Storage<u64> = storage!(b"next_secret_id");
    const NEXT_ASSET_ID: Storage<u32> = storage!(b"next_asset_id");

    const REGIONS: Mapping<u32, Bytes> = mapping!(b"regions");
    const WORKER_STATUS: Mapping<u64, u8> = mapping!(b"worker_status");
    const OWNER_OF_WORKER: Mapping<Address, u64> = mapping!(b"owner_of_worker");
    const MINT_OF_WORKER: Mapping<AccountId, u64> = mapping!(b"mint_of_worker");
    const SECRET_OF_USER: Mapping<Address, u64> = mapping!(b"secret_of_user");
    const SECRET_MORTGAGES: Mapping<u64, U256> = mapping!(b"secret_mortgages");
    const LEVEL_PRICES: Mapping<u8, RunPrice> = mapping!(b"level_prices");
    const ASSET_INFOS: Mapping<u32, AssetInfo> = mapping!(b"asset_infos");
    const ASSET_PRICES: Mapping<u32, U256> = mapping!(b"asset_prices");

    const WORKERS: Mapping<u64, K8sCluster> = mapping!(b"workers");
    const SECRETS: Mapping<u64, SecretNode> = mapping!(b"secrets");

    const REGION_WORKERS: List2D<u32, u32, u64> = list_2d!(b"region_workers");
    const WORKER_MORTGAGES: List2D<u64, u32, AssetDeposit> = list_2d!(b"worker_mortgages");

    const BOOT_NODES: Mapping<u32, u64> = mapping!(b"boot_nodes");
    const BOOT_NODES_LEN: Storage<u32> = storage!(b"boot_nodes_len");

    const RUNING_SECRETS: Mapping<u32, (u64, u32)> = mapping!(b"runing_secrets");
    const RUNING_SECRETS_LEN: Storage<u32> = storage!(b"runing_secrets_len");
    const PENDING_SECRETS: Mapping<u32, (u64, u32)> = mapping!(b"pending_secrets");
    const PENDING_SECRETS_LEN: Storage<u32> = storage!(b"pending_secrets_len");

    #[revive(constructor)]
    pub fn new() -> Result<(), Error> {
        Ok(())
    }

    #[revive(message, write)]
    pub fn init() -> Result<(), Error> {
        if let Some(_) = GOV_CONTRACT.get() {
            return Ok(());
        }
        let caller = env().caller();
        GOV_CONTRACT.set(&caller);
        EPOCH_SOLT.set(&72000u32);
        EPOCH.set(&0u32);
        LAST_EPOCH_BLOCK.set(&0u32);
        SIDE_CHAIN_MULTI_KEY.set(&Address::zero());
        NEXT_REGION_ID.set(&0u32);
        NEXT_WORKER_ID.set(&0u64);
        NEXT_SECRET_ID.set(&0u64);
        NEXT_ASSET_ID.set(&0u32);
        Ok(())
    }

    #[revive(message)]
    pub fn epoch_info() -> EpochInfo {
        let now = env().now();
        EpochInfo {
            epoch: EPOCH.get().unwrap_or(0),
            epoch_solt: EPOCH_SOLT.get().unwrap_or(72000),
            last_epoch_block: LAST_EPOCH_BLOCK.get().unwrap_or(0),
            now,
            side_chain_pub: SIDE_CHAIN_MULTI_KEY.get().unwrap_or(Address::zero()),
        }
    }

    #[revive(message, write)]
    pub fn set_epoch_solt(epoch_solt: u32) -> Result<(), Error> {
        ensure_from_gov()?;
        EPOCH_SOLT.set(&epoch_solt);
        Ok(())
    }

    #[revive(message)]
    pub fn side_chain_key() -> Address {
        SIDE_CHAIN_MULTI_KEY.get().unwrap_or(Address::zero())
    }

    #[revive(message, write)]
    pub fn set_region(name: Bytes) -> Result<(), Error> {
        ensure_from_gov()?;
        let id = NEXT_REGION_ID.get().unwrap_or(0);
        NEXT_REGION_ID.set(&(id + 1));
        REGIONS.set(&id, &name);
        Ok(())
    }

    #[revive(message)]
    pub fn region(id: u32) -> Option<Bytes> {
        REGIONS.get(&id)
    }

    /// List all regions (id, name); desc order, max 1000.
    #[revive(message)]
    pub fn regions() -> Vec<(u32, Bytes)> {
        let next_id = NEXT_REGION_ID.get().unwrap_or(0);
        let mut out = Vec::new();
        for id in 0..next_id {
            if let Some(name) = REGIONS.get(&id) {
                out.push((id, name));
            }
        }
        out.reverse();
        if out.len() > 1000 {
            out.truncate(1000);
        }
        out
    }

    #[revive(message, write)]
    pub fn set_level_price(level: u8, price: RunPrice) -> Result<(), Error> {
        ensure_from_gov()?;
        LEVEL_PRICES.set(&level, &price);
        Ok(())
    }

    #[revive(message)]
    pub fn level_price(level: u8) -> Option<RunPrice> {
        LEVEL_PRICES.get(&level)
    }

    #[revive(message, write)]
    pub fn set_asset(info: AssetInfo, price: U256) -> Result<(), Error> {
        ensure_from_gov()?;
        let id = NEXT_ASSET_ID.get().unwrap_or(0);
        NEXT_ASSET_ID.set(&(id + 1));
        ASSET_INFOS.set(&id, &info);
        ASSET_PRICES.set(&id, &price);
        Ok(())
    }

    #[revive(message)]
    pub fn asset(id: u32) -> Option<(AssetInfo, U256)> {
        let info = ASSET_INFOS.get(&id)?;
        let price = ASSET_PRICES.get(&id)?;
        Some((info, price))
    }

    #[revive(message)]
    pub fn worker(id: NodeID) -> Option<K8sCluster> {
        let mut worker = WORKERS.get(&id)?;
        worker.status = WORKER_STATUS.get(&id).unwrap_or(0);
        Some(worker)
    }

    /// List workers descending by id; start=None means from latest.
    #[revive(message)]
    pub fn workers(start: Option<u64>, size: u64) -> Vec<(u64, K8sCluster)> {
        let total = NEXT_WORKER_ID.get().unwrap_or(0);
        if total == 0 || size == 0 {
            return Vec::new();
        }
        let mut cur = start.unwrap_or(total - 1);
        if cur >= total {
            cur = total - 1;
        }
        let mut out = Vec::new();
        for _ in 0..size {
            if let Some(mut w) = WORKERS.get(&cur) {
                w.status = WORKER_STATUS.get(&cur).unwrap_or(0);
                out.push((cur, w));
            }
            if cur == 0 {
                break;
            }
            cur -= 1;
        }
        out
    }

    #[revive(message)]
    pub fn user_worker(user: Address) -> Option<(u64, K8sCluster)> {
        let id = OWNER_OF_WORKER.get(&user)?;
        let mut worker = WORKERS.get(&id)?;
        worker.status = WORKER_STATUS.get(&id).unwrap_or(0);
        Some((id, worker))
    }

    #[revive(message)]
    pub fn mint_worker(id: AccountId) -> Option<(u64, K8sCluster)> {
        let worker_id = MINT_OF_WORKER.get(&id)?;
        let mut worker = WORKERS.get(&worker_id)?;
        worker.status = WORKER_STATUS.get(&worker_id).unwrap_or(0);
        Some((worker_id, worker))
    }

    #[revive(message, write)]
    pub fn worker_register(
        name: Bytes,
        p2p_id: AccountId,
        ip: Ip,
        port: u32,
        level: u8,
        region_id: u32,
    ) -> Result<NodeID, Error> {
        REGIONS
            .get(&region_id)
            .ok_or(Error::RegionNotExist)?;
        let caller = env().caller();
        let worker_id = NEXT_WORKER_ID.get().unwrap_or(0);
        let next = worker_id.checked_add(1).ok_or(Error::WorkerNotExist)?;
        NEXT_WORKER_ID.set(&next);
        let now = env().block_number();
        let worker = K8sCluster {
            name,
            owner: caller,
            level,
            region_id,
            start_block: now,
            stop_block: None,
            terminal_block: None,
            p2p_id,
            ip,
            port,
            status: 0,
        };
        WORKERS.set(&worker_id, &worker);
        OWNER_OF_WORKER.set(&caller, &worker_id);
        MINT_OF_WORKER.set(&p2p_id, &worker_id);
        REGION_WORKERS.insert(&region_id, &worker_id);
        Ok(worker_id)
    }

    #[revive(message, write)]
    pub fn worker_update(id: NodeID, name: Bytes, ip: Ip, port: u32) -> Result<(), Error> {
        let caller = env().caller();
        let mut worker = WORKERS.get(&id).ok_or(Error::WorkerNotExist)?;
        ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
        worker.name = name;
        worker.ip = ip;
        worker.port = port;
        WORKERS.set(&id, &worker);
        Ok(())
    }

    #[revive(message, write)]
    pub fn worker_mortgage(
        id: NodeID,
        cpu: u32,
        mem: u32,
        cvm_cpu: u32,
        cvm_mem: u32,
        disk: u32,
        gpu: u32,
        deposit: U256,
    ) -> Result<u32, Error> {
        let caller = env().caller();
        let worker = WORKERS.get(&id).ok_or(Error::WorkerNotExist)?;
        ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
        ensure!(
            WORKER_STATUS.get(&id).unwrap_or(0) == 0,
            Error::WorkerStatusNotReady
        );
        let dep = AssetDeposit {
            amount: deposit,
            cpu,
            cvm_cpu,
            mem,
            cvm_mem,
            disk,
            gpu,
            deleted: None,
        };
        let mid = WORKER_MORTGAGES
            .insert(&id, &dep)
            .ok_or(Error::WorkerMortgageNotExist)?;
        Ok(mid)
    }

    #[revive(message, write)]
    pub fn worker_unmortgage(worker_id: NodeID, mortgage_id: u32) -> Result<u32, Error> {
        let caller = env().caller();
        let worker = WORKERS
            .get(&worker_id)
            .ok_or(Error::WorkerNotExist)?;
        ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
        ensure!(
            WORKER_STATUS.get(&worker_id).unwrap_or(0) == 0,
            Error::WorkerStatusNotReady
        );
        let mut mortgage = WORKER_MORTGAGES
            .get(&worker_id, mortgage_id)
            .ok_or(Error::WorkerMortgageNotExist)?;
        let now = env().block_number();
        mortgage.deleted = Some(now);
        WORKER_MORTGAGES
            .update(&worker_id, mortgage_id, &mortgage)
            .ok_or(Error::WorkerMortgageNotExist)?;
        transfer_native(&worker.owner, mortgage.amount)?;
        Ok(mortgage_id)
    }

    #[revive(message, write)]
    pub fn worker_start(id: NodeID) -> Result<(), Error> {
        ensure_from_side_chain()?;
        WORKER_STATUS.set(&id, &1u8);
        Ok(())
    }

    #[revive(message, write)]
    pub fn worker_stop(id: NodeID) -> Result<NodeID, Error> {
        let caller = env().caller();
        let worker = WORKERS.get(&id).ok_or(Error::WorkerNotExist)?;
        ensure!(worker.owner == caller, Error::WorkerNotOwnedByCaller);
        ensure!(
            WORKER_STATUS.get(&id).unwrap_or(0) == 0,
            Error::WorkerStatusNotReady
        );
        let list = WORKER_MORTGAGES.list_all(&id);
        for (_, dep) in list {
            if dep.deleted.is_none() {
                return Err(Error::WorkerIsUseByUser);
            }
        }
        Ok(id)
    }

    #[revive(message, write)]
    pub fn set_boot_nodes(nodes: Vec<u64>) -> Result<(), Error> {
        ensure_from_gov()?;
        let mut lnodes = nodes;
        lnodes.sort();
        lnodes.dedup();
        let len = lnodes.len() as u32;
        BOOT_NODES_LEN.set(&len);
        for (i, &node_id) in lnodes.iter().enumerate() {
            let k = i as u32;
            BOOT_NODES.set(&k, &node_id);
        }
        Ok(())
    }

    #[revive(message)]
    pub fn boot_nodes() -> Result<Vec<SecretNode>, Error> {
        let len = BOOT_NODES_LEN.get().unwrap_or(0);
        let mut out = Vec::new();
        for i in 0..len {
            if let Some(id) = BOOT_NODES.get(&i) {
                if let Some(node) = SECRETS.get(&id) {
                    out.push(node);
                }
            }
        }
        Ok(out)
    }

    #[revive(message)]
    pub fn get_pending_secrets() -> Vec<(u64, u32)> {
        let len = PENDING_SECRETS_LEN.get().unwrap_or(0);
        (0..len)
            .filter_map(|i| PENDING_SECRETS.get(&i))
            .collect()
    }

    #[revive(message)]
    pub fn secrets() -> Vec<(u64, SecretNode)> {
        let next_id = NEXT_SECRET_ID.get().unwrap_or(0);
        let mut out = Vec::new();
        for id in 0..next_id {
            if let Some(node) = SECRETS.get(&id) {
                out.push((id, node));
            }
        }
        out.reverse();
        if out.len() > 10000 {
            out.truncate(10000);
        }
        out
    }

    #[revive(message, write)]
    pub fn secret_register(
        name: Bytes,
        validator_id: AccountId,
        p2p_id: AccountId,
        ip: Ip,
        port: u32,
    ) -> Result<NodeID, Error> {
        let caller = env().caller();
        let now = env().block_number();
        let node = SecretNode {
            name,
            owner: caller,
            validator_id,
            p2p_id,
            start_block: now,
            terminal_block: None,
            ip,
            port,
            status: 0,
        };
        let id = NEXT_SECRET_ID.get().unwrap_or(0);
        let next = id.checked_add(1).ok_or(Error::NodeNotExist)?;
        NEXT_SECRET_ID.set(&next);
        SECRETS.set(&id, &node);
        SECRET_OF_USER.set(&caller, &id);
        if id == 0 {
            RUNING_SECRETS.set(&0u32, &(0u64, 1u32));
            RUNING_SECRETS_LEN.set(&1u32);
        }
        Ok(id)
    }

    #[revive(message, write)]
    pub fn secret_update(id: NodeID, name: Bytes, ip: Ip, port: u32) -> Result<(), Error> {
        let caller = env().caller();
        let mut node = SECRETS.get(&id).ok_or(Error::NodeNotExist)?;
        ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);
        node.name = name;
        node.ip = ip;
        node.port = port;
        SECRETS.set(&id, &node);
        Ok(())
    }

    #[revive(message, write)]
    pub fn secret_deposit(id: NodeID, deposit: U256) -> Result<(), Error> {
        let caller = env().caller();
        let node = SECRETS.get(&id).ok_or(Error::NodeNotExist)?;
        ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);
        let mut amount = SECRET_MORTGAGES.get(&id).unwrap_or(U256::ZERO);
        amount = amount.wrapping_add(deposit);
        SECRET_MORTGAGES.set(&id, &amount);
        Ok(())
    }

    #[revive(message, write)]
    pub fn secret_delete(id: NodeID) -> Result<(), Error> {
        let caller = env().caller();
        let mut node = SECRETS.get(&id).ok_or(Error::NodeNotExist)?;
        ensure!(node.owner == caller, Error::WorkerNotOwnedByCaller);
        let runing_len = RUNING_SECRETS_LEN.get().unwrap_or(0);
        for i in 0..runing_len {
            if let Some((nid, _)) = RUNING_SECRETS.get(&i) {
                if nid == id {
                    return Err(Error::NodeIsRunning);
                }
            }
        }
        let pending_len = PENDING_SECRETS_LEN.get().unwrap_or(0);
        for i in 0..pending_len {
            if let Some((nid, _)) = PENDING_SECRETS.get(&i) {
                if nid == id {
                    return Err(Error::NodeIsRunning);
                }
            }
        }
        if SECRET_MORTGAGES.get(&id).unwrap_or(U256::ZERO) != U256::ZERO {
            return Err(Error::NodeIsRunning);
        }
        node.terminal_block = Some(env().block_number());
        SECRETS.set(&id, &node);
        Ok(())
    }

    #[revive(message)]
    pub fn validators() -> Vec<(u64, SecretNode, u32)> {
        let len = RUNING_SECRETS_LEN.get().unwrap_or(0);
        let mut out = Vec::new();
        for i in 0..len {
            if let Some((id, power)) = RUNING_SECRETS.get(&i) {
                if let Some(node) = SECRETS.get(&id) {
                    out.push((id, node, power));
                }
            }
        }
        out
    }

    #[revive(message, write)]
    pub fn validator_join(id: NodeID) -> Result<(), Error> {
        ensure_from_gov()?;
        SECRETS.get(&id).ok_or(Error::NodeNotExist)?;
        let raw_len = PENDING_SECRETS_LEN.get().unwrap_or(0);

        let mut nodes: Vec<(u64, u32)> = (0..raw_len)
            .filter_map(|i| PENDING_SECRETS.get(&i))
            .collect();
        let mut found = false;
        for n in nodes.iter_mut() {
            if n.0 == id {
                n.1 = 1;
                found = true;
                break;
            }
        }
        if !found {
            nodes.push((id, 1));
        }
        let new_len = nodes.len() as u32;
        for (idx, (nid, pow)) in nodes.into_iter().enumerate() {
            PENDING_SECRETS.set(&(idx as u32), &(nid, pow));
        }
        
        PENDING_SECRETS_LEN.set(&new_len);
        Ok(())
    }

    #[revive(message, write)]
    pub fn validator_delete(id: NodeID) -> Result<(), Error> {
        ensure_from_gov()?;
        let pending_len = PENDING_SECRETS_LEN.get().unwrap_or(0);
        let mut nodes: Vec<(u64, u32)> = (0..pending_len)
            .filter_map(|i| PENDING_SECRETS.get(&i))
            .collect();
        let mut found = false;
        for n in nodes.iter_mut() {
            if n.0 == id {
                n.1 = 0;
                found = true;
                break;
            }
        }
        if !found {
            nodes.push((id, 0));
        }
        let new_len = nodes.len() as u32;
        for (idx, (nid, pow)) in nodes.into_iter().enumerate() {
            PENDING_SECRETS.set(&(idx as u32), &(nid, pow));
        }
        PENDING_SECRETS_LEN.set(&new_len);
        Ok(())
    }

    #[revive(message, write)]
    pub fn set_next_epoch(_node_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        let now = env().block_number();
        let last_epoch = LAST_EPOCH_BLOCK.get().unwrap_or(0);
        let key = SIDE_CHAIN_MULTI_KEY.get().unwrap_or(Address::zero());
        if key == Address::zero() {
            SIDE_CHAIN_MULTI_KEY.set(&caller);
        } else {
            ensure!(caller == key, Error::InvalidSideChainCaller);
        }
        let epoch_solt = EPOCH_SOLT.get().unwrap_or(72000) as u64;
        ensure!(
            (now as u64).saturating_sub(last_epoch as u64) >= epoch_solt,
            Error::EpochNotExpired
        );
        let epoch = EPOCH.get().unwrap_or(0);
        EPOCH.set(&(epoch + 1));
        LAST_EPOCH_BLOCK.set(&now);
        calc_new_validators();
        Ok(())
    }

    #[revive(message)]
    pub fn next_epoch_validators() -> Result<Vec<(u64, SecretNode, u32)>, Error> {
        let now = env().block_number();
        let last_epoch = LAST_EPOCH_BLOCK.get().unwrap_or(0);
        let epoch_solt = EPOCH_SOLT.get().unwrap_or(72000);
        ensure!(
            (now as u64).saturating_sub(last_epoch as u64) >= (epoch_solt.saturating_sub(5)) as u64,
            Error::EpochNotExpired
        );
        let runing_len = RUNING_SECRETS_LEN.get().unwrap_or(0);
        let pending_len = PENDING_SECRETS_LEN.get().unwrap_or(0);
        let mut runings: Vec<(u64, u32)> = (0..runing_len)
            .filter_map(|i| RUNING_SECRETS.get(&i))
            .collect();
        let pendings: Vec<(u64, u32)> = (0..pending_len)
            .filter_map(|i| PENDING_SECRETS.get(&i))
            .collect();
        for (pid, ppow) in pendings {
            if let Some(r) = runings.iter_mut().find(|x| x.0 == pid) {
                r.1 = ppow;
            } else {
                runings.push((pid, ppow));
            }
        }
        runings.retain(|x| x.1 != 0);
        let out: Vec<(u64, SecretNode, u32)> = runings
            .into_iter()
            .filter_map(|(id, power)| SECRETS.get(&id).map(|node| (id, node, power)))
            .collect();
        Ok(out)
    }

    fn calc_new_validators() {
        let runing_len = RUNING_SECRETS_LEN.get().unwrap_or(0);
        let pending_len = PENDING_SECRETS_LEN.get().unwrap_or(0);
        let mut runings: Vec<(u64, u32)> = (0..runing_len)
            .filter_map(|i| RUNING_SECRETS.get(&i))
            .collect();
        let pendings: Vec<(u64, u32)> = (0..pending_len)
            .filter_map(|i| PENDING_SECRETS.get(&i))
            .collect();
        for (pid, ppow) in pendings {
            if let Some(r) = runings.iter_mut().find(|x| x.0 == pid) {
                r.1 = ppow;
            } else {
                runings.push((pid, ppow));
            }
        }
        runings.retain(|x| x.1 != 0);
        let new_len = runings.len() as u32;
        for (idx, (nid, pow)) in runings.into_iter().enumerate() {
            RUNING_SECRETS.set(&(idx as u32), &(nid, pow));
        }
        RUNING_SECRETS_LEN.set(&new_len);
        PENDING_SECRETS_LEN.set(&0);
    }

    fn ensure_from_gov() -> Result<(), Error> {
        let caller = env().caller();
        let gov = GOV_CONTRACT.get().unwrap_or(Address::zero());
        ensure!(caller == gov, Error::MustCallByMainContract);
        Ok(())
    }

    fn ensure_from_side_chain() -> Result<(), Error> {
        let caller = env().caller();
        let key = SIDE_CHAIN_MULTI_KEY.get().unwrap_or(Address::zero());
        ensure!(caller == key, Error::InvalidSideChainCaller);
        Ok(())
    }

    fn transfer_native(to: &Address, amount: U256) -> Result<(), Error> {
        env()
            .transfer(to, &amount)
            .map_err(|_| Error::TransferFailed)
    }
}

#[cfg(test)]
mod tests;

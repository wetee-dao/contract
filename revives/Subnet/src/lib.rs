//! Subnet 合约 — PolkaVM/wrevive 迁移版
//! 从 inks/Subnet 迁移，使用 wrevive-api + wrevive-macro。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(all(not(test), not(feature = "interface")))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

mod datas;
mod errors;

use wrevive_api::{env, Address, BlockNumber, Bytes, H256, Storage, U256};
use wrevive_macro::{list, mapping, revive_contract, storage};

pub use datas::{AssetInfo, EpochInfo, Ip, K8sCluster, NodeID, RunPrice, SecretNode};
pub use errors::Error;
pub use primitives::{ensure, ok_or_err};

#[revive_contract]
pub mod subnet {
    use super::*;
    use crate::datas::NodeID;
    use crate::{ensure, Error};
    use alloc::vec::Vec;
    use wrevive_api::{List, Mapping};

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
    const MINT_OF_WORKER: Mapping<[u8; 32], u64> = mapping!(b"mint_of_worker");
    const SECRET_OF_USER: Mapping<Address, u64> = mapping!(b"secret_of_user");
    const SECRET_MORTGAGES: Mapping<u64, U256> = mapping!(b"secret_mortgages");
    const LEVEL_PRICES: Mapping<u8, RunPrice> = mapping!(b"level_prices");
    const ASSET_INFOS: Mapping<u32, AssetInfo> = mapping!(b"asset_infos");
    const ASSET_PRICES: Mapping<u32, U256> = mapping!(b"asset_prices");

    const WORKERS: Mapping<u64, K8sCluster> = mapping!(b"workers");
    const SECRETS: Mapping<u64, SecretNode> = mapping!(b"secrets");

    const BOOT_NODES: Mapping<u32, u64> = mapping!(b"boot_nodes");
    const BOOT_NODES_LEN: Storage<u32> = storage!(b"boot_nodes_len");

    const RUNING_SECRETS: List<u32, (u64, u32)> = list!(b"runing_secrets");
    const PENDING_SECRETS: List<u32, (u64, u32)> = list!(b"pending_secrets");

    #[revive(constructor)]
    pub fn new() -> Result<(), Error> {
        let caller = env().caller();
        GOV_CONTRACT.set(env(), &caller);
        EPOCH_SOLT.set(env(), &72000u32);
        EPOCH.set(env(), &0u32);
        LAST_EPOCH_BLOCK.set(env(), &0u32);
        SIDE_CHAIN_MULTI_KEY.set(env(), &Address::zero());
        NEXT_REGION_ID.set(env(), &0u32);
        NEXT_WORKER_ID.set(env(), &0u64);
        NEXT_SECRET_ID.set(env(), &0u64);
        NEXT_ASSET_ID.set(env(), &0u32);
        Ok(())
    }

    #[revive(message)]
    pub fn epoch_info() -> EpochInfo {
        let now = env().now();
        EpochInfo {
            epoch: EPOCH.get(env()).unwrap_or(0),
            epoch_solt: EPOCH_SOLT.get(env()).unwrap_or(72000),
            last_epoch_block: LAST_EPOCH_BLOCK.get(env()).unwrap_or(0),
            now,
            side_chain_pub: SIDE_CHAIN_MULTI_KEY.get(env()).unwrap_or(Address::zero()),
        }
    }

    #[revive(message)]
    pub fn set_epoch_solt(epoch_solt: u32) -> Result<(), Error> {
        ensure_from_gov()?;
        EPOCH_SOLT.set(env(), &epoch_solt);
        Ok(())
    }

    #[revive(message)]
    pub fn side_chain_key() -> Address {
        SIDE_CHAIN_MULTI_KEY.get(env()).unwrap_or(Address::zero())
    }

    #[revive(message)]
    pub fn set_region(name: Bytes) -> Result<(), Error> {
        ensure_from_gov()?;
        let id = NEXT_REGION_ID.get(env()).unwrap_or(0);
        NEXT_REGION_ID.set(env(), &(id + 1));
        REGIONS.set(env(), &id, &name);
        Ok(())
    }

    #[revive(message)]
    pub fn region(id: u32) -> Option<Bytes> {
        REGIONS.get(env(), &id).ok()
    }

    #[revive(message)]
    pub fn set_level_price(level: u8, price: RunPrice) -> Result<(), Error> {
        ensure_from_gov()?;
        LEVEL_PRICES.set(env(), &level, &price);
        Ok(())
    }

    #[revive(message)]
    pub fn level_price(level: u8) -> Option<RunPrice> {
        LEVEL_PRICES.get(env(), &level).ok()
    }

    #[revive(message)]
    pub fn set_asset(info: AssetInfo, price: U256) -> Result<(), Error> {
        ensure_from_gov()?;
        let id = NEXT_ASSET_ID.get(env()).unwrap_or(0);
        NEXT_ASSET_ID.set(env(), &(id + 1));
        ASSET_INFOS.set(env(), &id, &info);
        ASSET_PRICES.set(env(), &id, &price);
        Ok(())
    }

    #[revive(message)]
    pub fn asset(id: u32) -> Option<(AssetInfo, U256)> {
        let info = ASSET_INFOS.get(env(), &id).ok()?;
        let price = ASSET_PRICES.get(env(), &id).ok()?;
        Some((info, price))
    }

    #[revive(message)]
    pub fn worker(id: NodeID) -> Option<K8sCluster> {
        let mut worker = WORKERS.get(env(), &id).ok()?;
        worker.status = WORKER_STATUS.get(env(), &id).unwrap_or(0);
        Some(worker)
    }

    #[revive(message)]
    pub fn set_boot_nodes(nodes: alloc::vec::Vec<u64>) -> Result<(), Error> {
        ensure_from_gov()?;
        let len = nodes.len() as u32;
        BOOT_NODES_LEN.set(env(), &len);
        for (i, &node_id) in nodes.iter().enumerate() {
            let k = i as u32;
            BOOT_NODES.set(env(), &k, &node_id);
        }
        Ok(())
    }

    #[revive(message)]
    pub fn boot_nodes() -> Result<Vec<SecretNode>, Error> {
        let len = BOOT_NODES_LEN.get(env()).unwrap_or(0);
        let mut out = Vec::new();
        for i in 0..len {
            if let Ok(id) = BOOT_NODES.get(env(), &i) {
                if let Ok(node) = SECRETS.get(env(), &id) {
                    out.push(node);
                }
            }
        }
        Ok(out)
    }

    #[revive(message)]
    pub fn get_pending_secrets() -> Vec<(u64, u32)> {
        PENDING_SECRETS
            .list(env(), 0, 10000)
            .into_iter()
            .map(|(_, v)| v)
            .collect()
    }

    #[revive(message)]
    pub fn set_code(_code_hash: H256) -> Result<(), Error> {
        ensure_from_gov()?;
        Err(Error::SetCodeFailed)
    }

    fn ensure_from_gov() -> Result<(), Error> {
        let caller = env().caller();
        let gov = GOV_CONTRACT.get(env()).unwrap_or(Address::zero());
        ensure!(caller == gov, Error::MustCallByMainContract);
        Ok(())
    }

    fn ensure_from_side_chain() -> Result<(), Error> {
        let caller = env().caller();
        let key = SIDE_CHAIN_MULTI_KEY.get(env()).unwrap_or(Address::zero());
        ensure!(caller == key, Error::InvalidSideChainCaller);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrevive_api::with_engine;

    #[test]
    fn deploy_and_epoch_info() {
        with_engine(|e| {
            e.reset();
            e.set_caller([1u8; 20]);
            e.set_call_data(&[]);
        });
        let _ = subnet::new();
        let info = subnet::epoch_info();
        assert_eq!(info.epoch, 0);
        assert_eq!(info.epoch_solt, 72000);
        assert_eq!(subnet::side_chain_key(), Address::zero());
    }
}

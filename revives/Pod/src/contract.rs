//! Pod 合约 — PolkaVM/wrevive 迁移版
//! 从 inks/Pod 迁移，使用 wrevive-api + wrevive-macro。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(all(not(test), not(feature = "api")))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

use parity_scale_codec::Encode as ScaleEncode;
use wrevive_api::{Address, H256, Storage, U256, Env, env};
use wrevive_macro::{revive_contract, storage};

pub use primitives::{AssetInfo, ensure, ok_or_err};

/// 合约错误类型（与 inks Pod 保持一致）
#[derive(Debug, Clone, Copy, PartialEq, Eq, ScaleEncode, parity_scale_codec::Decode)]
pub enum Error {
    SetCodeFailed,
    MustCallByCloudContract,
    InsufficientBalance,
    PayFailed,
    NotOwner,
    NotEnoughAllowance,
    NotEnoughBalance,
    InvalidSideChainCaller,
    /// PolkaVM 暂不支持 ERC20 预编译
    UnsupportedAsset,
}

#[revive_contract]
pub mod pod {
    use super::*;
    use crate::{AssetInfo, Error, ensure};

    /// 云合约地址（父合约）
    const CLOUD_CONTRACT: Storage<Address> = storage!(b"cloud_contract");
    /// 侧链多重签名账户
    const SIDE_CHAIN_MULTI_KEY: Storage<Address> = storage!(b"side_chain_multi_key");
    /// Pod ID
    const POD_ID: Storage<u64> = storage!(b"pod_id");
    /// Pod 所有者地址
    const OWNER: Storage<Address> = storage!(b"owner");

    /// 创建新的 Pod 合约；参数由 wrevive 从 call data 解码并传入
    #[revive(constructor)]
    pub fn new(id: u64, owner: Address, side_chain_multi_key: Address) -> Result<(), Error> {
        let caller = env().caller();
        CLOUD_CONTRACT.set(&caller);
        SIDE_CHAIN_MULTI_KEY.set(&side_chain_multi_key);
        POD_ID.set(&id);
        OWNER.set(&owner);
        Ok(())
    }

    /// 向 Pod 充值原生代币（可支付）；转入金额由运行时处理
    #[revive(message)]
    pub fn charge() -> Result<(), Error> {
        let _ = env().value_transferred();
        Ok(())
    }

    /// 当前合约地址（对应 ink account_id）
    #[revive(message)]
    pub fn account_id() -> Address {
        env().address()
    }

    /// 获取 Pod ID
    #[revive(message)]
    pub fn id() -> u64 {
        POD_ID.get().unwrap_or(0)
    }

    /// 获取云合约地址
    #[revive(message)]
    pub fn cloud() -> Address {
        CLOUD_CONTRACT.get().unwrap_or(Address::zero())
    }

    /// 获取所有者
    #[revive(message)]
    pub fn owner() -> Address {
        OWNER.get().unwrap_or(Address::zero())
    }

    /// 向工作节点支付（仅云合约可调用）
    #[revive(message, write)]
    pub fn pay_for_woker(to: Address, asset: AssetInfo, amount: U256) -> Result<(), Error> {
        ensure_from_cloud()?;
        match asset {
            AssetInfo::Native(_) => {
                ensure!(env().balance() >= amount, Error::NotEnoughBalance);
                transfer_native(&to, amount)?;
                Ok(())
            }
            AssetInfo::ERC20(_, _) => {
                // PolkaVM 当前无 ERC20 预编译，返回不支持
                Err(Error::UnsupportedAsset)
            }
        }
    }

    /// 提现（仅 owner 可调用）
    #[revive(message, write)]
    pub fn withdraw(asset: AssetInfo, to: Address, amount: U256) -> Result<(), Error> {
        let caller = env().caller();
        let owner = OWNER.get().unwrap_or(Address::zero());
        ensure!(caller == owner, Error::NotOwner);
        match asset {
            AssetInfo::Native(_) => {
                ensure!(env().balance() >= amount, Error::InsufficientBalance);
                transfer_native(&to, amount)?;
                Ok(())
            }
            AssetInfo::ERC20(_, _) => Err(Error::UnsupportedAsset),
        }
    }

    /// 更新合约代码（仅云合约可调用）
    /// 注：pallet-revive Env 当前未暴露 set_code_hash，链上需由 runtime 提供 host fn
    #[revive(message, write)]
    pub fn set_code(_code_hash: H256) -> Result<(), Error> {
        ensure_from_cloud()?;
        // TODO: 若 pallet_revive_uapi 提供 set_code_hash，在此调用
        Err(Error::SetCodeFailed)
    }

    fn ensure_from_cloud() -> Result<(), Error> {
        let caller = env().caller();
        let cloud = CLOUD_CONTRACT.get().unwrap_or(Address::zero());
        ensure!(caller == cloud, Error::MustCallByCloudContract);
        Ok(())
    }

    /// 原生代币转账：使用 Env::transfer
    fn transfer_native(to: &Address, amount: U256) -> Result<(), Error> {
        match env().transfer(to, &amount) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::PayFailed),
        }
    }
}

#[cfg(test)]
mod tests;

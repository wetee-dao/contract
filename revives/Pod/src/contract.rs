//! Pod 合约 — PolkaVM/wrevive 迁移版
//! 从 inks/Pod 迁移，使用 wrevive-api + wrevive-macro。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(all(not(test), not(feature = "api")))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<65536> = pvm_bump_allocator::BumpAllocator::new();

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
    /// 目标代码哈希与链上不一致，且 runtime 未提供合约内升级能力（见 set_code）
    CodeUpgradeNotSupported,
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

    /// 创建新的 Pod 合约实例。
    ///
    /// 该函数为合约构造函数，在合约部署时由运行时自动调用。
    /// 调用者（云合约）将被记录为父合约地址，同时初始化 Pod ID、所有者和侧链多重签名账户。
    ///
    /// # 调用权限
    /// 由部署流程触发，通常由云合约调用部署。
    ///
    /// # 参数
    /// - `id`：Pod 的唯一标识 ID。
    /// - `owner`：Pod 的所有者地址，拥有提现等高级权限。
    /// - `side_chain_multi_key`：侧链多重签名账户地址，用于跨链相关操作。
    ///
    /// # 返回值
    /// - `Ok(())`：初始化成功。
    /// - `Err(Error)`：初始化失败（当前实现不会返回错误，保持与未来兼容）。
    #[revive(constructor)]
    pub fn new(id: u64, owner: Address, side_chain_multi_key: Address) -> Result<(), Error> {
        let caller = env().caller();
        CLOUD_CONTRACT.set(&caller);
        SIDE_CHAIN_MULTI_KEY.set(&side_chain_multi_key);
        POD_ID.set(&id);
        OWNER.set(&owner);
        Ok(())
    }

    /// 向 Pod 充值原生代币。
    ///
    /// 该函数允许外部账户向当前 Pod 合约转入原生代币。
    /// 转入金额由运行时通过 `value_transferred` 提供，本函数仅做确认并返回成功。
    ///
    /// # 调用权限
    /// 任何人都可以调用，调用时需附加转账金额。
    ///
    /// # 参数
    /// 无显式参数，转账金额通过交易附加。
    ///
    /// # 返回值
    /// - `Ok(())`：充值成功。
    #[revive(message)]
    pub fn charge() -> Result<(), Error> {
        let _ = env().value_transferred();
        Ok(())
    }

    /// 获取当前合约的链上地址。
    ///
    /// 返回本 Pod 合约在区块链上的唯一地址标识，功能上类似于 ink! 中的 `account_id`。
    ///
    /// # 调用权限
    /// 任何人都可以调用。
    ///
    /// # 参数
    /// 无。
    ///
    /// # 返回值
    /// - `Address`：当前合约的地址。
    #[revive(message)]
    pub fn account_id() -> Address {
        env().address()
    }

    /// 获取当前 Pod 的唯一标识 ID。
    ///
    /// 返回在合约初始化时设置的 Pod ID，用于在整个系统中唯一标识该 Pod。
    ///
    /// # 调用权限
    /// 任何人都可以调用。
    ///
    /// # 参数
    /// 无。
    ///
    /// # 返回值
    /// - `u64`：Pod ID；若未初始化则返回 0。
    #[revive(message)]
    pub fn id() -> u64 {
        POD_ID.get().unwrap_or(0)
    }

    /// 获取关联的云合约地址。
    ///
    /// 云合约是当前 Pod 的父合约，负责 Pod 的创建、支付、升级等管理操作。
    ///
    /// # 调用权限
    /// 任何人都可以调用。
    ///
    /// # 参数
    /// 无。
    ///
    /// # 返回值
    /// - `Address`：云合约地址；若未初始化则返回零地址。
    #[revive(message)]
    pub fn cloud() -> Address {
        CLOUD_CONTRACT.get().unwrap_or(Address::zero())
    }

    /// 获取当前 Pod 的所有者地址。
    ///
    /// 所有者拥有对该 Pod 的高级操作权限，例如提现等。
    ///
    /// # 调用权限
    /// 任何人都可以调用。
    ///
    /// # 参数
    /// 无。
    ///
    /// # 返回值
    /// - `Address`：Pod 所有者地址；若未初始化则返回零地址。
    #[revive(message)]
    pub fn owner() -> Address {
        OWNER.get().unwrap_or(Address::zero())
    }

    /// 向工作节点支付报酬。
    ///
    /// 由云合约发起调用，从当前 Pod 合约余额中向指定工作节点地址转账。
    /// 支持原生代币转账；ERC20 代币因 PolkaVM 当前无对应预编译而暂不支持。
    ///
    /// # 调用权限
    /// **仅云合约可调用**，其他调用者将返回 `Error::MustCallByCloudContract`。
    ///
    /// # 参数
    /// - `to`：收款方（工作节点）地址。
    /// - `asset`：资产类型（原生代币或 ERC20 代币信息）。
    /// - `amount`：转账金额。
    ///
    /// # 返回值
    /// - `Ok(())`：支付成功。
    /// - `Err(Error::MustCallByCloudContract)`：调用者不是云合约。
    /// - `Err(Error::NotEnoughBalance)`：Pod 合约余额不足。
    /// - `Err(Error::PayFailed)`：转账执行失败。
    /// - `Err(Error::UnsupportedAsset)`：尝试使用不支持的 ERC20 资产。
    ///
    /// # 执行流程
    /// 1. 校验调用者是否为云合约。
    /// 2. 根据资产类型分支处理：
    ///    - 原生代币：检查余额充足后执行转账。
    ///    - ERC20：直接返回不支持错误。
    #[revive(message, write)]
    pub fn pay_for_woker(to: Address, asset: AssetInfo, amount: U256) -> Result<(), Error> {
        ensure_from_cloud()?;
        match asset {
            AssetInfo::Native(_) => {
                // 资金从 Pod 合约余额转出至指定地址（矿工/Cloud/区块奖励池）
                // Funds transferred from Pod contract balance to specified address (worker/Cloud/block reward pool)
                ensure!(env().balance() >= amount, Error::NotEnoughBalance);
                transfer_native(&to, amount)?;
                Ok(())
            }
            AssetInfo::ERC20(_, _) => {
                // PolkaVM 当前未提供 ERC20 预编译合约，暂不支持 ERC20 资产转账
                // PolkaVM does not provide ERC20 precompile yet; ERC20 transfers are not supported
                Err(Error::UnsupportedAsset)
            }
        }
    }

    /// 从 Pod 提取资产。
    ///
    /// 允许 Pod 所有者将合约中的资产提取到指定地址。
    /// 支持原生代币提现；ERC20 代币因 PolkaVM 当前无对应预编译而暂不支持。
    ///
    /// # 调用权限
    /// **仅 Pod 所有者可调用**，其他调用者将返回 `Error::NotOwner`。
    ///
    /// # 参数
    /// - `asset`：要提取的资产类型（原生代币或 ERC20 代币信息）。
    /// - `to`：接收提现资金的地址。
    /// - `amount`：提现金额。
    ///
    /// # 返回值
    /// - `Ok(())`：提现成功。
    /// - `Err(Error::NotOwner)`：调用者不是 Pod 所有者。
    /// - `Err(Error::InsufficientBalance)`：合约余额不足。
    /// - `Err(Error::PayFailed)`：转账执行失败。
    /// - `Err(Error::UnsupportedAsset)`：尝试提取不支持的 ERC20 资产。
    ///
    /// # 执行流程
    /// 1. 获取调用者地址并与存储中的所有者地址比对。
    /// 2. 根据资产类型分支处理：
    ///    - 原生代币：检查余额充足后执行转账。
    ///    - ERC20：直接返回不支持错误。
    #[revive(message, write)]
    pub fn withdraw(asset: AssetInfo, to: Address, amount: U256) -> Result<(), Error> {
        let caller = env().caller();
        let owner = OWNER.get().unwrap_or(Address::zero());
        ensure!(caller == owner, Error::NotOwner);
        match asset {
            AssetInfo::Native(_) => {
                // Pod 所有者从 Pod 合约余额中提取资金，用于回收未使用的预付款
                // Pod owner withdraws funds from Pod contract balance to reclaim unused prepayment
                ensure!(env().balance() >= amount, Error::InsufficientBalance);
                // 禁止一次性提取全部余额，至少保留 1 wei，防止在 mint_pod 之前抢跑提光导致支付失败
                // Disallow withdrawing full balance; keep at least 1 wei to prevent race-condition emptying before mint_pod
                ensure!(env().balance() > amount, Error::InsufficientBalance);
                transfer_native(&to, amount)?;
                Ok(())
            }
            AssetInfo::ERC20(_, _) => Err(Error::UnsupportedAsset),
        }
    }

    /// 更新合约代码。
    ///
    /// 由云合约发起调用，用于将当前 Pod 合约的代码升级为指定代码哈希对应的版本。
    /// 若当前代码哈希已与目标一致，则直接返回成功（幂等）。
    /// 由于当前 PolkaVM / pallet-revive 运行时未在合约内暴露 `set_code_hash` 能力，
    /// 实际的代码升级操作尚不支持，因此非幂等情况下会返回 `Error::CodeUpgradeNotSupported`。
    ///
    /// # 调用权限
    /// **仅云合约可调用**，其他调用者将返回 `Error::MustCallByCloudContract`。
    ///
    /// # 参数
    /// - `code_hash`：目标代码哈希，用于标识要升级到的合约代码版本。
    ///
    /// # 返回值
    /// - `Ok(())`：代码哈希一致（无需升级）或升级成功。
    /// - `Err(Error::MustCallByCloudContract)`：调用者不是云合约。
    /// - `Err(Error::CodeUpgradeNotSupported)`：目标代码哈希与当前不一致，且运行时暂不支持合约内代码升级。
    ///
    /// # 执行流程
    /// 1. 校验调用者是否为云合约。
    /// 2. 获取当前合约地址的代码哈希并与传入值比较。
    /// 3. 若相同则直接返回成功；否则返回 `CodeUpgradeNotSupported`。
    #[revive(message, write)]
    pub fn set_code(code_hash: H256) -> Result<(), Error> {
        ensure_from_cloud()?;
        let current = env().code_hash(env().address().as_ref());
        // 代码哈希一致时幂等返回成功，避免重复升级；不一致时返回不支持（runtime 未开放能力）
        // Return success idempotently if code hash matches; otherwise return unsupported (runtime capability not exposed)
        if current == code_hash {
            return Ok(());
        }
        Err(Error::CodeUpgradeNotSupported)
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

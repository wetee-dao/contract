//! Cloud 合约 — PolkaVM/wrevive 迁移版
//! 从 inks/Cloud 迁移，使用 wrevive-api + wrevive-macro。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<65536> = pvm_bump_allocator::BumpAllocator::new();

mod datas;
mod errors;

use wrevive_api::{AccountId, Address, BlockNumber, Env, H256, Storage, U256, Vec, env};
use wrevive_macro::{list_2d, mapping, revive_contract, storage};

pub use datas::*;
pub use errors::Error;
pub use primitives::{ensure, ok_or_err};

#[revive_contract]
pub mod cloud {
    use super::*;
    use crate::{Error, ensure};
    use wrevive_api::{AccountId, Decode, Encode, List2D, Mapping, Vec};

    const GOV_CONTRACT: Storage<Address> = storage!(b"gov_contract");
    const SUBNET_ADDRESS: Storage<Address> = storage!(b"subnet_address");
    const POD_CONTRACT_CODE_HASH: Storage<H256> = storage!(b"pod_contract_code_hash");
    const MINT_INTERVAL: Storage<BlockNumber> = storage!(b"mint_interval");
    const NEXT_POD_ID: Storage<u64> = storage!(b"next_pod_id");

    const PODS: Mapping<u64, Pod> = mapping!(b"pods");
    const POD_VERSION: Mapping<u64, BlockNumber> = mapping!(b"pod_version");
    const POD_STATUS: Mapping<u64, u8> = mapping!(b"pod_status");
    const LAST_MINT_BLOCK: Mapping<u64, BlockNumber> = mapping!(b"last_mint_block");
    const POD_REPORT: Mapping<u64, H256> = mapping!(b"pod_report");
    const POD_KEY: Mapping<u64, AccountId> = mapping!(b"pod_key");
    const WORKER_OF_POD: Mapping<u64, u64> = mapping!(b"worker_of_pod");

    const POD_OF_USER: List2D<Address, u64, u64> = list_2d!(b"pod_of_user");
    const PODS_OF_WORKER: List2D<u64, u64, u64> = list_2d!(b"pods_of_worker");
    const POD_CONTAINERS: List2D<u64, u64, Container> = list_2d!(b"pod_containers");
    const USER_SECRETS: List2D<Address, u64, Secret> = list_2d!(b"user_secrets");
    const USER_DISKS: List2D<Address, u64, Disk> = list_2d!(b"user_disks");
    const PLATFORM_FEE_RATE: Storage<u16> = storage!(b"platform_fee_rate");
    const PLATFORM_FEE_TOTAL: Storage<U256> = storage!(b"platform_fee_total");
    const BLOCK_REWARD_POOL: Storage<U256> = storage!(b"block_reward_pool");
    const NEXT_ARBITRATION_ID: Storage<u64> = storage!(b"next_arbitration_id");
    const ARBITRATIONS: Mapping<u64, Arbitration> = mapping!(b"arbitrations");
    const POD_ARBITRATIONS: List2D<u64, u64, u64> = list_2d!(b"pod_arbitrations");

    /// 合约构造函数，部署时自动执行。
    ///
    /// 当前实现为空，合约状态通过 `init` 方法进行初始化。
    /// 调用权限：无特殊限制，任何人可部署。
    ///
    /// # 返回值
    /// - `Ok(())`：构造成功。
    #[revive(constructor)]
    pub fn new() -> Result<(), Error> {
        Ok(())
    }

    /// 初始化 Cloud 合约的全局配置参数。
    ///
    /// 仅在合约首次部署后调用一次，用于设置治理合约地址、Subnet 合约地址、Pod 合约代码哈希
    /// 以及各类默认参数（mint 间隔、平台费率等）。若已初始化则直接返回成功，不做任何修改。
    ///
    /// 调用权限：首次调用时无限制；已初始化后再次调用不影响状态。
    ///
    /// # 参数
    /// - `subnet_addr`：Subnet 合约地址，用于查询 worker、region、侧链密钥等信息。
    /// - `pod_code_hash`：Pod 合约的代码哈希，创建 Pod 时用于链上实例化。
    ///
    /// # 返回值
    /// - `Ok(())`：初始化成功或已初始化。
    #[revive(message, write)]
    pub fn init(subnet_addr: Address, pod_code_hash: H256) -> Result<(), Error> {
        if let Some(_) = GOV_CONTRACT.get() {
            return Ok(());
        }
        let caller = env().caller();
        GOV_CONTRACT.set(&caller);
        SUBNET_ADDRESS.set(&subnet_addr);
        POD_CONTRACT_CODE_HASH.set(&pod_code_hash);
        MINT_INTERVAL.set(&14400u32);
        NEXT_POD_ID.set(&0u64);
        PLATFORM_FEE_RATE.set(&500u16);
        PLATFORM_FEE_TOTAL.set(&U256::ZERO);
        BLOCK_REWARD_POOL.set(&U256::ZERO);
        NEXT_ARBITRATION_ID.set(&0u64);
        Ok(())
    }

    /// 设置 Pod 合约的代码哈希。
    ///
    /// 用于升级或更换 Pod 合约模板，后续新创建的 Pod 将使用新的代码哈希进行实例化。
    ///
    /// 调用权限：仅治理合约（gov）可调用。
    ///
    /// # 参数
    /// - `pod_contract`：新的 Pod 合约代码哈希。
    ///
    /// # 返回值
    /// - `Ok(())`：设置成功。
    /// - `Err(Error::MustCallByGovContract)`：调用者非治理合约。
    #[revive(message, write)]
    pub fn set_pod_contract(pod_contract: H256) -> Result<(), Error> {
        ensure_from_gov()?;
        POD_CONTRACT_CODE_HASH.set(&pod_contract);
        Ok(())
    }

    /// 获取当前设置的 Pod 合约代码哈希。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `H256`：当前 Pod 合约代码哈希；若未设置则返回 `H256::zero()`。
    #[revive(message)]
    pub fn pod_contract() -> H256 {
        POD_CONTRACT_CODE_HASH.get().unwrap_or(H256::zero())
    }

    /// 更新指定 Pod 的合约代码。
    ///
    /// 获取当前 Pod 合约代码哈希，后续可通过调用 Pod 合约的 `set_code` 实现代码升级。
    /// 当前为占位实现，待完善 TODO。
    ///
    /// 调用权限：任何人可调用（写操作）。
    ///
    /// # 参数
    /// - `pod_id`：目标 Pod 的唯一标识。
    ///
    /// # 返回值
    /// - `Ok(())`：操作成功（当前为占位）。
    /// - `Err(Error::PodNotFound)`：Pod 不存在。
    #[revive(message, write)]
    pub fn update_pod_contract(pod_id: u64) -> Result<(), Error> {
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        let code_hash = POD_CONTRACT_CODE_HASH.get().unwrap_or(H256::zero());

        Ok(())
    }

    /// 设置 Pod 的 mint（计费）间隔区块数。
    ///
    /// mint 间隔决定了 Pod 资源费用结算的周期长度。
    ///
    /// 调用权限：仅治理合约（gov）可调用。
    ///
    /// # 参数
    /// - `t`：新的 mint 间隔，以区块数为单位。
    ///
    /// # 返回值
    /// - `Ok(())`：设置成功。
    /// - `Err(Error::MustCallByGovContract)`：调用者非治理合约。
    #[revive(message, write)]
    pub fn set_mint_interval(t: BlockNumber) -> Result<(), Error> {
        ensure_from_gov()?;
        MINT_INTERVAL.set(&t);
        Ok(())
    }

    /// 获取当前设置的 mint 间隔区块数。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `BlockNumber`：当前 mint 间隔；若未设置则返回默认值 `14400`。
    #[revive(message)]
    pub fn mint_interval() -> BlockNumber {
        MINT_INTERVAL.get().unwrap_or(14400)
    }

    /// 设置平台费率。
    ///
    /// 平台费率用于计算每次 Pod mint 时收取的平台费用比例，基数为 10000（即 100%）。
    /// 例如 500 表示 5% 的平台费率。
    ///
    /// 调用权限：仅治理合约（gov）可调用。
    ///
    /// # 参数
    /// - `rate`：新的平台费率，必须小于等于 10000。
    ///
    /// # 返回值
    /// - `Ok(())`：设置成功。
    /// - `Err(Error::InvalidFeeRate)`：费率超出合法范围。
    /// - `Err(Error::MustCallByGovContract)`：调用者非治理合约。
    #[revive(message, write)]
    pub fn set_platform_fee_rate(rate: u16) -> Result<(), Error> {
        ensure_from_gov()?;
        ensure!(rate <= 10000, Error::InvalidFeeRate);
        PLATFORM_FEE_RATE.set(&rate);
        Ok(())
    }

    /// 获取当前平台费率。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `u16`：当前平台费率；若未设置则返回默认值 `500`（即 5%）。
    #[revive(message)]
    pub fn platform_fee_rate() -> u16 {
        PLATFORM_FEE_RATE.get().unwrap_or(500)
    }

    /// 获取平台累计收取的费用总额。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `U256`：平台费用累计总额；若未设置则返回 `U256::ZERO`。
    #[revive(message)]
    pub fn platform_fee_total() -> U256 {
        PLATFORM_FEE_TOTAL.get().unwrap_or(U256::ZERO)
    }

    /// 获取区块奖励池的当前余额。
    ///
    /// 区块奖励池来源于 Pod mint 时抽取的一部分平台费，由侧链分配给打包节点。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `U256`：区块奖励池余额；若未设置则返回 `U256::ZERO`。
    #[revive(message)]
    pub fn block_reward_pool() -> U256 {
        BLOCK_REWARD_POOL.get().unwrap_or(U256::ZERO)
    }

    /// 从区块奖励池中分配奖励给指定地址。
    ///
    /// 链下验证出块节点后，由群体签名提交，链上执行转账操作。
    /// 从 `BLOCK_REWARD_POOL` 中扣除指定金额并转账给目标地址。
    ///
    /// 调用权限：仅侧链（side-chain）可调用。
    ///
    /// # 参数
    /// - `to`：接收奖励的地址。
    /// - `amount`：奖励金额。
    ///
    /// # 返回值
    /// - `Ok(())`：分配成功。
    /// - `Err(Error::BalanceNotEnough)`：奖励池余额不足。
    /// - `Err(Error::PayFailed)`：转账失败。
    /// - `Err(Error::InvalidSideChainCaller)`：调用者非侧链密钥。
    #[revive(message, write)]
    pub fn distribute_block_reward(to: Address, amount: U256) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let pool = BLOCK_REWARD_POOL.get().unwrap_or(U256::ZERO);
        ensure!(pool >= amount, Error::BalanceNotEnough);
        // 从区块奖励池扣减并转账给目标地址（由侧链链下验证出块节点后调用）
        // Deduct from block reward pool and transfer to target address (called by side-chain after off-chain block producer verification)
        BLOCK_REWARD_POOL.set(&(pool - amount));
        env().transfer(&to, &amount).map_err(|_| Error::PayFailed)?;
        Ok(())
    }

    /// 提取平台费用到指定地址。
    ///
    /// 从 `PLATFORM_FEE_TOTAL` 中扣除指定金额并转账给目标地址，通常用于平台运营支出。
    ///
    /// 调用权限：仅治理合约（gov）可调用。
    ///
    /// # 参数
    /// - `to`：接收平台费的地址。
    /// - `amount`：提取金额。
    ///
    /// # 返回值
    /// - `Ok(())`：提取成功。
    /// - `Err(Error::BalanceNotEnough)`：平台费余额不足。
    /// - `Err(Error::PayFailed)`：转账失败。
    /// - `Err(Error::MustCallByGovContract)`：调用者非治理合约。
    #[revive(message, write)]
    pub fn withdraw_platform_fee(to: Address, amount: U256) -> Result<(), Error> {
        ensure_from_gov()?;
        let total = PLATFORM_FEE_TOTAL.get().unwrap_or(U256::ZERO);
        ensure!(total >= amount, Error::BalanceNotEnough);
        PLATFORM_FEE_TOTAL.set(&(total - amount));
        env().transfer(&to, &amount).map_err(|_| Error::PayFailed)?;
        Ok(())
    }

    /// 获取当前配置的 Subnet 合约地址。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `Address`：Subnet 合约地址；若未设置则返回 `Address::zero()`。
    #[revive(message)]
    pub fn subnet_address() -> Address {
        SUBNET_ADDRESS.get().unwrap_or(Address::zero())
    }

    /// 接收转账的占位函数。
    ///
    /// 用于接收用户向 Cloud 合约转账（充值），目前仅记录转账事件（读取 `value_transferred`），
    /// 不做额外处理。实际业务中转账金额可能用于后续创建 Pod 或其他付费操作。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `Ok(())`：接收成功。
    #[revive(message)]
    pub fn charge() -> Result<(), Error> {
        let _ = env().value_transferred();
        Ok(())
    }

    /// 查询 Cloud 合约的账户余额。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `asset`：资产类型，`AssetInfo::Native` 查询原生代币余额，`AssetInfo::ERC20` 暂不支持。
    ///
    /// # 返回值
    /// - `U256`：对应资产的余额；ERC20 类型固定返回 `U256::ZERO`。
    #[revive(message)]
    pub fn balance(asset: AssetInfo) -> U256 {
        match asset {
            AssetInfo::Native(_) => env().balance(),
            AssetInfo::ERC20(_, _) => U256::ZERO,
        }
    }

    /// 获取已创建的 Pod 总数量。
    ///
    /// Pod ID 从 0 开始递增，此值即为下一个待分配 Pod ID，也代表历史创建的 Pod 总数。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `u64`：Pod 总数量；若未设置则返回 0。
    #[revive(message)]
    pub fn pod_len() -> u64 {
        NEXT_POD_ID.get().unwrap_or(0)
    }

    /// 分页列出所有 Pod（按 pod_id 倒序）。
    ///
    /// 从最新的 Pod 开始，按 ID 递减顺序返回 Pod 基本信息、容器列表及状态。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `start`：起始 Pod ID，`None` 表示从最新（即 `pod_len - 1`）开始。
    /// - `size`：每页返回的最大数量。
    ///
    /// # 返回值
    /// - `Vec<(u64, Pod, Vec<(u64, Container)>, u8)>`：Pod ID、Pod 元信息、容器列表、状态的元组列表。
    #[revive(message)]
    pub fn pods(start: Option<u64>, size: u64) -> Vec<(u64, Pod, Vec<(u64, Container)>, u8)> {
        let total = NEXT_POD_ID.get().unwrap_or(0);
        let mut out = Vec::new();
        if total == 0 || size == 0 {
            return out;
        }

        let mut cur = start.unwrap_or(total - 1);
        if cur >= total {
            cur = total - 1;
        }

        for _ in 0..size {
            if let Some(pod) = PODS.get(&cur) {
                let containers = POD_CONTAINERS.desc_list(&cur, None, 20);
                let status = POD_STATUS.get(&cur).unwrap_or(0);
                out.push((cur, pod, containers, status));
            }
            if cur == 0 {
                break;
            }
            cur -= 1;
        }
        out
    }

    /// 获取当前调用者（msg.sender）拥有的 Pod 数量。
    ///
    /// 调用权限：任何人可调用，返回当前调用地址的 Pod 数量。
    ///
    /// # 返回值
    /// - `u64`：当前用户拥有的 Pod 数量。
    #[revive(message)]
    pub fn user_pod_len() -> u64 {
        let caller = env().caller();
        POD_OF_USER.len(&caller)
    }

    /// 分页获取当前调用者拥有的 Pod 列表（按内部 k2 倒序）。
    ///
    /// 返回当前用户所拥有 Pod 的详细信息，包括 Pod 元信息、容器列表及状态。
    ///
    /// 调用权限：任何人可调用，查询当前调用地址的 Pod。
    ///
    /// # 参数
    /// - `start`：分页起始位置（k2 值），`None` 表示从最新开始。
    /// - `size`：每页数量。
    ///
    /// # 返回值
    /// - `Vec<(u64, Pod, Vec<(u64, Container)>, u8)>`：当前用户的 Pod 列表。
    #[revive(message)]
    pub fn user_pods(start: Option<u64>, size: u64) -> Vec<(u64, Pod, Vec<(u64, Container)>, u8)> {
        let caller = env().caller();
        let ids = POD_OF_USER.desc_list(&caller, start, size as u32);
        let mut out = Vec::new();
        for (_k2, pod_id) in ids.into_iter() {
            if let Some(pod) = PODS.get(&pod_id) {
                let containers = POD_CONTAINERS.desc_list(&pod_id, None, 20);
                let status = POD_STATUS.get(&pod_id).unwrap_or(0);
                out.push((pod_id, pod, containers, status));
            }
        }
        out
    }

    /// 查询指定 worker 上所有 Pod 的版本同步信息（用于 side-chain 同步）。
    ///
    /// 返回每个 Pod 的 ID、版本号（区块高度）、上次 mint 区块及状态。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `worker_id`：worker 的唯一标识。
    ///
    /// # 返回值
    /// - `Vec<(u64, BlockNumber, BlockNumber, u8)>`：
    ///   每个元素为 (pod_id, version, last_mint_block, status)。
    #[revive(message)]
    pub fn worker_pods_version(worker_id: u64) -> Vec<(u64, BlockNumber, BlockNumber, u8)> {
        let ids = PODS_OF_WORKER.desc_list(&worker_id, None, u32::MAX);
        let mut out = Vec::new();
        for (_k2, pod_id) in ids.into_iter() {
            let version = POD_VERSION.get(&pod_id).unwrap_or(0);
            let last_mint = LAST_MINT_BLOCK.get(&pod_id).unwrap_or(0);
            let status = POD_STATUS.get(&pod_id).unwrap_or(0);
            out.push((pod_id, version, last_mint, status));
        }
        out
    }

    /// 分页查询指定 worker 上的 Pod 列表（按内部 k2 倒序）。
    ///
    /// 返回 worker 上托管的 Pod 详细信息，包括 Pod 元信息、容器列表及状态。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `worker_id`：worker 的唯一标识。
    /// - `start`：分页起始位置（k2 值），`None` 表示从最新开始。
    /// - `size`：每页数量。
    ///
    /// # 返回值
    /// - `Vec<(u64, Pod, Vec<(u64, Container)>, u8)>`：worker 上的 Pod 列表。
    #[revive(message)]
    pub fn worker_pods(
        worker_id: u64,
        start: Option<u64>,
        size: u64,
    ) -> Vec<(u64, Pod, Vec<(u64, Container)>, u8)> {
        let ids = PODS_OF_WORKER.desc_list(&worker_id, start, size as u32);
        let mut out = Vec::new();
        for (_k2, pod_id) in ids.into_iter() {
            if let Some(pod) = PODS.get(&pod_id) {
                let containers = POD_CONTAINERS.desc_list(&pod_id, None, 20);
                let status = POD_STATUS.get(&pod_id).unwrap_or(0);
                out.push((pod_id, pod, containers, status));
            }
        }
        out
    }

    /// 获取指定 worker 上托管的 Pod 数量。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `worker_id`：worker 的唯一标识。
    ///
    /// # 返回值
    /// - `u64`：该 worker 上的 Pod 数量。
    #[revive(message)]
    pub fn worker_pod_len(worker_id: u64) -> u64 {
        PODS_OF_WORKER.len(&worker_id)
    }

    /// 分页查询指定用户的 Secret 列表（按 k2 倒序）。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `user`：目标用户地址。
    /// - `start`：分页起始位置（k2 值），`None` 表示从最新开始。
    /// - `size`：每页数量。
    ///
    /// # 返回值
    /// - `Vec<(u64, Secret)>`：Secret 的 ID 与内容列表。
    #[revive(message)]
    pub fn user_secrets(user: Address, start: Option<u64>, size: u64) -> Vec<(u64, Secret)> {
        USER_SECRETS.desc_list(&user, start, size as u32)
    }

    /// 读取指定用户的某个 Secret。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `user`：Secret 所属用户地址。
    /// - `index`：Secret 在 List2D 中的索引（k2）。
    ///
    /// # 返回值
    /// - `Some(Secret)`：存在时返回 Secret 内容。
    /// - `None`：不存在。
    #[revive(message)]
    pub fn secret(user: Address, index: u64) -> Option<Secret> {
        USER_SECRETS.get(&user, index)
    }

    /// 创建一个新的 Secret。
    ///
    /// 将用户提供的密钥和哈希值保存到链上，初始状态为未 mint（`minted: false`）。
    /// Secret 创建后由侧链进行后续 mint 标记。
    ///
    /// 调用权限：仅 Secret 的 owner（调用者）可创建。
    ///
    /// # 参数
    /// - `key`：密钥内容（字节数组）。
    /// - `hash`：密钥的哈希值，用于校验。
    ///
    /// # 返回值
    /// - `Ok(u64)`：创建成功，返回分配到的 Secret ID（k2）。
    /// - `Err(Error::NotFound)`：插入失败。
    #[revive(message, write)]
    pub fn create_secret(key: Vec<u8>, hash: H256) -> Result<u64, Error> {
        let caller = env().caller();
        let s = Secret {
            k: key,
            hash,
            minted: false,
        };
        USER_SECRETS.insert(&caller, &s).ok_or(Error::NotFound)
    }

    /// 侧链标记指定 Secret 已 mint。
    ///
    /// 侧链完成 Secret 的链下处理后，调用此函数将 Secret 状态置为已 mint。
    ///
    /// 调用权限：仅侧链（side-chain）可调用。
    ///
    /// # 参数
    /// - `user`：Secret 所属用户地址。
    /// - `index`：Secret 的索引（k2）。
    ///
    /// # 返回值
    /// - `Ok(())`：标记成功。
    /// - `Err(Error::NotFound)`：Secret 不存在。
    /// - `Err(Error::InvalidSideChainCaller)`：调用者非侧链密钥。
    #[revive(message, write)]
    pub fn mint_secret(user: Address, index: u64) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let mut s = USER_SECRETS.get(&user, index).ok_or(Error::NotFound)?;
        s.minted = true;
        USER_SECRETS
            .update(&user, index, &s)
            .ok_or(Error::NotFound)?;
        Ok(())
    }

    /// 删除当前调用者的指定 Secret。
    ///
    /// 从链上移除调用者拥有的 Secret 记录。
    ///
    /// 调用权限：仅 Secret 的 owner（调用者）可删除。
    ///
    /// # 参数
    /// - `index`：Secret 的索引（k2）。
    ///
    /// # 返回值
    /// - `Ok(())`：删除成功。
    /// - `Err(Error::DelFailed)`：删除失败（可能索引不存在）。
    #[revive(message, write)]
    pub fn del_secret(index: u64) -> Result<(), Error> {
        let caller = env().caller();
        USER_SECRETS.clear(&caller, index).ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// 创建一个新的磁盘（SecretSSD 类型）。
    ///
    /// 用户可创建加密磁盘供 Pod 容器挂载使用，磁盘大小以 GB 为单位。
    ///
    /// 调用权限：仅磁盘 owner（调用者）可创建。
    ///
    /// # 参数
    /// - `key`：磁盘的加密密钥（字节数组）。
    /// - `size`：磁盘大小（GB）。
    ///
    /// # 返回值
    /// - `Ok(u64)`：创建成功，返回分配到的磁盘 ID（k2）。
    /// - `Err(Error::NotFound)`：插入失败。
    #[revive(message, write)]
    pub fn create_disk(key: Vec<u8>, size: u32) -> Result<u64, Error> {
        let caller = env().caller();
        let d = Disk::SecretSSD(key, Vec::new(), size);
        USER_DISKS.insert(&caller, &d).ok_or(Error::NotFound)
    }

    /// 侧链更新磁盘的加密密钥。
    ///
    /// 侧链在处理磁盘挂载或迁移时，可能需要更新磁盘的加密哈希。
    ///
    /// 调用权限：仅侧链（side-chain）可调用。
    ///
    /// # 参数
    /// - `user`：磁盘所属用户地址。
    /// - `id`：磁盘 ID（k2）。
    /// - `hash`：新的加密哈希值。
    ///
    /// # 返回值
    /// - `Ok(())`：更新成功。
    /// - `Err(Error::NotFound)`：磁盘不存在。
    /// - `Err(Error::InvalidSideChainCaller)`：调用者非侧链密钥。
    #[revive(message, write)]
    pub fn update_disk_key(user: Address, id: u64, hash: H256) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let disk = USER_DISKS.get(&user, id).ok_or(Error::NotFound)?;
        match disk {
            Disk::SecretSSD(k, _old, size) => {
                USER_DISKS
                    .update(
                        &user,
                        id,
                        &Disk::SecretSSD(k, hash.as_bytes().to_vec(), size),
                    )
                    .ok_or(Error::NotFound)?;
                Ok(())
            }
        }
    }

    /// 读取指定用户的磁盘信息。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `user`：磁盘所属用户地址。
    /// - `disk_id`：磁盘 ID（k2）。
    ///
    /// # 返回值
    /// - `Some(Disk)`：存在时返回磁盘信息。
    /// - `None`：不存在。
    #[revive(message)]
    pub fn disk(user: Address, disk_id: u64) -> Option<Disk> {
        USER_DISKS.get(&user, disk_id)
    }

    /// 分页查询指定用户的磁盘列表（按 k2 倒序）。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `user`：目标用户地址。
    /// - `start`：分页起始位置（k2 值），`None` 表示从最新开始。
    /// - `size`：每页数量。
    ///
    /// # 返回值
    /// - `Vec<(u64, Disk)>`：磁盘 ID 与信息列表。
    #[revive(message)]
    pub fn user_disks(user: Address, start: Option<u64>, size: u64) -> Vec<(u64, Disk)> {
        USER_DISKS.desc_list(&user, start, size as u32)
    }

    /// 删除当前调用者的指定磁盘。
    ///
    /// 从链上移除调用者拥有的磁盘记录。
    ///
    /// 调用权限：仅磁盘 owner（调用者）可删除。
    ///
    /// # 参数
    /// - `disk_id`：磁盘 ID（k2）。
    ///
    /// # 返回值
    /// - `Ok(())`：删除成功。
    /// - `Err(Error::DelFailed)`：删除失败。
    #[revive(message, write)]
    pub fn del_disk(disk_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        USER_DISKS.clear(&caller, disk_id).ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// 查询 Pod 的扩展信息。
    ///
    /// 通过 Pod ID 查询其所在 worker 的详细信息及 region 名称。
    /// 信息来源于 Subnet 合约的跨合约调用。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `pod_id`：目标 Pod 的唯一标识。
    ///
    /// # 返回值
    /// - `Some((u64, K8sClusterInfo, Vec<u8>))`：
    ///   返回 (worker_id, worker 信息结构体, region 名称字节数组)。
    /// - `None`：Pod 不存在或查询失败。
    #[revive(message)]
    pub fn pod_ext_info(pod_id: u64) -> Option<(u64, K8sClusterInfo, Vec<u8>)> {
        let worker_id = WORKER_OF_POD.get(&pod_id)?;
        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
        let worker: K8sCluster = subnet::subnet::api::worker(&subnet, &worker_id)
            .ok()
            .and_then(|o| o)?;

        let region: Vec<u8> = subnet::subnet::api::region(&subnet, &worker.region_id)
            .ok()
            .and_then(|o| o)
            .unwrap_or_default();

        Some((
            worker_id,
            K8sClusterInfo {
                name: worker.name,
                owner: worker.owner,
                level: worker.level,
                region_id: worker.region_id,
                port: worker.port,
                status: worker.status,
                start_block: worker.start_block,
                stop_block: worker.stop_block,
                terminal_block: worker.terminal_block,
                ip: worker.ip,
            },
            region,
        ))
    }

    /// 按 Pod ID 列表批量查询 Pod 详细信息。
    ///
    /// 批量返回 Pod 元信息、容器列表（附带磁盘信息）、版本号、上次 mint 区块及状态。
    /// 不存在的 Pod ID 会被自动跳过。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `pod_ids`：待查询的 Pod ID 列表。
    ///
    /// # 返回值
    /// - `Vec<(u64, Pod, Vec<(u64, (Container, Vec<Option<Disk>>))>, BlockNumber, BlockNumber, u8)>`：
    ///   每个元素为 (pod_id, pod, 容器及磁盘列表, version, last_mint, status)。
    #[revive(message)]
    pub fn pods_by_ids(
        pod_ids: Vec<u64>,
    ) -> Vec<(
        u64,
        Pod,
        Vec<(u64, (Container, Vec<Option<Disk>>))>,
        BlockNumber,
        BlockNumber,
        u8,
    )> {
        let mut out = Vec::new();
        for pod_id in pod_ids.into_iter() {
            let pod = match PODS.get(&pod_id) {
                Some(p) => p,
                None => continue,
            };
            let containers = POD_CONTAINERS.desc_list(&pod_id, None, 20);
            let mut containers_with_disk = Vec::new();
            for (container_id, container) in containers.into_iter() {
                let disks = container
                    .disk
                    .iter()
                    .map(|c| USER_DISKS.get(&pod.owner, c.id))
                    .collect::<Vec<_>>();
                containers_with_disk.push((container_id, (container, disks)));
            }
            let version = POD_VERSION.get(&pod_id).unwrap_or(0);
            let last_mint = LAST_MINT_BLOCK.get(&pod_id).unwrap_or(0);
            let status = POD_STATUS.get(&pod_id).unwrap_or(0);
            out.push((
                pod_id,
                pod,
                containers_with_disk,
                version,
                last_mint,
                status,
            ));
        }
        out
    }

    /// 从 Cloud 合约向指定地址转账。
    ///
    /// 支持原生代币转账，ERC20 暂不支持。
    /// 从 Cloud 合约的余额中扣除对应金额并转账给目标地址。
    ///
    /// 调用权限：仅治理合约（gov）可调用。
    ///
    /// # 参数
    /// - `asset`：资产类型，目前仅支持 `AssetInfo::Native`。
    /// - `to`：接收转账的地址。
    /// - `amount`：转账金额。
    ///
    /// # 返回值
    /// - `Ok(())`：转账成功。
    /// - `Err(Error::BalanceNotEnough)`：合约余额不足。
    /// - `Err(Error::PayFailed)`：转账失败或资产类型不支持。
    /// - `Err(Error::MustCallByGovContract)`：调用者非治理合约。
    #[revive(message, write)]
    pub fn transfer(asset: AssetInfo, to: Address, amount: U256) -> Result<(), Error> {
        ensure_from_gov()?;
        match asset {
            AssetInfo::Native(_) => {
                ensure!(env().balance() >= amount, Error::BalanceNotEnough);
                env().transfer(&to, &amount).map_err(|_| Error::PayFailed)?;
                Ok(())
            }
            AssetInfo::ERC20(_, _) => Err(Error::PayFailed),
        }
    }

    /// 客户对指定 Pod 发起仲裁。
    ///
    /// 当客户认为 worker 未按约定提供服务时，可提交仲裁申请，
    /// 记录仲裁信息并生成唯一的仲裁 ID。
    ///
    /// 调用权限：仅 Pod 的 owner（调用者）可发起。
    ///
    /// # 参数
    /// - `pod_id`：待仲裁的 Pod ID。
    /// - `amount`：索赔金额。
    /// - `reason`：仲裁原因（字节数组）。
    ///
    /// # 返回值
    /// - `Ok(u64)`：仲裁提交成功，返回仲裁 ID。
    /// - `Err(Error::PodNotFound)`：Pod 不存在。
    /// - `Err(Error::NotPodOwner)`：调用者非 Pod owner。
    /// - `Err(Error::WorkerIdNotFound)`：Pod 未绑定 worker。
    #[revive(message, write)]
    pub fn submit_arbitration(pod_id: u64, amount: U256, reason: Vec<u8>) -> Result<u64, Error> {
        let caller = env().caller();
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        // 仲裁针对的是该 Pod 绑定的 Worker，从保证金中赔付
        // Arbitration targets the Worker bound to this Pod; compensation is paid from the Worker's mortgage
        let worker_id = WORKER_OF_POD.get(&pod_id).ok_or(Error::WorkerIdNotFound)?;

        let id = NEXT_ARBITRATION_ID.get().unwrap_or(0);
        NEXT_ARBITRATION_ID.set(&(id + 1));

        let arbitration = Arbitration {
            id,
            pod_id,
            worker_id,
            claimant: caller,
            amount,
            reason,
            status: ArbitrationStatus::Pending,
            result_amount: U256::ZERO,
            created_at: env().block_number(),
            resolved_at: None,
        };

        ARBITRATIONS.set(&id, &arbitration);
        POD_ARBITRATIONS.insert(&pod_id, &id);

        Ok(id)
    }

    /// 侧链处理仲裁请求。
    ///
    /// 侧链根据链下验证结果，对 pending 状态的仲裁进行裁决：
    /// - 若 `approved` 为 true，可从 worker 保证金中扣除指定金额赔付给 Pod 合约；
    /// - 若 `approved` 为 false，则驳回仲裁。
    ///
    /// 调用权限：仅侧链（side-chain）可调用。
    ///
    /// # 参数
    /// - `arbitration_id`：待处理的仲裁 ID。
    /// - `approved`：是否批准仲裁（true 为批准，false 为驳回）。
    /// - `deduction_amount`：批准时从 worker 保证金扣除的金额。
    ///
    /// # 返回值
    /// - `Ok(())`：处理成功。
    /// - `Err(Error::ArbitrationNotFound)`：仲裁不存在。
    /// - `Err(Error::ArbitrationAlreadyResolved)`：仲裁已被处理过。
    /// - `Err(Error::WorkerMortgageCheckFailed)`：扣除 worker 保证金失败。
    /// - `Err(Error::InvalidSideChainCaller)`：调用者非侧链密钥。
    #[revive(message, write)]
    pub fn resolve_arbitration(
        arbitration_id: u64,
        approved: bool,
        deduction_amount: U256,
    ) -> Result<(), Error> {
        ensure_from_side_chain()?;

        let mut arbitration = ARBITRATIONS
            .get(&arbitration_id)
            .ok_or(Error::ArbitrationNotFound)?;
        ensure!(
            arbitration.status == ArbitrationStatus::Pending,
            Error::ArbitrationAlreadyResolved
        );

        let now = env().block_number();

        if approved {
            let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
            let pod = PODS.get(&arbitration.pod_id).ok_or(Error::PodNotFound)?;

            // 侧链裁决通过后，从 Worker 保证金中扣除赔偿金额，直接赔付到客户 Pod 合约
            // After side-chain approves, slash compensation from Worker's mortgage and pay directly to customer's Pod contract
            if deduction_amount > U256::ZERO {
                subnet::subnet::api::slash_worker_mortgage(
                    &subnet,
                    &arbitration.worker_id,
                    &deduction_amount,
                    &pod.pod_address,
                )
                .map_err(|_| Error::WorkerMortgageCheckFailed)?
                .map_err(|_| Error::WorkerMortgageCheckFailed)?;
            }

            arbitration.status = ArbitrationStatus::Approved;
            arbitration.result_amount = deduction_amount;
        } else {
            arbitration.status = ArbitrationStatus::Rejected;
        }

        arbitration.resolved_at = Some(now);
        ARBITRATIONS.set(&arbitration_id, &arbitration);

        Ok(())
    }

    /// 查询指定仲裁的详细信息。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `arbitration_id`：仲裁 ID。
    ///
    /// # 返回值
    /// - `Some(Arbitration)`：仲裁详情。
    /// - `None`：仲裁不存在。
    #[revive(message)]
    pub fn arbitration(arbitration_id: u64) -> Option<Arbitration> {
        ARBITRATIONS.get(&arbitration_id)
    }

    /// 分页查询指定 Pod 关联的仲裁列表（按内部 k2 倒序）。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `pod_id`：目标 Pod ID。
    /// - `start`：分页起始位置（k2 值），`None` 表示从最新开始。
    /// - `size`：每页数量。
    ///
    /// # 返回值
    /// - `Vec<(u64, Arbitration)>`：仲裁 ID 与详情列表。
    #[revive(message)]
    pub fn pod_arbitrations(
        pod_id: u64,
        start: Option<u64>,
        size: u64,
    ) -> Vec<(u64, Arbitration)> {
        let ids = POD_ARBITRATIONS.desc_list(&pod_id, start, size as u32);
        let mut out = Vec::new();
        for (_k2, id) in ids.into_iter() {
            if let Some(a) = ARBITRATIONS.get(&id) {
                out.push((id, a));
            }
        }
        out
    }

    /// 查询单个 Pod 的基本信息。
    ///
    /// 返回 Pod 元信息、容器列表、版本号及状态。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `pod_id`：目标 Pod ID。
    ///
    /// # 返回值
    /// - `Some((Pod, Vec<(u64, Container)>, BlockNumber, u8))`：
    ///   (Pod 元信息, 容器列表, version, status)。
    /// - `None`：Pod 不存在。
    #[revive(message)]
    pub fn pod(pod_id: u64) -> Option<(Pod, Vec<(u64, Container)>, BlockNumber, u8)> {
        let pod = PODS.get(&pod_id)?;
        let containers = POD_CONTAINERS.list(&pod_id, 0, 20);
        let version = POD_VERSION.get(&pod_id).unwrap_or(0);
        let status = POD_STATUS.get(&pod_id).unwrap_or(0);
        Some((pod, containers, version, status))
    }

    /// 创建一个新的 Pod。
    ///
    /// 调用者需支付一定金额（通过 `value_transferred`），函数会：
    /// 1. 校验目标 worker 是否满足等级和区域要求；
    /// 2. 链上实例化一个新的 Pod 子合约（`pod-polkadot`）；
    /// 3. 保存 Pod 元信息、容器列表，并建立用户与 worker 的索引关系。
    ///
    /// 调用权限：任何人可调用（需支付转账）。
    ///
    /// # 参数
    /// - `name`：Pod 名称（字节数组）。
    /// - `pod_type`：Pod 类型（如 GPU、CPU 等）。
    /// - `tee_type`：TEE 类型（SGX 或 CVM）。
    /// - `containers`：容器配置列表。
    /// - `region_id`：期望部署的区域 ID。
    /// - `level`：Pod 要求的 worker 等级。
    /// - `pay_asset`：支付资产的 ID。
    /// - `worker_id`：指定的 worker ID。
    ///
    /// # 返回值
    /// - `Ok(())`：创建成功。
    /// - `Err(Error::WorkerNotFound)`：worker 不存在。
    /// - `Err(Error::WorkerLevelNotEnough)`：worker 等级不足。
    /// - `Err(Error::RegionNotMatch)`：worker 区域不匹配。
    /// - `Err(Error::PodCodeNotFound)`：Pod 合约代码哈希未设置。
    /// - `Err(Error::PodInstantiateFailed)`：Pod 子合约实例化失败。
    #[revive(message, write)]
    pub fn create_pod(
        name: Vec<u8>,
        pod_type: PodType,
        tee_type: TEEType,
        containers: Vec<Container>,
        region_id: u32,
        level: u8,
        pay_asset: u32,
        worker_id: u64,
    ) -> Result<(), Error> {
        let caller = env().caller();

        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());

        // 通过 Subnet 合约查询目标 Worker 信息，校验 Worker 等级 >= Pod 等级，且区域匹配
        // Query target Worker info via Subnet contract; verify Worker level >= Pod level and region matches
        let worker: K8sCluster = subnet::subnet::api::worker(&subnet, &worker_id)
            .map_err(|_| Error::WorkerNotFound)?
            .ok_or(Error::WorkerNotFound)?;
        ensure!(worker.level >= level, Error::WorkerLevelNotEnough);
        ensure!(worker.region_id == region_id, Error::RegionNotMatch);

        let side_chain_key: Address =
            subnet::subnet::api::side_chain_key(&subnet).map_err(|_| Error::NotFound)?;

        let pod_id = NEXT_POD_ID.get().unwrap_or(0);
        NEXT_POD_ID.set(&(pod_id + 1));

        // 用户创建 Pod 时转入的预付款，随 Pod 子合约实例化传入，作为后续计费的储备金
        // User's prepaid funds transferred during Pod creation, passed to Pod sub-contract instantiation as billing reserve
        let transferred = env().value_transferred();
        let code_hash = POD_CONTRACT_CODE_HASH.get().ok_or(Error::PodCodeNotFound)?;

        // 链上实例化 Pod 子合约，每个 Pod 拥有独立的合约地址和资金池，实现资源隔离
        // Instantiate Pod sub-contract on-chain; each Pod has independent contract address and fund pool for resource isolation
        let (pod_address, _ctor_ret) = pod::pod::api::instantiate_new(
            &code_hash,
            &pod_id,
            &caller,
            &side_chain_key,
            &U256::MAX,
            &transferred,
        )
        .map_err(|_| Error::PodInstantiateFailed)?;

        let now = env().block_number();
        let pod = Pod {
            name,
            owner: caller,
            pod_address,
            ptype: pod_type,
            start_block: now,
            tee_type,
            level,
            pay_asset_id: pay_asset,
        };

        PODS.set(&pod_id, &pod);
        POD_OF_USER.insert(&caller, &pod_id);
        PODS_OF_WORKER.insert(&worker_id, &pod_id);
        WORKER_OF_POD.set(&pod_id, &worker_id);
        LAST_MINT_BLOCK.set(&pod_id, &now);
        Ok(())
    }

    /// 侧链通知启动指定 Pod。
    ///
    /// 侧链完成 Pod 的调度和初始化后，调用此函数将 Pod 状态置为运行中（status = 1），
    /// 并记录 Pod 的密钥。
    ///
    /// 调用权限：仅侧链（side-chain）可调用。
    ///
    /// # 参数
    /// - `pod_id`：待启动的 Pod ID。
    /// - `pod_key`：Pod 的账户公钥。
    ///
    /// # 返回值
    /// - `Ok(())`：启动成功。
    /// - `Err(Error::PodStatusError)`：Pod 状态不允许启动（非 0 或 1）。
    /// - `Err(Error::InvalidSideChainCaller)`：调用者非侧链密钥。
    #[revive(message, write)]
    pub fn start_pod(pod_id: u64, pod_key: AccountId) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let status = POD_STATUS.get(&pod_id).unwrap_or(0);
        if status != 0 && status != 1 {
            return Err(Error::PodStatusError);
        }
        if status == 0 {
            let now = env().block_number();
            LAST_MINT_BLOCK.set(&pod_id, &now);
            POD_STATUS.set(&pod_id, &1);
        }
        POD_KEY.set(&pod_id, &pod_key);
        Ok(())
    }

    /// 停止指定 Pod。
    ///
    /// 将 Pod 状态置为 stopped（status = 3），并从 worker 的 Pod 列表中移除该 Pod，
    /// 释放 worker 资源。停止后 Pod 不再计费。
    ///
    /// 调用权限：仅 Pod 的 owner（调用者）可调用。
    ///
    /// # 参数
    /// - `pod_id`：待停止的 Pod ID。
    ///
    /// # 返回值
    /// - `Ok(())`：停止成功。
    /// - `Err(Error::PodNotFound)`：Pod 不存在。
    /// - `Err(Error::NotPodOwner)`：调用者非 Pod owner。
    /// - `Err(Error::WorkerNotFound)`：Pod 未绑定 worker。
    /// - `Err(Error::DelFailed)`：从 worker 列表移除失败。
    #[revive(message, write)]
    pub fn stop_pod(pod_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        POD_STATUS.set(&pod_id, &3u8);
        let worker_id = WORKER_OF_POD.get(&pod_id).ok_or(Error::WorkerNotFound)?;

        let all = PODS_OF_WORKER.list_all(&worker_id);
        let mut found = None;
        for (k2, v) in all {
            if v == pod_id {
                found = Some(k2);
                break;
            }
        }
        let k2 = found.ok_or(Error::DelFailed)?;
        PODS_OF_WORKER
            .clear(&worker_id, k2)
            .ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// 重启指定 Pod。
    ///
    /// 若 Pod 处于 stopped 状态（status = 3），将其重新加入 worker 列表并恢复计费；
    /// 若处于运行中状态（status = 1），则仅更新版本号触发重新部署。
    ///
    /// 调用权限：仅 Pod 的 owner（调用者）可调用。
    ///
    /// # 参数
    /// - `pod_id`：待重启的 Pod ID。
    ///
    /// # 返回值
    /// - `Ok(())`：重启成功。
    /// - `Err(Error::PodNotFound)`：Pod 不存在。
    /// - `Err(Error::NotPodOwner)`：调用者非 Pod owner。
    /// - `Err(Error::PodStatusError)`：Pod 状态不允许重启（非 1 或 3）。
    /// - `Err(Error::WorkerNotFound)`：Pod 未绑定 worker。
    #[revive(message, write)]
    pub fn restart_pod(pod_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        let status = POD_STATUS.get(&pod_id).unwrap_or(0);
        if status != 1 && status != 3 {
            return Err(Error::PodStatusError);
        }

        if status == 3 {
            let worker_id = WORKER_OF_POD.get(&pod_id).ok_or(Error::WorkerNotFound)?;
            POD_STATUS.set(&pod_id, &0u8);
            PODS_OF_WORKER.insert(&worker_id, &pod_id);
            let now = env().block_number();
            LAST_MINT_BLOCK.set(&pod_id, &now);
        }

        POD_VERSION.set(&pod_id, &env().block_number());
        Ok(())
    }

    /// 对指定 Pod 执行 mint（计费结算）。
    ///
    /// 侧链定期调用此函数，按累积的计费周期扣除 Pod 的资源使用费用：
    /// 1. 检查 Pod 状态是否为运行中（status = 1）；
    /// 2. 计算从上一次 mint 至今经过了多少个计费周期；
    /// 3. 根据容器配置（CPU、内存、GPU、磁盘）和 worker 等级价格计算单周期费用；
    /// 4. 将总费用按资产价格换算后，拆分为：worker 报酬（95%）、平台费（2.5%）、区块奖励池（2.5%）；
    /// 5. 通过 Pod 子合约分别转账给 worker、Cloud 合约（平台费）和区块奖励池。
    ///
    /// 调用权限：仅侧链（side-chain）可调用。
    ///
    /// # 参数
    /// - `pod_id`：待计费的 Pod ID。
    /// - `report`：侧链提交的 Pod 运行报告哈希。
    ///
    /// # 返回值
    /// - `Ok(())`：计费成功或无新周期。
    /// - `Err(Error::PodStatusError)`：Pod 未处于运行状态。
    /// - `Err(Error::WorkerIdNotFound)`：Pod 未绑定 worker。
    /// - `Err(Error::WorkerNotFound)`：worker 信息获取失败。
    /// - `Err(Error::LevelPriceNotFound)`：等级价格未配置。
    /// - `Err(Error::AssetNotFound)`：支付资产未配置或价格异常。
    /// - `Err(Error::PayFailed)`：转账失败。
    /// - `Err(Error::InvalidSideChainCaller)`：调用者非侧链密钥。
    #[revive(message, write)]
    pub fn mint_pod(pod_id: u64, report: H256) -> Result<(), Error> {
        ensure_from_side_chain()?;

        let status = POD_STATUS.get(&pod_id).unwrap_or(0);
        if status != 1 {
            return Err(Error::PodStatusError);
        }

        let now = env().block_number();
        let last_mint = LAST_MINT_BLOCK.get(&pod_id).unwrap_or(0);
        let interval = MINT_INTERVAL.get().unwrap_or(14400);

        // 计算从上一次计费至今经过了多少个计费周期，支持按需结算（非固定周期触发）
        // Calculate billing periods since last settlement; supports on-demand settlement (not fixed-interval triggers)
        let elapsed = now.saturating_sub(last_mint);
        if elapsed == 0 {
            return Ok(());
        }

        let periods = if interval == 0 {
            1u64
        } else {
            (elapsed as u64) / (interval as u64)
        };

        if periods == 0 {
            return Ok(());
        }

        POD_REPORT.set(&pod_id, &report);

        let blocks_to_add = if interval == 0 {
            elapsed
        } else {
            (periods as u32).saturating_mul(interval)
        };
        LAST_MINT_BLOCK.set(&pod_id, &last_mint.saturating_add(blocks_to_add));

        let worker_id = WORKER_OF_POD.get(&pod_id).ok_or(Error::WorkerIdNotFound)?;
        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
        let worker: K8sCluster = subnet::subnet::api::worker(&subnet, &worker_id)
            .map_err(|_| Error::WorkerNotFound)?
            .ok_or(Error::WorkerNotFound)?;

        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        let containers = POD_CONTAINERS.list_all(&pod_id);

        // 按 Pod 的 level 查询对应的服务品质单价，level 越高单价越贵（稳定性/网络品质溢价）
        // Query service quality unit price by Pod's level; higher level = higher unit price (stability/network quality premium)
        let level_price: RunPrice = subnet::subnet::api::level_price(&subnet, &pod.level)
            .map_err(|_| Error::LevelPriceNotFound)?
            .ok_or(Error::LevelPriceNotFound)?;

        // 按容器配置逐条计算资源费用：CPU + 内存 + GPU + 磁盘，区分 SGX/CVM 两种 TEE 类型的不同单价
        // Calculate resource cost per container config: CPU + memory + GPU + disk; SGX and CVM TEE types have different unit prices
        let mut pay_value = U256::ZERO;
        for (_cid, c) in containers.iter() {
            let mut disk_cost = U256::ZERO;
            for d in c.disk.iter() {
                let size_gb = USER_DISKS
                    .get(&pod.owner, d.id)
                    .map(|disk| U256::from(disk.size() as u64))
                    .unwrap_or(U256::ZERO);
                disk_cost = disk_cost + size_gb * U256::from(level_price.disk_per);
            }

            let base = match pod.tee_type {
                TEEType::SGX => {
                    U256::from(c.cpu as u64) * U256::from(level_price.cpu_per)
                        + U256::from(c.mem as u64) * U256::from(level_price.memory_per)
                }
                TEEType::CVM => {
                    U256::from(c.cpu as u64) * U256::from(level_price.cvm_cpu_per)
                        + U256::from(c.mem as u64) * U256::from(level_price.cvm_memory_per)
                }
            };

            let gpu_cost = U256::from(c.gpu as u64) * U256::from(level_price.gpu_per);
            pay_value = pay_value + base + gpu_cost + disk_cost;
        }

        let total_pay_value = pay_value * U256::from(periods);

        let (asset_info, price) = subnet::subnet::api::asset(&subnet, &pod.pay_asset_id)
            .map_err(|_| Error::AssetNotFound)?
            .ok_or(Error::AssetNotFound)?;
        if price == U256::ZERO {
            return Err(Error::AssetNotFound);
        }

        // 将资源费用按资产价格换算为实际代币数量（公式：pay_value * 1000 / price）
        // Convert resource cost to actual token amount using asset price (formula: pay_value * 1000 / price)
        let total_amount = total_pay_value * U256::from(1000u64) / price;

        // 平台费固定 5%，拆分为 2.5% Cloud 运营费 + 2.5% 区块奖励池（由侧链分配给打包节点）
        // Platform fee fixed at 5%, split: 2.5% Cloud operations + 2.5% block reward pool (distributed by side-chain to block producers)
        let platform_total = total_amount * U256::from(500u64) / U256::from(10000u64);
        let cloud_fee = platform_total / U256::from(2u64);
        let block_reward = platform_total - cloud_fee;
        let worker_amount = total_amount - platform_total;
        let cloud_address = env().address();

        // 95% 转给矿工作为任务执行报酬
        // 95% transferred to worker as task execution reward
        if worker_amount > U256::ZERO {
            pod::pod::api::pay_for_woker(&pod.pod_address, &worker.owner, &asset_info, &worker_amount)
                .map_err(|_| Error::PayFailed)?
                .map_err(|_| Error::PayFailed)?;
        }

        // 2.5% 平台费转入 Cloud 合约，累加到 PLATFORM_FEE_TOTAL 供治理方提现
        // 2.5% platform fee transferred to Cloud contract, accumulated in PLATFORM_FEE_TOTAL for governance withdrawal
        if cloud_fee > U256::ZERO {
            pod::pod::api::pay_for_woker(&pod.pod_address, &cloud_address, &asset_info, &cloud_fee)
                .map_err(|_| Error::PayFailed)?
                .map_err(|_| Error::PayFailed)?;
            let current_total = PLATFORM_FEE_TOTAL.get().unwrap_or(U256::ZERO);
            PLATFORM_FEE_TOTAL.set(&(current_total + cloud_fee));
        }

        // 2.5% 区块奖励转入 Cloud 合约的 BLOCK_REWARD_POOL，由侧链验证后分配给打包节点
        // 2.5% block reward transferred to Cloud's BLOCK_REWARD_POOL, distributed to block producers after side-chain verification
        if block_reward > U256::ZERO {
            pod::pod::api::pay_for_woker(&pod.pod_address, &cloud_address, &asset_info, &block_reward)
                .map_err(|_| Error::PayFailed)?
                .map_err(|_| Error::PayFailed)?;
            let current_pool = BLOCK_REWARD_POOL.get().unwrap_or(U256::ZERO);
            BLOCK_REWARD_POOL.set(&(current_pool + block_reward));
        }

        Ok(())
    }

    /// 获取指定 Pod 的最新运行报告。
    ///
    /// 报告由侧链在每次 mint 时更新，用于记录 Pod 的运行状态。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 参数
    /// - `pod_id`：目标 Pod ID。
    ///
    /// # 返回值
    /// - `Some(H256)`：最新的报告哈希。
    /// - `None`：未记录报告。
    #[revive(message)]
    pub fn pod_report(pod_id: u64) -> Option<H256> {
        POD_REPORT.get(&pod_id)
    }

    /// 批量编辑指定 Pod 的容器配置。
    ///
    /// 支持对容器进行插入、更新和删除操作，每次编辑后更新 Pod 版本号。
    ///
    /// 调用权限：仅 Pod 的 owner（调用者）可调用。
    ///
    /// # 参数
    /// - `pod_id`：目标 Pod ID。
    /// - `containers`：容器编辑操作列表，每个元素包含操作类型（INSERT/UPDATE/REMOVE）和容器配置。
    ///
    /// # 返回值
    /// - `Ok(())`：编辑成功。
    /// - `Err(Error::PodNotFound)`：Pod 不存在。
    /// - `Err(Error::NotPodOwner)`：调用者非 Pod owner。
    /// - `Err(Error::NotFound)`：更新时容器不存在。
    /// - `Err(Error::DelFailed)`：删除时容器不存在。
    #[revive(message, write)]
    pub fn edit_container(pod_id: u64, containers: Vec<ContainerInput>) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        for c in containers.iter() {
            match &c.etype {
                EditType::INSERT => add_container(pod_id, c.container.clone())?,
                EditType::UPDATE(container_id) => {
                    update_container(pod_id, *container_id, c.container.clone())?
                }
                EditType::REMOVE(container_id) => {
                    let _ = del_container(pod_id, *container_id)?;
                }
            }
        }

        POD_VERSION.set(&pod_id, &env().block_number());
        Ok(())
    }

    /// 获取当前 Subnet 合约配置的侧链密钥地址。
    ///
    /// 侧链密钥用于验证侧链相关调用的合法性。
    ///
    /// 调用权限：任何人可调用。
    ///
    /// # 返回值
    /// - `Address`：侧链密钥地址；若查询失败则返回 `Address::zero()`。
    #[revive(message)]
    pub fn subnet_side_chain_key() -> Address {
        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
        subnet::subnet::api::side_chain_key(&subnet).unwrap_or(Address::zero())
    }

    fn ensure_from_gov() -> Result<(), Error> {
        let caller = env().caller();
        let gov = GOV_CONTRACT.get().unwrap_or(Address::zero());
        ensure!(caller == gov, Error::MustCallByGovContract);
        Ok(())
    }

    fn ensure_from_side_chain() -> Result<(), Error> {
        let caller = env().caller();
        let key = subnet_side_chain_key();
        ensure!(caller == key, Error::InvalidSideChainCaller);
        Ok(())
    }

    fn add_container(pod_id: u64, container: Container) -> Result<(), Error> {
        POD_CONTAINERS.insert(&pod_id, &container);
        Ok(())
    }

    fn update_container(pod_id: u64, container_id: u64, container: Container) -> Result<(), Error> {
        POD_CONTAINERS
            .update(&pod_id, container_id, &container)
            .ok_or(Error::NotFound)?;
        Ok(())
    }

    fn del_container(pod_id: u64, container_id: u64) -> Result<bool, Error> {
        POD_CONTAINERS
            .clear(&pod_id, container_id)
            .ok_or(Error::DelFailed)?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests;

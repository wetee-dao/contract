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

pub use datas::{
    AssetInfo, Container, ContainerInput, Disk, EditType, K8sCluster, Pod, PodType, RunPrice,
    Secret, TEEType,
};
pub use errors::Error;
pub use primitives::{ensure, ok_or_err};

#[revive_contract]
pub mod cloud {
    use super::*;
    use crate::{Error, ensure};
    use wrevive_api::{AccountId, List2D, Mapping};

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

    #[revive(constructor)]
    pub fn new() -> Result<(), Error> {
        Ok(())
    }

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
        Ok(())
    }

    #[revive(message, write)]
    pub fn set_pod_contract(pod_contract: H256) -> Result<(), Error> {
        ensure_from_gov()?;
        POD_CONTRACT_CODE_HASH.set(&pod_contract);
        Ok(())
    }

    #[revive(message)]
    pub fn pod_contract() -> H256 {
        POD_CONTRACT_CODE_HASH.get().unwrap_or(H256::zero())
    }

    #[revive(message, write)]
    pub fn update_pod_contract(pod_id: u64) -> Result<(), Error> {
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        let code_hash = POD_CONTRACT_CODE_HASH.get().unwrap_or(H256::zero());

        // 调用 Pod 合约的 set_code(code_hash)，返回已 decode 的结果
        pod::pod::api::set_code(&pod.pod_address, &code_hash)
            .map_err(|_| Error::SetCodeFailed)?
            .map_err(|_| Error::SetCodeFailed)?;
        Ok(())
    }

    #[revive(message, write)]
    pub fn set_mint_interval(t: BlockNumber) -> Result<(), Error> {
        ensure_from_gov()?;
        MINT_INTERVAL.set(&t);
        Ok(())
    }

    #[revive(message)]
    pub fn mint_interval() -> BlockNumber {
        MINT_INTERVAL.get().unwrap_or(14400)
    }

    #[revive(message)]
    pub fn subnet_address() -> Address {
        SUBNET_ADDRESS.get().unwrap_or(Address::zero())
    }

    #[revive(message)]
    pub fn charge() -> Result<(), Error> {
        let _ = env().value_transferred();
        Ok(())
    }

    #[revive(message)]
    pub fn balance(asset: AssetInfo) -> U256 {
        match asset {
            AssetInfo::Native(_) => env().balance(),
            AssetInfo::ERC20(_, _) => U256::ZERO, // PolkaVM 暂不支持 ERC20
        }
    }

    #[revive(message)]
    pub fn pod_len() -> u64 {
        NEXT_POD_ID.get().unwrap_or(0)
    }

    /// 分页列出 Pod（按 pod_id 倒序）；`start=None` 表示从最新开始。
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

    /// 当前调用者拥有的 Pod 数量
    #[revive(message)]
    pub fn user_pod_len() -> u64 {
        let caller = env().caller();
        POD_OF_USER.len(&caller)
    }

    /// 当前调用者拥有的 Pod 列表（按内部 k2 倒序分页）
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

    /// worker 上的 Pod 版本信息（用于 side-chain 同步）
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

    /// worker 上的 Pod 列表（按内部 k2 倒序分页）
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

    /// worker 上的 Pod 数量
    #[revive(message)]
    pub fn worker_pod_len(worker_id: u64) -> u64 {
        PODS_OF_WORKER.len(&worker_id)
    }

    /// 查询用户 Secret 列表（按 k2 倒序分页）
    #[revive(message)]
    pub fn user_secrets(user: Address, start: Option<u64>, size: u64) -> Vec<(u64, Secret)> {
        USER_SECRETS.desc_list(&user, start, size as u32)
    }

    /// 读取指定 Secret
    #[revive(message)]
    pub fn secret(user: Address, index: u64) -> Option<Secret> {
        USER_SECRETS.get(&user, index)
    }

    /// 创建 Secret（owner 调用），返回分配到的 id
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

    /// 侧链标记 Secret 已 mint（仅 side-chain 可调用）
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

    /// 删除 Secret（owner 调用）
    #[revive(message, write)]
    pub fn del_secret(index: u64) -> Result<(), Error> {
        let caller = env().caller();
        USER_SECRETS.clear(&caller, index).ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// 创建磁盘（owner 调用），返回分配到的 id
    #[revive(message, write)]
    pub fn create_disk(key: Vec<u8>, size: u32) -> Result<u64, Error> {
        let caller = env().caller();
        let d = Disk::SecretSSD(key, Vec::new(), size);
        USER_DISKS.insert(&caller, &d).ok_or(Error::NotFound)
    }

    /// 侧链更新磁盘加密 key（仅 side-chain 可调用）
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

    /// 读取磁盘信息
    #[revive(message)]
    pub fn disk(user: Address, disk_id: u64) -> Option<Disk> {
        USER_DISKS.get(&user, disk_id)
    }

    /// 用户磁盘列表（按 k2 倒序分页）
    #[revive(message)]
    pub fn user_disks(user: Address, start: Option<u64>, size: u64) -> Vec<(u64, Disk)> {
        USER_DISKS.desc_list(&user, start, size as u32)
    }

    /// 删除磁盘（owner 调用）
    #[revive(message, write)]
    pub fn del_disk(disk_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        USER_DISKS.clear(&caller, disk_id).ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// Pod 扩展信息：worker_id、worker 信息、region 名称/字节（通过 Subnet 查询）
    #[revive(message)]
    pub fn pod_ext_info(pod_id: u64) -> Option<(u64, K8sCluster, Vec<u8>)> {
        let _pod = PODS.get(&pod_id)?;
        let worker_id = WORKER_OF_POD.get(&pod_id)?;
        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
        let worker: K8sCluster = subnet::subnet::api::worker(&subnet, &worker_id)
            .ok()
            .and_then(|o| o)?;
        let region: Vec<u8> = subnet::subnet::api::region(&subnet, &worker.region_id)
            .ok()
            .and_then(|o| o)
            .unwrap_or_default();
        Some((worker_id, worker, region))
    }

    /// 按 pod_id 列表批量查询（附带容器与磁盘信息）
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

    /// 从云合约向指定账户转账（仅 gov 可调用）
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

    #[revive(message)]
    pub fn pod(pod_id: u64) -> Option<(Pod, Vec<(u64, Container)>, BlockNumber, u8)> {
        let pod = PODS.get(&pod_id)?;
        let containers = POD_CONTAINERS.list(&pod_id, 0, 20);
        let version = POD_VERSION.get(&pod_id).unwrap_or(0);
        let status = POD_STATUS.get(&pod_id).unwrap_or(0);
        Some((pod, containers, version, status))
    }

    /// 创建 Pod（可支付）：会在链上实例化 `pod-polkadot` 合约并保存 pod 元信息与容器列表。
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

        // worker 校验：来自 Subnet 合约（使用 interface 同名函数，返回已 decode 结果）
        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
        let worker: K8sCluster = subnet::subnet::api::worker(&subnet, &worker_id)
            .map_err(|_| Error::WorkerNotFound)?
            .ok_or(Error::WorkerNotFound)?;
        ensure!(worker.level >= level, Error::WorkerLevelNotEnough);
        ensure!(worker.region_id == region_id, Error::RegionNotMatch);

        let side_chain_key: Address =
            subnet::subnet::api::side_chain_key(&subnet).map_err(|_| Error::NotFound)?;

        let pod_id = NEXT_POD_ID.get().unwrap_or(0);
        NEXT_POD_ID.set(&(pod_id + 1));

        // 实例化 Pod 合约：使用 interface 的 instantiate_new，返回 (地址, 已 decode 的 constructor 返回值)
        let transferred = env().value_transferred();
        let code_hash = POD_CONTRACT_CODE_HASH.get().unwrap_or(H256::zero());
        // 实例化 Pod 合约，若实例化失败统一映射为 SetCodeFailed；忽略构造函数返回值
        let (pod_address, _ctor_ret) = pod::pod::api::instantiate_new(
            &code_hash,
            &pod_id,
            &caller,
            &side_chain_key,
            &transferred,
            &U256::ZERO,
        )
        .map_err(|_| Error::SetCodeFailed)?;

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

        for c in containers.iter() {
            POD_CONTAINERS.insert(&pod_id, c);
        }
        Ok(())
    }

    /// side-chain 通知启动 Pod（仅 side_chain_key 可调用）
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

    /// 停止 Pod（仅 owner 可调用）：将状态置为 stopped，并从 worker 列表中移除该 pod。
    #[revive(message, write)]
    pub fn stop_pod(pod_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        POD_STATUS.set(&pod_id, &3u8);
        let worker_id = WORKER_OF_POD.get(&pod_id).ok_or(Error::WorkerNotFound)?;

        // 在 pods_of_worker 中清掉对应条目（List2D 目前不收缩，仅 clear 产生空洞）
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

    /// 重启 Pod（仅 owner 可调用）
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

    /// Mint pod：按 `mint_interval` 扣除资源费用并向 worker 支付（仅 side-chain 可调用）。
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

        if now < last_mint.saturating_add(interval) {
            return Ok(());
        }

        POD_REPORT.set(&pod_id, &report);

        // 更新 last_mint_block：若长时间未 mint，直接追到 now；否则按 interval 递增
        if now.saturating_sub(last_mint) > interval.saturating_mul(2) {
            LAST_MINT_BLOCK.set(&pod_id, &now);
        } else {
            LAST_MINT_BLOCK.set(&pod_id, &last_mint.saturating_add(interval));
        }

        // 计算需要支付的费用
        let worker_id = WORKER_OF_POD.get(&pod_id).ok_or(Error::WorkerIdNotFound)?;
        let subnet = SUBNET_ADDRESS.get().unwrap_or(Address::zero());
        let worker: K8sCluster = subnet::subnet::api::worker(&subnet, &worker_id)
            .map_err(|_| Error::WorkerNotFound)?
            .ok_or(Error::WorkerNotFound)?;

        let pod = PODS.get(&pod_id).ok_or(Error::PodNotFound)?;
        let containers = POD_CONTAINERS.list_all(&pod_id);

        let level_price: RunPrice = subnet::subnet::api::level_price(&subnet, &pod.level)
            .map_err(|_| Error::LevelPriceNotFound)?
            .ok_or(Error::LevelPriceNotFound)?;

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

        // Subnet::asset(id) -> Option<(AssetInfo, U256)>
        let (asset_info, price) = subnet::subnet::api::asset(&subnet, &pod.pay_asset_id)
            .map_err(|_| Error::AssetNotFound)?
            .ok_or(Error::AssetNotFound)?;
        if price == U256::ZERO {
            return Err(Error::AssetNotFound);
        }

        // 与 ink 一致：pay_value * 1000 / price
        let amount = pay_value * U256::from(1000u64) / price;

        // 调用 Pod 合约支付给 worker.owner（primitives::AssetInfo 与 Pod 共用同一类型）
        pod::pod::api::pay_for_woker(&pod.pod_address, &worker.owner, &asset_info, &amount)
            .map_err(|_| Error::PayFailed)?
            .map_err(|_| Error::PayFailed)?;
        Ok(())
    }

    /// Pod 的最新 report（由 side-chain 维护）
    #[revive(message)]
    pub fn pod_report(pod_id: u64) -> Option<H256> {
        POD_REPORT.get(&pod_id)
    }

    /// 批量编辑容器（插入/更新/删除），仅 Pod owner 可调用。
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

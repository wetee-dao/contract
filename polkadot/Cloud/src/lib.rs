//! Cloud 合约 — PolkaVM/wrevive 迁移版
//! 从 inks/Cloud 迁移，使用 wrevive-api + wrevive-macro。

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc;
use alloc::vec::Vec;

#[cfg(not(test))]
#[global_allocator]
static ALLOC: pvm_bump_allocator::BumpAllocator<1024> = pvm_bump_allocator::BumpAllocator::new();

mod datas;
mod errors;

use wrevive_api::{env, Address, BlockNumber, H256, ReturnFlags, Storage, U256};
use wrevive_macro::{list_2d, mapping, revive_contract, storage};

pub use datas::{
    AssetInfo, BlockNumber as DataBlockNumber, Container, ContainerInput, Disk, EditType, K8sCluster,
    Pod, PodType, RunPrice, Secret, TEEType,
};
pub use errors::Error;
pub use primitives::{ensure, ok_or_err};

#[revive_contract]
mod cloud {
    use super::*;
    use crate::{ensure, Error};
    use alloc::vec;
    use alloc::vec::Vec;
    use parity_scale_codec::{Decode, Encode};
    use wrevive_api::{List2D, Mapping};

    const SUBNET_WORKER_SELECTOR: [u8; 4] = [0xb5, 0xaf, 0x86, 0x68]; // blake2s("worker")[0..4]
    const SUBNET_SIDE_CHAIN_KEY_SELECTOR: [u8; 4] = [0x33, 0x4c, 0xf9, 0x07]; // blake2s("side_chain_key")[0..4]
    const SUBNET_LEVEL_PRICE_SELECTOR: [u8; 4] = [0x99, 0xdf, 0x4d, 0xf3]; // blake2s("level_price")[0..4]
    const SUBNET_ASSET_SELECTOR: [u8; 4] = [0x48, 0x13, 0x9d, 0x63]; // blake2s("asset")[0..4]
    const SUBNET_REGION_SELECTOR: [u8; 4] = [0x90, 0x52, 0x4f, 0x41]; // blake2s("region")[0..4]

    const POD_SET_CODE_SELECTOR: [u8; 4] = [0x1c, 0x8e, 0xcd, 0x54]; // blake2s("set_code")[0..4]
    const POD_PAY_FOR_WORKER_SELECTOR: [u8; 4] = [0x2c, 0x9a, 0x5b, 0x1c]; // blake2s("pay_for_woker")[0..4]

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
    const POD_KEY: Mapping<u64, H256> = mapping!(b"pod_key");
    const WORKER_OF_POD: Mapping<u64, u64> = mapping!(b"worker_of_pod");

    const POD_OF_USER: List2D<Address, u64, u64> = list_2d!(b"pod_of_user");
    const PODS_OF_WORKER: List2D<u64, u64, u64> = list_2d!(b"pods_of_worker");
    const POD_CONTAINERS: List2D<u64, u64, Container> = list_2d!(b"pod_containers");
    const USER_SECRETS: List2D<Address, u64, Secret> = list_2d!(b"user_secrets");
    const USER_DISKS: List2D<Address, u64, Disk> = list_2d!(b"user_disks");

    #[revive(constructor)]
    pub fn new(subnet_addr: Address, code_hash: H256) -> Result<(), Error> {
        let caller = env().caller();
        GOV_CONTRACT.set(env(), &caller);
        SUBNET_ADDRESS.set(env(), &subnet_addr);
        POD_CONTRACT_CODE_HASH.set(env(), &code_hash);
        MINT_INTERVAL.set(env(), &14400u32);
        NEXT_POD_ID.set(env(), &0u64);
        Ok(())
    }

    #[revive(message)]
    pub fn set_pod_contract(pod_contract: H256) -> Result<(), Error> {
        ensure_from_gov()?;
        POD_CONTRACT_CODE_HASH.set(env(), &pod_contract);
        Ok(())
    }

    #[revive(message)]
    pub fn pod_contract() -> H256 {
        POD_CONTRACT_CODE_HASH.get(env()).unwrap_or(H256::zero())
    }

    #[revive(message)]
    pub fn update_pod_contract(pod_id: u64) -> Result<(), Error> {
        let pod = PODS.get(env(), &pod_id).map_err(|_| Error::PodNotFound)?;
        let code_hash = POD_CONTRACT_CODE_HASH.get(env()).unwrap_or(H256::zero());

        // 调用 Pod 合约的 set_code(code_hash)
        let input = encode_message(POD_SET_CODE_SELECTOR, &code_hash);
        call_contract_raw(&pod.pod_address, &input).map_err(|_| Error::SetCodeFailed)?;

        Ok(())
    }

    #[revive(message)]
    pub fn set_mint_interval(t: BlockNumber) -> Result<(), Error> {
        ensure_from_gov()?;
        MINT_INTERVAL.set(env(), &t);
        Ok(())
    }

    #[revive(message)]
    pub fn mint_interval() -> BlockNumber {
        MINT_INTERVAL.get(env()).unwrap_or(14400)
    }

    #[revive(message)]
    pub fn subnet_address() -> Address {
        SUBNET_ADDRESS.get(env()).unwrap_or(Address::zero())
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
        NEXT_POD_ID.get(env()).unwrap_or(0)
    }

    /// 分页列出 Pod（按 pod_id 倒序）；`start=None` 表示从最新开始。
    #[revive(message)]
    pub fn pods(start: Option<u64>, size: u64) -> Vec<(u64, Pod, Vec<(u64, Container)>, u8)> {
        let total = NEXT_POD_ID.get(env()).unwrap_or(0);
        let mut out = Vec::new();
        if total == 0 || size == 0 {
            return out;
        }

        let mut cur = start.unwrap_or(total - 1);
        if cur >= total {
            cur = total - 1;
        }

        for _ in 0..size {
            if let Ok(pod) = PODS.get(env(), &cur) {
                let containers = POD_CONTAINERS.desc_list(env(), &cur, None, 20);
                let status = POD_STATUS.get(env(), &cur).unwrap_or(0);
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
        POD_OF_USER.len(env(), &caller)
    }

    /// 当前调用者拥有的 Pod 列表（按内部 k2 倒序分页）
    #[revive(message)]
    pub fn user_pods(
        start: Option<u64>,
        size: u64,
    ) -> Vec<(u64, Pod, Vec<(u64, Container)>, u8)> {
        let caller = env().caller();
        let ids = POD_OF_USER.desc_list(env(), &caller, start, size as u32);
        let mut out = Vec::new();
        for (_k2, pod_id) in ids.into_iter() {
            if let Ok(pod) = PODS.get(env(), &pod_id) {
                let containers = POD_CONTAINERS.desc_list(env(), &pod_id, None, 20);
                let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
                out.push((pod_id, pod, containers, status));
            }
        }
        out
    }

    /// worker 上的 Pod 版本信息（用于 side-chain 同步）
    #[revive(message)]
    pub fn worker_pods_version(worker_id: u64) -> Vec<(u64, BlockNumber, BlockNumber, u8)> {
        let ids = PODS_OF_WORKER.desc_list(env(), &worker_id, None, u32::MAX);
        let mut out = Vec::new();
        for (_k2, pod_id) in ids.into_iter() {
            let version = POD_VERSION.get(env(), &pod_id).unwrap_or(0);
            let last_mint = LAST_MINT_BLOCK.get(env(), &pod_id).unwrap_or(0);
            let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
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
        let ids = PODS_OF_WORKER.desc_list(env(), &worker_id, start, size as u32);
        let mut out = Vec::new();
        for (_k2, pod_id) in ids.into_iter() {
            if let Ok(pod) = PODS.get(env(), &pod_id) {
                let containers = POD_CONTAINERS.desc_list(env(), &pod_id, None, 20);
                let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
                out.push((pod_id, pod, containers, status));
            }
        }
        out
    }

    /// worker 上的 Pod 数量
    #[revive(message)]
    pub fn worker_pod_len(worker_id: u64) -> u64 {
        PODS_OF_WORKER.len(env(), &worker_id)
    }

    /// 查询用户 Secret 列表（按 k2 倒序分页）
    #[revive(message)]
    pub fn user_secrets(user: Address, start: Option<u64>, size: u64) -> Vec<(u64, Secret)> {
        USER_SECRETS.desc_list(env(), &user, start, size as u32)
    }

    /// 读取指定 Secret
    #[revive(message)]
    pub fn secret(user: Address, index: u64) -> Option<Secret> {
        USER_SECRETS.get(env(), &user, index)
    }

    /// 创建 Secret（owner 调用），返回分配到的 id
    #[revive(message)]
    pub fn create_secret(key: Vec<u8>, hash: H256) -> Result<u64, Error> {
        let caller = env().caller();
        let s = Secret {
            k: key,
            hash,
            minted: false,
        };
        USER_SECRETS.insert(env(), &caller, &s).ok_or(Error::NotFound)
    }

    /// 侧链标记 Secret 已 mint（仅 side-chain 可调用）
    #[revive(message)]
    pub fn mint_secret(user: Address, index: u64) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let mut s = USER_SECRETS.get(env(), &user, index).ok_or(Error::NotFound)?;
        s.minted = true;
        USER_SECRETS
            .update(env(), &user, index, &s)
            .ok_or(Error::NotFound)?;
        Ok(())
    }

    /// 删除 Secret（owner 调用）
    #[revive(message)]
    pub fn del_secret(index: u64) -> Result<(), Error> {
        let caller = env().caller();
        USER_SECRETS
            .clear(env(), &caller, index)
            .ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// 创建磁盘（owner 调用），返回分配到的 id
    #[revive(message)]
    pub fn create_disk(key: Vec<u8>, size: u32) -> Result<u64, Error> {
        let caller = env().caller();
        let d = Disk::SecretSSD(key, Vec::new(), size);
        USER_DISKS.insert(env(), &caller, &d).ok_or(Error::NotFound)
    }

    /// 侧链更新磁盘加密 key（仅 side-chain 可调用）
    #[revive(message)]
    pub fn update_disk_key(user: Address, id: u64, hash: H256) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let disk = USER_DISKS.get(env(), &user, id).ok_or(Error::NotFound)?;
        match disk {
            Disk::SecretSSD(k, _old, size) => {
                USER_DISKS
                    .update(env(), &user, id, &Disk::SecretSSD(k, hash.as_bytes().to_vec(), size))
                    .ok_or(Error::NotFound)?;
                Ok(())
            }
        }
    }

    /// 读取磁盘信息
    #[revive(message)]
    pub fn disk(user: Address, disk_id: u64) -> Option<Disk> {
        USER_DISKS.get(env(), &user, disk_id)
    }

    /// 用户磁盘列表（按 k2 倒序分页）
    #[revive(message)]
    pub fn user_disks(user: Address, start: Option<u64>, size: u64) -> Vec<(u64, Disk)> {
        USER_DISKS.desc_list(env(), &user, start, size as u32)
    }

    /// 删除磁盘（owner 调用）
    #[revive(message)]
    pub fn del_disk(disk_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        USER_DISKS
            .clear(env(), &caller, disk_id)
            .ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// Pod 扩展信息：worker_id、worker 信息、region 名称/字节（通过 Subnet 查询）
    #[revive(message)]
    pub fn pod_ext_info(pod_id: u64) -> Option<(u64, K8sCluster, Vec<u8>)> {
        let _pod = PODS.get(env(), &pod_id).ok()?;
        let worker_id = WORKER_OF_POD.get(env(), &pod_id).ok()?;
        let worker: Option<K8sCluster> = subnet_call_decode(SUBNET_WORKER_SELECTOR, &worker_id).ok()?;
        let worker = worker?;
        let region: Option<Vec<u8>> = subnet_call_decode(SUBNET_REGION_SELECTOR, &worker.region_id).ok()?;
        Some((worker_id, worker, region.unwrap_or_default()))
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
            let pod = match PODS.get(env(), &pod_id) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let containers = POD_CONTAINERS.desc_list(env(), &pod_id, None, 20);
            let mut containers_with_disk = Vec::new();
            for (container_id, container) in containers.into_iter() {
                let disks = container
                    .disk
                    .iter()
                    .map(|c| USER_DISKS.get(env(), &pod.owner, c.id))
                    .collect::<Vec<_>>();
                containers_with_disk.push((container_id, (container, disks)));
            }
            let version = POD_VERSION.get(env(), &pod_id).unwrap_or(0);
            let last_mint = LAST_MINT_BLOCK.get(env(), &pod_id).unwrap_or(0);
            let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
            out.push((pod_id, pod, containers_with_disk, version, last_mint, status));
        }
        out
    }

    /// 从云合约向指定账户转账（仅 gov 可调用）
    #[revive(message)]
    pub fn transfer(asset: AssetInfo, to: Address, amount: U256) -> Result<(), Error> {
        ensure_from_gov()?;
        match asset {
            AssetInfo::Native(_) => {
                ensure!(env().balance() >= amount, Error::BalanceNotEnough);
                let r = env().call(
                    pallet_revive_uapi::CallFlags::empty(),
                    &to,
                    1_000_000,
                    1_000_000,
                    &U256::ZERO,
                    &amount,
                    &[],
                    None,
                );
                r.map_err(|_| Error::PayFailed)?;
                Ok(())
            }
            AssetInfo::ERC20(_, _) => Err(Error::PayFailed),
        }
    }

    #[revive(message)]
    pub fn pod(pod_id: u64) -> Option<(Pod, Vec<(u64, Container)>, BlockNumber, u8)> {
        let pod = PODS.get(env(), &pod_id).ok()?;
        let containers = POD_CONTAINERS.list(env(), &pod_id, 0, 20);
        let version = POD_VERSION.get(env(), &pod_id).unwrap_or(0);
        let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
        Some((pod, containers, version, status))
    }

    /// 创建 Pod（可支付）：会在链上实例化 `pod-polkadot` 合约并保存 pod 元信息与容器列表。
    #[revive(message)]
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

        // worker 校验：来自 Subnet 合约
        let worker: Option<K8sCluster> = subnet_call_decode(SUBNET_WORKER_SELECTOR, &worker_id)?;
        let worker = worker.ok_or(Error::WorkerNotFound)?;
        ensure!(worker.level >= level, Error::WorkerLevelNotEnough);
        ensure!(worker.region_id == region_id, Error::RegionNotMatch);

        let side_chain_key: Address = subnet_call_decode(SUBNET_SIDE_CHAIN_KEY_SELECTOR, &())?;

        let pod_id = NEXT_POD_ID.get(env()).unwrap_or(0);
        NEXT_POD_ID.set(env(), &(pod_id + 1));

        // 实例化 Pod 合约：constructor 参数 SCALE 编码（无 selector）
        let transferred = env().value_transferred();
        let code_hash = POD_CONTRACT_CODE_HASH.get(env()).unwrap_or(H256::zero());
        let input_data = (pod_id, caller, side_chain_key).encode();
        let mut addr = [0u8; 20];
        let r = env().instantiate(
            pallet_revive_uapi::CallFlags::empty(),
            code_hash.as_bytes(),
            10_000_000,
            10_000_000,
            U256::ZERO.as_bytes(),
            transferred.as_bytes(),
            &input_data,
            &mut addr,
            None,
        );
        r.map_err(|_| Error::SetCodeFailed)?;
        let pod_address = Address::from(addr);

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

        PODS.set(env(), &pod_id, &pod);
        POD_OF_USER.insert(env(), &caller, &pod_id);
        PODS_OF_WORKER.insert(env(), &worker_id, &pod_id);
        WORKER_OF_POD.set(env(), &pod_id, &worker_id);
        LAST_MINT_BLOCK.set(env(), &pod_id, &now);

        for c in containers.iter() {
            POD_CONTAINERS.insert(env(), &pod_id, c);
        }
        Ok(())
    }

    /// side-chain 通知启动 Pod（仅 side_chain_key 可调用）
    #[revive(message)]
    pub fn start_pod(pod_id: u64, pod_key: H256) -> Result<(), Error> {
        ensure_from_side_chain()?;
        let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
        if status != 0 && status != 1 {
            return Err(Error::PodStatusError);
        }
        if status == 0 {
            let now = env().block_number();
            LAST_MINT_BLOCK.set(env(), &pod_id, &now);
            POD_STATUS.set(env(), &pod_id, &1);
        }
        POD_KEY.set(env(), &pod_id, &pod_key);
        Ok(())
    }

    /// 停止 Pod（仅 owner 可调用）：将状态置为 stopped，并从 worker 列表中移除该 pod。
    #[revive(message)]
    pub fn stop_pod(pod_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(env(), &pod_id).map_err(|_| Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        POD_STATUS.set(env(), &pod_id, &3u8);
        let worker_id = WORKER_OF_POD.get(env(), &pod_id).map_err(|_| Error::WorkerNotFound)?;

        // 在 pods_of_worker 中清掉对应条目（List2D 目前不收缩，仅 clear 产生空洞）
        let all = PODS_OF_WORKER.list_all(env(), &worker_id);
        let mut found = None;
        for (k2, v) in all {
            if v == pod_id {
                found = Some(k2);
                break;
            }
        }
        let k2 = found.ok_or(Error::DelFailed)?;
        PODS_OF_WORKER.clear(env(), &worker_id, k2).ok_or(Error::DelFailed)?;
        Ok(())
    }

    /// 重启 Pod（仅 owner 可调用）
    #[revive(message)]
    pub fn restart_pod(pod_id: u64) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(env(), &pod_id).map_err(|_| Error::PodNotFound)?;
        ensure!(pod.owner == caller, Error::NotPodOwner);

        let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
        if status != 1 && status != 3 {
            return Err(Error::PodStatusError);
        }

        if status == 3 {
            let worker_id = WORKER_OF_POD.get(env(), &pod_id).map_err(|_| Error::WorkerNotFound)?;
            POD_STATUS.set(env(), &pod_id, &0u8);
            PODS_OF_WORKER.insert(env(), &worker_id, &pod_id);
            let now = env().block_number();
            LAST_MINT_BLOCK.set(env(), &pod_id, &now);
        }

        POD_VERSION.set(env(), &pod_id, &env().block_number());
        Ok(())
    }

    /// Mint pod：按 `mint_interval` 扣除资源费用并向 worker 支付（仅 side-chain 可调用）。
    #[revive(message)]
    pub fn mint_pod(pod_id: u64, report: H256) -> Result<(), Error> {
        ensure_from_side_chain()?;

        let status = POD_STATUS.get(env(), &pod_id).unwrap_or(0);
        if status != 1 {
            return Err(Error::PodStatusError);
        }

        let now = env().block_number();
        let last_mint = LAST_MINT_BLOCK.get(env(), &pod_id).unwrap_or(0);
        let interval = MINT_INTERVAL.get(env()).unwrap_or(14400);

        if now < last_mint.saturating_add(interval) {
            return Ok(());
        }

        POD_REPORT.set(env(), &pod_id, &report);

        // 更新 last_mint_block：若长时间未 mint，直接追到 now；否则按 interval 递增
        if now.saturating_sub(last_mint) > interval.saturating_mul(2) {
            LAST_MINT_BLOCK.set(env(), &pod_id, &now);
        } else {
            LAST_MINT_BLOCK.set(env(), &pod_id, &last_mint.saturating_add(interval));
        }

        // 计算需要支付的费用
        let worker_id = WORKER_OF_POD
            .get(env(), &pod_id)
            .map_err(|_| Error::WorkerIdNotFound)?;
        let worker: Option<K8sCluster> = subnet_call_decode(SUBNET_WORKER_SELECTOR, &worker_id)?;
        let worker = worker.ok_or(Error::WorkerNotFound)?;

        let pod = PODS.get(env(), &pod_id).map_err(|_| Error::PodNotFound)?;
        let containers = POD_CONTAINERS.list_all(env(), &pod_id);

        let level_price: Option<RunPrice> = subnet_call_decode(SUBNET_LEVEL_PRICE_SELECTOR, &pod.level)?;
        let level_price = level_price.ok_or(Error::LevelPriceNotFound)?;

        let mut pay_value = U256::ZERO;
        for (_cid, c) in containers.iter() {
            let mut disk_cost = U256::ZERO;
            for d in c.disk.iter() {
                let size_gb = USER_DISKS
                    .get(env(), &pod.owner, d.id)
                    .map(|disk| U256::from(disk.size() as u64))
                    .unwrap_or(U256::ZERO);
                disk_cost = disk_cost + size_gb * U256::from(level_price.disk_per);
            }

            let base = match pod.tee_type {
                TEEType::SGX => U256::from(c.cpu as u64) * U256::from(level_price.cpu_per)
                    + U256::from(c.mem as u64) * U256::from(level_price.memory_per),
                TEEType::CVM => U256::from(c.cpu as u64) * U256::from(level_price.cvm_cpu_per)
                    + U256::from(c.mem as u64) * U256::from(level_price.cvm_memory_per),
            };

            let gpu_cost = U256::from(c.gpu as u64) * U256::from(level_price.gpu_per);
            pay_value = pay_value + base + gpu_cost + disk_cost;
        }

        // Subnet::asset(id) -> Option<(AssetInfo, U256)>
        let pay_asset: Option<(AssetInfo, U256)> = subnet_call_decode(SUBNET_ASSET_SELECTOR, &pod.pay_asset_id)?;
        let (asset_info, price) = pay_asset.ok_or(Error::AssetNotFound)?;
        if price == U256::ZERO {
            return Err(Error::AssetNotFound);
        }

        // 与 ink 一致：pay_value * 1000 / price
        let amount = pay_value * U256::from(1000u64) / price;

        // 调用 Pod 合约支付给 worker.owner
        let input = encode_message(POD_PAY_FOR_WORKER_SELECTOR, &(worker.owner, asset_info, amount));
        let raw = call_contract_raw(&pod.pod_address, &input).map_err(|_| Error::PayFailed)?;
        let mut cur = &raw[..];
        let r: Result<(), u8> = Result::<(), u8>::decode(&mut cur).map_err(|_| Error::PayFailed)?;
        r.map_err(|_| Error::PayFailed)?;
        Ok(())
    }

    /// Pod 的最新 report（由 side-chain 维护）
    #[revive(message)]
    pub fn pod_report(pod_id: u64) -> Option<H256> {
        POD_REPORT.get(env(), &pod_id).ok()
    }

    /// 批量编辑容器（插入/更新/删除），仅 Pod owner 可调用。
    #[revive(message)]
    pub fn edit_container(pod_id: u64, containers: Vec<ContainerInput>) -> Result<(), Error> {
        let caller = env().caller();
        let pod = PODS.get(env(), &pod_id).map_err(|_| Error::PodNotFound)?;
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

        POD_VERSION.set(env(), &pod_id, &env().block_number());
        Ok(())
    }

    #[revive(message)]
    pub fn set_code(_code_hash: H256) -> Result<(), Error> {
        ensure_from_gov()?;
        Err(Error::SetCodeFailed)
    }

    fn ensure_from_gov() -> Result<(), Error> {
        let caller = env().caller();
        let gov = GOV_CONTRACT.get(env()).unwrap_or(Address::zero());
        ensure!(caller == gov, Error::MustCallByGovContract);
        Ok(())
    }

    fn ensure_from_side_chain() -> Result<(), Error> {
        let caller = env().caller();
        let key = subnet_side_chain_key();
        ensure!(caller == key, Error::InvalidSideChainCaller);
        Ok(())
    }

    fn subnet_side_chain_key() -> Address {
        subnet_call_decode(SUBNET_SIDE_CHAIN_KEY_SELECTOR, &()).unwrap_or(Address::zero())
    }

    fn subnet_call_decode<R: Decode, A: Encode>(selector: [u8; 4], args: &A) -> Result<R, Error> {
        let subnet = SUBNET_ADDRESS.get(env()).unwrap_or(Address::zero());
        let input = encode_message(selector, args);
        let raw = call_contract_raw(&subnet, &input)?;
        let mut cur = &raw[..];
        R::decode(&mut cur).map_err(|_| Error::NotFound)
    }

    fn encode_message<A: Encode>(selector: [u8; 4], args: &A) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + 32);
        out.extend_from_slice(&selector);
        out.extend_from_slice(&args.encode());
        out
    }

    fn add_container(pod_id: u64, container: Container) -> Result<(), Error> {
        POD_CONTAINERS.insert(env(), &pod_id, &container);
        Ok(())
    }

    fn update_container(pod_id: u64, container_id: u64, container: Container) -> Result<(), Error> {
        POD_CONTAINERS
            .update(env(), &pod_id, container_id, &container)
            .ok_or(Error::NotFound)?;
        Ok(())
    }

    fn del_container(pod_id: u64, container_id: u64) -> Result<bool, Error> {
        POD_CONTAINERS
            .clear(env(), &pod_id, container_id)
            .ok_or(Error::DelFailed)?;
        Ok(true)
    }

    fn call_contract_raw(callee: &Address, input_data: &[u8]) -> Result<Vec<u8>, Error> {
        let r = env().call(
            pallet_revive_uapi::CallFlags::empty(),
            callee,
            10_000_000,
            10_000_000,
            &U256::ZERO,
            &U256::ZERO,
            input_data,
            None,
        );
        r.map_err(|_| Error::NotFound)?;

        let size = env().return_data_size() as usize;
        let mut buf = vec![0u8; size];
        let mut slice = buf.as_mut_slice();
        env().return_data_copy(&mut slice, 0);
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrevive_api::with_engine;

    #[test]
    fn deploy_and_getters() {
        let subnet_addr = Address::from([1u8; 20]);
        let code_hash = H256::from([2u8; 32]);
        with_engine(|e| {
            e.reset();
            e.set_caller([3u8; 20]);
        });
        let _ = cloud::new(subnet_addr, code_hash);
        assert_eq!(cloud::subnet_address(), subnet_addr);
        assert_eq!(cloud::mint_interval(), 14400);
        assert_eq!(cloud::pod_len(), 0);
        assert_eq!(cloud::pod_contract(), H256::from([2u8; 32]));
    }
}

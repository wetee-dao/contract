//! 云合约数据类型（SCALE 编码，使用 wrevive_api 类型）

#![allow(unused_imports)]
extern crate alloc;

use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use wrevive_api::{Address, Bytes, H256, U256};

pub type BlockNumber = u32;
/// 与 ink AccountId 兼容的 32 字节账户标识（用于 pod_key 等）。
pub type AccountId = [u8; 32];

/// 与 Subnet 合约保持一致的 IP 表示（用于解码 Subnet::worker 返回值）。
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Ip {
    pub ipv4: Option<u32>,
    pub ipv6: Option<u128>,
    pub domain: Option<Bytes>,
}

/// 与 Subnet 合约保持一致的 K8sCluster（worker）结构（用于跨合约调用解码）。
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct K8sCluster {
    pub name: Bytes,
    pub owner: Address,
    pub level: u8,
    pub region_id: u32,
    pub start_block: BlockNumber,
    pub stop_block: Option<BlockNumber>,
    pub terminal_block: Option<BlockNumber>,
    pub p2p_id: AccountId,
    pub ip: Ip,
    pub port: u32,
    pub status: u8,
}

/// 与 Subnet 合约保持一致的运行价格（用于 mint_pod 计算费用）。
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct RunPrice {
    pub cpu_per: u64,
    pub cvm_cpu_per: u64,
    pub memory_per: u64,
    pub cvm_memory_per: u64,
    pub disk_per: u64,
    pub gpu_per: u64,
}

/// Pod 元数据（PolkaVM 下存合约地址，无 PodRef）
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Pod {
    pub name: Bytes,
    pub owner: Address,
    /// 部署后的 Pod 合约地址
    pub pod_address: Address,
    pub ptype: PodType,
    pub start_block: BlockNumber,
    pub tee_type: TEEType,
    pub level: u8,
    pub pay_asset_id: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Encode, Decode)]
pub enum PodType {
    #[default]
    CPU,
    GPU,
    SCRIPT,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Encode, Decode)]
pub enum TEEType {
    #[default]
    SGX,
    CVM,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Service {
    Tcp(u16),
    Udp(u16),
    Http(u16),
    Https(u16),
    ProjectTcp(u16),
    ProjectUdp(u16),
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Command {
    SH(Bytes),
    BASH(Bytes),
    ZSH(Bytes),
    NONE,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum EditType<T> {
    INSERT,
    UPDATE(T),
    REMOVE(T),
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Env {
    Env(Bytes, Bytes),
    File(Bytes, Bytes),
    Encrypt(Bytes, u64),
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct ContainerDisk {
    pub id: u64,
    pub path: Bytes,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct ContainerInput {
    pub etype: EditType<u64>,
    pub container: Container,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Container {
    pub image: Bytes,
    pub command: Command,
    pub port: Vec<Service>,
    pub cpu: u32,
    pub mem: u32,
    pub disk: Vec<ContainerDisk>,
    pub gpu: u32,
    pub env: Vec<Env>,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum Disk {
    SecretSSD(Bytes, Bytes, u32),
}

impl Disk {
    pub fn size(&self) -> u32 {
        match self {
            Disk::SecretSSD(_, _, size) => *size,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Secret {
    pub k: Bytes,
    pub hash: H256,
    pub minted: bool,
}

impl Default for Secret {
    fn default() -> Self {
        Self {
            k: Bytes::new(),
            hash: H256::zero(),
            minted: false,
        }
    }
}

/// 资产类型（与 Subnet/Pod 一致）
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum AssetInfo {
    Native(Bytes),
    ERC20(Bytes, H256),
}

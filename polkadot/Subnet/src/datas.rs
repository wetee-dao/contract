//! 子网合约数据类型（SCALE 编码，使用 wrevive_api 类型）

use parity_scale_codec::{Decode, Encode};
use wrevive_api::{Address, Bytes, H256, U256};

pub type NodeID = u64;
/// 链上 AccountId 用 32 字节表示（与 ink AccountId 兼容）
pub type AccountId = [u8; 32];
pub type BlockNumber = u32;

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct Ip {
    pub ipv4: Option<u32>,
    pub ipv6: Option<u128>,
    pub domain: Option<Bytes>,
}

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

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct SecretNode {
    pub name: Bytes,
    pub owner: Address,
    pub validator_id: AccountId,
    pub p2p_id: AccountId,
    pub start_block: BlockNumber,
    pub terminal_block: Option<BlockNumber>,
    pub ip: Ip,
    pub port: u32,
    pub status: u8,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct AssetDeposit {
    pub amount: U256,
    pub cpu: u32,
    pub cvm_cpu: u32,
    pub mem: u32,
    pub cvm_mem: u32,
    pub disk: u32,
    pub gpu: u32,
    pub deleted: Option<BlockNumber>,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct RunPrice {
    pub cpu_per: u64,
    pub cvm_cpu_per: u64,
    pub memory_per: u64,
    pub cvm_memory_per: u64,
    pub disk_per: u64,
    pub gpu_per: u64,
}

#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub struct EpochInfo {
    pub epoch: u32,
    pub epoch_solt: u32,
    pub last_epoch_block: BlockNumber,
    pub now: BlockNumber,
    pub side_chain_pub: Address,
}

/// 资产类型（与 Pod/inks primitives 兼容）
#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum AssetInfo {
    Native(Bytes),
    ERC20(Bytes, H256),
}

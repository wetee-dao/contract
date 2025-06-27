use ink::{env::BlockNumber, prelude::vec::Vec, primitives::AccountId, Address, U256};

pub type NodeID = u128;
// pub type AssetId = u64;

#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct K8sCluster {
    /// name of the K8sCluster.
    /// 集群名字
    pub name: Vec<u8>,
    /// owner of K8sCluster
    /// 创建者
    pub owner: Address,
    /// cluster hardware and security level
    /// 集群的硬件和安全等级
    pub level: u8,
    /// The block that creates the K8sCluster
    /// App创建的区块
    pub start_block: BlockNumber,
    /// Stop time
    /// 停止时间
    pub stop_block: Option<BlockNumber>,
    /// terminal time
    /// 终止时间
    pub terminal_block: Option<BlockNumber>,
	// subnet ed25519 p2p
	pub p2p_id: AccountId,
    /// ip of service
    /// 服务端口号
    pub ip: Ip,
    /// port of service
    /// 服务端口号
    pub port: u32,
    /// State of the App
    /// K8sCluster 状态
    pub status: u8,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct SecretNode {
    /// name of the K8sCluster.
    /// 集群名字
    pub name: Vec<u8>,
    /// owner of K8sCluster
    /// 创建者
    pub owner: Address,
    // subnet ed25519 validator
	pub validator_id: AccountId,
	// subnet ed25519 p2p
	pub p2p_id: AccountId,
    /// The block that creates the K8sCluster
    /// App创建的区块
    pub start_block: BlockNumber,
    /// Stop time
    /// 停止时间
    pub stop_block: Option<BlockNumber>,
    /// terminal time
    /// 终止时间
    pub terminal_block: Option<BlockNumber>,
    /// ip of service
    /// 服务端口号
    pub ip: Ip,
    /// port of service
    /// 服务端口号
    pub port: u32,
    /// State of the App
    /// K8sCluster 状态
    pub status: u8,
}

/// Ip
#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Ip {
    pub ipv4: Option<u32>,
    pub ipv6: Option<u128>,
    pub domain: Option<Vec<u8>>,
}

/// 质押数据
/// deposit of computing resource
#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct AssetDeposit {
    /// Deposit amount
    /// 质押金额
    pub amount: U256,
    /// cpu
    pub cpu: u32,
    pub cvm_cpu: u32,
    /// memory
    pub mem: u32,
    pub cvm_mem: u32,
    /// disk
    pub disk: u32,
    /// gpu
    pub gpu: u32,
    /// deleted timestamp
    pub deleted: Option<BlockNumber>,
}

/// 抵押价格
/// DepositPrice
#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct DepositPrice {
    /// SGX cpu price of one core
    pub cpu: u32,
    /// TDX/SNP cpu price of one core
    pub cvm_cpu_per: u32,
    /// SGX memory price of 1G
    pub memory_per: u32,
    /// TDX/SNP memory price of 1G
    pub cvm_memory_per: u32,
    /// Disk price of 1G
    pub disk_per: u32,
    /// GPU price of one GPU
    pub gpu_per: u32,
}

primitives::define_map!(Workers, NodeID, K8sCluster);

primitives::define_double_map!(WorkerMortgages, u128, AssetDeposit);

primitives::define_map!(Secrets, NodeID, SecretNode);

// mortgage_of_worker: Mapping<NodeID, Vec<u128>>,
// primitives::define_double_map!(MortgageOfWorker, NodeID, u128);
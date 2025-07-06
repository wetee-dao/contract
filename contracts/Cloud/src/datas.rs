use ink::{env::BlockNumber, prelude::vec::Vec, Address};

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Pod {
    /// Pod name
    pub name: Vec<u8>,
    /// Owner of pod
    /// 创建者
    pub owner: Address,
    /// Contract id
    /// 合约账户
    pub contract: pod::PodRef,
    /// Type of pod,Different pods will be called to different clusters.
    pub ptype: PodType,
    /// The block that creates the App
    /// App创建的区块
    pub start_block: BlockNumber,
    /// tee version
    /// tee 版本
    pub tee_type: TEEType,
}

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PodType {
    // Only use CPU 
    CpuService,
    // Use GPU/CPU
    GpuService,
    // Script to execute one-time or as a scheduled task
    Script,
}
impl Default for PodType {
    fn default() -> Self {
        PodType::CpuService
    }
}

/// 网络设置
/// disk setting
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Service {
    /// TCP
    Tcp(u16),
    /// UDP
    Udp(u16),
    /// TCP
    Http(u16),
    /// TCP
    Https(u16),
    /// Project Tcp
    ProjectTcp(u16),
    /// Project Udp
    ProjectUdp(u16),
}
impl Default for Service {
    fn default() -> Self {
        Service::Http(80)
    }
}

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Command {
    /// /bin/sh 启动
    SH(Vec<u8>),
    /// /bin/bash 启动
    BASH(Vec<u8>),
    /// /bin/zsh 启动
    ZSH(Vec<u8>),
    NONE,
}
impl Default for Command {
    fn default() -> Self {
        Command::SH("".as_bytes().to_vec()) // 默认为TCP协议，端口为0
    }
}

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum EditType {
    /// INSERT
    INSERT,
    /// UPDATE
    UPDATE(u16),
    /// REMOVE
    REMOVE(u16),
}
impl Default for EditType {
    fn default() -> Self {
        EditType::INSERT
    }
}

/// App setting
/// 应用设置
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct EnvInput {
    /// edit type
    pub etype: EditType,
    /// container index
    pub index: u16,
    /// key
    pub k: EnvKey,
    /// value
    pub v: Vec<u8>,
}
impl Default for EnvInput {
    fn default() -> Self {
        EnvInput {
            etype: EditType::INSERT,
            index: 1,
            k: EnvKey::Env(Vec::new()),
            v: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum EnvKey {
    /// Env 环境变量
    Env(Vec<u8>),
    /// UPDATE
    File(Vec<u8>),
}
impl Default for EnvKey {
    fn default() -> Self {
        EnvKey::Env("".as_bytes().to_vec()) // 默认为TCP协议，端口为0
    }
}

/// 储存类型
/// disk setting
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum DiskClass {
    /// TCP
    SSD(Vec<u8>),
}
impl Default for DiskClass {
    fn default() -> Self {
        DiskClass::SSD("".as_bytes().to_vec()) // 默认为TCP协议，端口为0
    }
}

/// 储存设置
/// disk setting
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Disk {
    /// key
    pub path: DiskClass,
    /// value
    pub size: u32,
}
impl Default for Disk {
    fn default() -> Self {
        Disk {
            path: DiskClass::SSD("".as_bytes().to_vec()),
            size: 1,
        }
    }
}

/// App specific information
/// 程序信息
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Container {
    pub name: Vec<u8>,
    /// img of the App.
    /// image 目标宗旨
    pub image: Vec<u8>,
    /// command of service
    /// 执行命令
    pub command: Command,
    /// port of service
    /// 服务端口号
    pub port: Vec<Service>,
    /// cpu memory disk
    /// cpu memory disk
    pub cr: CR,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            name: Vec::new(),
            image: Vec::new(),
            command: Command::NONE,
            port: Vec::new(),
            cr: CR::default(),
        }
    }
}

/// 计算资源
/// computing resource
#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CR {
    pub cpu: u32,
    pub mem: u32,
    pub disk: Vec<Disk>,
    pub gpu: u32,
}

/// TEEType
/// TEE 实现版本
#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum TEEType {
    #[default]
    SGX,
    CVM,
}

primitives::define_map!(Pods, u64, Pod);

primitives::define_double_map!(UserPods, Address, u64);

primitives::define_double_map!(WorkerPods, u64, u64);

primitives::define_double_map!(Containers, u64, Container);
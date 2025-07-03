use ink::{env::BlockNumber, prelude::vec::Vec, Address};

#[derive(Clone,Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Pod {
    pub id: u64,
    /// creator of app
    /// 创建者
    pub creator: Address,
    /// contract id
    /// 合约账户
    pub contract_id: Address,
    /// The block that creates the App
    /// App创建的区块
    pub start_block: BlockNumber,
    /// name of the app.
    /// 程序名字
    pub name: Vec<u8>,
    /// app template id
    pub template_id: Option<u128>,
    /// img of the App.
    /// image 目标宗旨
    pub image: Vec<u8>,
    /// meta of the App.
    /// 应用元数据
    pub meta: Vec<u8>,
    /// command of service
    /// 执行命令
    pub command: Command,
    /// port of service
    /// 服务端口号
    pub port: Vec<Service>,
    /// cpu memory disk
    /// cpu memory disk
    pub cr: Cr,
    /// side container
    /// 附属容器
    pub side_container: Vec<Container>,
    /// tee version
    /// tee 版本
    pub tee_version: TEEVersion,
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
    pub cr: Cr,
}
impl Default for Container {
    fn default() -> Self {
        Container {
            image: "".as_bytes().to_vec(),
            command: Command::NONE,
            port: Vec::new(),
            cr: Cr::default(),
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
pub struct Cr {
    pub cpu: u32,
    pub mem: u32,
    pub disk: Vec<Disk>,
    pub gpu: u32,
}

/// TEEVersion
/// TEE 实现版本
#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum TEEVersion {
    #[default]
    SGX,
    CVM,
}

primitives::define_map!(Pods, u64, Pod);

primitives::define_double_map!(UserPods, Address, u64);

primitives::define_double_map!(WorkerPods, Address, u64);
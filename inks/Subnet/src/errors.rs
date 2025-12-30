/// Error types for Subnet contract
/// 子网合约错误类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Not enough balance / 余额不足
    NotEnoughBalance,
    /// Caller must be main contract / 调用者必须是主合约
    MustCallByMainContract,
    /// Worker does not exist / 工作节点不存在
    WorkerNotExist,
    /// Worker is not owned by caller / 工作节点不属于调用者
    WorkerNotOwnedByCaller,
    /// Worker status is not ready / 工作节点状态未就绪
    WorkerStatusNotReady,
    /// Worker mortgage does not exist / 工作节点抵押不存在
    WorkerMortgageNotExist,
    /// Transfer failed / 转账失败
    TransferFailed,
    /// Worker is being used by user / 工作节点正被用户使用
    WorkerIsUseByUser,
    /// Node does not exist / 节点不存在
    NodeNotExist,
    /// Secret node already exists / 密钥节点已存在
    SecretNodeAlreadyExists,
    /// Set code failed / 设置代码失败
    SetCodeFailed,
    /// Epoch has not expired / 周期未过期
    EpochNotExpired,
    /// Invalid side chain signature / 无效的侧链签名
    InvalidSideChainSignature,
    /// Cannot remove node when it is running / 节点运行时无法删除
    NodeIsRunning,
    /// Caller is not side chain multi-sig address / 调用者不是侧链多重签名地址
    InvalidSideChainCaller,
    /// Region does not exist / 区域不存在
    RegionNotExist,
}
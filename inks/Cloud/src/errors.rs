/// Error types for Cloud contract
/// 云合约错误类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Set code failed / 设置代码失败
    SetCodeFailed,
    /// Must be called by governance contract / 必须由治理合约调用
    MustCallByGovContract,
    /// Worker level is not enough / 工作节点级别不足
    WorkerLevelNotEnough,
    /// Region does not match / 区域不匹配
    RegionNotMatch,
    /// Worker is not online / 工作节点未在线
    WorkerNotOnline,
    /// Caller is not the pod owner / 调用者不是 Pod 所有者
    NotPodOwner,
    /// Pod key does not exist when starting pod / 启动 Pod 时 Pod 密钥不存在
    PodKeyNotExist,
    /// Pod status is invalid / Pod 状态无效
    PodStatusError,
    /// Invalid side chain caller / 无效的侧链调用者
    InvalidSideChainCaller,
    /// Delete pod failed / 删除 Pod 失败
    DelFailed,
    /// Resource not found / 资源未找到
    NotFound,
    /// Pod not found / Pod 未找到
    PodNotFound,
    /// Worker ID not found / 工作节点 ID 未找到
    WorkerIdNotFound,
    /// Worker not found / 工作节点未找到
    WorkerNotFound,
    /// Level price not found / 级别价格未找到
    LevelPriceNotFound,
    /// Asset not found / 资产未找到
    AssetNotFound,
    /// Balance is not enough / 余额不足
    BalanceNotEnough,
    /// Payment failed / 支付失败
    PayFailed,
}
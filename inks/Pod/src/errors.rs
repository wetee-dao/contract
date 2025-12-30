/// Error types for Pod contract
/// Pod 合约错误类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Set code failed / 设置代码失败
    SetCodeFailed,
    /// Must be called by cloud contract / 必须由云合约调用
    MustCallByCloudContract,
    /// Insufficient balance / 余额不足
    InsufficientBalance,
    /// Payment failed / 支付失败
    PayFailed,
    /// Caller is not the owner / 调用者不是所有者
    NotOwner,
    /// Not enough allowance / 授权额度不足
    NotEnoughAllowance,
    /// Not enough balance / 余额不足
    NotEnoughBalance,
    /// Invalid side chain caller / 无效的侧链调用者
    InvalidSideChainCaller,
}
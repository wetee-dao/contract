/// Error types for DAO contract
/// DAO 合约错误类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Token not found / 代币未找到
    TokenNotFound,
    /// Member already exists / 成员已存在
    MemberExisted,
    /// Member does not exist / 成员不存在
    MemberNotExisted,
    /// Member still has balance (cannot leave) / 成员仍有余额（无法离开）
    MemberBalanceNotZero,
    /// Public join is not allowed / 不允许公开加入
    PublicJoinNotAllowed,
    /// Balance is too low / 余额不足
    LowBalance,
    /// Allowance is too low / 授权额度不足
    InsufficientAllowance,
    /// Call execution failed / 调用执行失败
    CallFailed,
    /// Decision deposit is invalid / 决定押金无效
    InvalidDeposit,
    /// Transfer failed / 转账失败
    TransferFailed,
    /// Must be called by governance / 必须由治理调用
    MustCallByGov,
    /// Proposal is not ongoing / 提案未在进行中
    PropNotOngoing,
    /// Proposal is not ended / 提案未结束
    PropNotEnd,
    /// Proposal is invalid / 提案无效
    InvalidProposal,
    /// Proposal status is invalid / 提案状态无效
    InvalidProposalStatus,
    /// Proposal caller is invalid / 提案调用者无效
    InvalidProposalCaller,
    /// Deposit time is invalid / 存入时间无效
    InvalidDepositTime,
    /// Vote time is invalid / 投票时间无效
    InvalidVoteTime,
    /// Vote status is invalid / 投票状态无效
    InvalidVoteStatus,
    /// Vote user is invalid / 投票用户无效
    InvalidVoteUser,
    /// Proposal is in decision period / 提案在决策期内
    ProposalInDecision,
    /// Vote is already unlocked / 投票已解锁
    VoteAlreadyUnlocked,
    /// Vote unlock time is invalid / 投票解锁时间无效
    InvalidVoteUnlockTime,
    /// Proposal is not confirmed / 提案未确认
    ProposalNotConfirmed,
    /// DAO has no track / DAO 没有轨道
    NoTrack,
    /// Max balance exceeds track max balance / 最大余额超过轨道最大余额
    MaxBalanceOverflow,
    /// Transfer is disabled / 转账已禁用
    TransferDisable,
    /// Vote is invalid / 投票无效
    InvalidVote,
    /// Set code failed / 设置代码失败
    SetCodeFailed,
    /// Spend record not found / 支出记录未找到
    SpendNotFound,
    /// Spend already executed / 支出已执行
    SpendAlreadyExecuted,
    /// Spend transfer error / 支出转账错误
    SpendTransferError,
}

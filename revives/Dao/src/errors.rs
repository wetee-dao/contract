use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

/// DAO 合约错误类型，保持与 ink 版本语义一致。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum Error {
    TokenNotFound,
    MemberExisted,
    MemberNotExisted,
    MemberBalanceNotZero,
    PublicJoinNotAllowed,
    LowBalance,
    InsufficientAllowance,
    CallFailed,
    InvalidDeposit,
    TransferFailed,
    MustCallByGov,
    PropNotOngoing,
    PropNotEnd,
    InvalidProposal,
    InvalidProposalStatus,
    InvalidProposalCaller,
    InvalidDepositTime,
    InvalidVoteTime,
    InvalidVoteStatus,
    InvalidVoteUser,
    ProposalInDecision,
    VoteAlreadyUnlocked,
    InvalidVoteUnlockTime,
    ProposalNotConfirmed,
    NoTrack,
    MaxBalanceOverflow,
    TransferDisable,
    InvalidVote,
    SetCodeFailed,
    SpendNotFound,
    SpendAlreadyExecuted,
    SpendTransferError,
}

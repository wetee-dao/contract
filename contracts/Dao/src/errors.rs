#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Returned if member already exists.
    MemberExisted,
    /// Returned if  member does not exist.
    MemberNotExisted,
    /// Returned if member has balance.
    MemberBalanceNotZero,
    /// Returned if member public join not allowed.
    PublicJoinNotAllowed,
    /// Returned if balance is too low.
    LowBalance,
    /// Returned if call failed.
    CallFailed,
    /// Returned if decision deposit is invalid.
    InvalidDeposit,
    /// Returned if transfer failed.
    TransferFailed,
    /// must call by gov account.
    MustCallByGov,
    /// Prposal is not ongoing
    PropNotOngoing,
    /// Prposal is not end
    PropNotEnd,
    /// Returned if proposal is invalid.
    InvalidProposal,
    /// Returned if proposal status is invalid.
    InvalidProposalStatus,
    /// Returned if proposal is not caller.
    InvalidProposalCaller,
    /// Returned if deposit time is invalid.
    InvalidDepositTime,
    /// Returned if vote time is invalid.
    InvalidVoteTime,
    /// Returned if vote status is invalid.
    InvalidVoteStatus,
    /// Returned if vote user is invalid.
    InvalidVoteUser,
    /// Returned if proposal is in decision.
    ProposalInDecision,
    /// Returned if vote is unlocked.
    VoteAlreadyUnlocked,
    /// Returned if vote unlock time is invalid.
    InvalidVoteUnlockTime,
    /// Returned if proposal is not confirmed.
    ProposalNotConfirmed,
    /// Returned if dao has no track.
    NoTrack,
    /// Returned if max balance overflow track max balance.
    MaxBalanceOverflow,
    /// Returned if transfer is disable.
    TransferDisable,
    /// Return if vote is invalid.
    InvalidVote,
    /// Update: set code failed
    SetCodeFailed,
    /// Spend not found
    SpendNotFound,
    /// Spend already exists
    SpendAlreadyExecuted,
    /// Spend transfer error
    SpendTransferError,
}

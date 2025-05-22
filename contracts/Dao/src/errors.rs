#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Returned if balance is too low.
    LowBalance,
    /// Returned if call failed.
    CallFailed,
    /// Returned if decision deposit is invalid.
    InvalidDeposit,
    /// Returned if transfer failed.
    TransferFailed,
    /// Prposal is not ongoing
    PropNotOngoing,
    /// Returned if proposal is invalid.
    InvalidProposal,
    /// Returned if proposal status is invalid.
    InvalidProposalStatus,
    /// Returned if deposit time is invalid.
    InvalidDepositTime,
    /// Returned if vote time is invalid.
    InvalidVoteTime,
    /// Returned if vote status is invalid.
    InvalidVoteStatus,
    /// Returned if vote user is invalid.
    InvalidVoteUser,
    /// Returned if vote is unlocked.
    VoteAlreadyUnlocked,
    /// Returned if vote unlock time is invalid.
    InvalidVoteUnlockTime,
    /// Returned if proposal is not confirmed.
    ProposalNotConfirmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Returned if the call failed.
    CallFailed,
    /// Returned if the proposal status is invalid.
    InvalidProposalStatus,
    /// Returned if the decision deposit is invalid.
    InvalidDeposit,
    /// Returned if the transfer failed.
    TransferFailed
}
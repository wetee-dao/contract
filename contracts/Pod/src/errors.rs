#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Update: set code failed
    SetCodeFailed,
    /// Update: must call by cloud contract
    MustCallByCloudContract,
    /// Insufficient balance
    InsufficientBalance,
    /// Transfer failed
    TransferFailed,
    /// NotOwner
    NotOwner,
    /// not enough allowance
    NotEnoughAllowance,
    /// not enough balance
    NotEnoughBalance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Not enough balance
    NotEnoughBalance,
    /// Caller must be main contract
    MustCallByMainContract,
    /// Worker not exist
    WorkerNotExist,
    /// Worker not owned by caller
    WorkerNotOwnedByCaller,
    /// worker status not ready
    WorkerStatusNotReady,
    /// Worker mortgage not exist
    WorkerMortgageNotExist,
    /// Transfer failed
    TransferFailed,
    /// Worker is use by user
    WorkerIsUseByUser,
    /// Node not exist
    NodeNotExist,
    /// Secret node already exists
    SecretNodeAlreadyExists,
    /// Update: set code failed
    SetCodeFailed,
    /// Epoch not expired
    EpochNotExpired,
    /// InvalidSideChainSignature
    InvalidSideChainSignature,
    /// Remove when Node is running
    NodeIsRunning,
    /// caller is not side chain multi-sig address
    InvalidSideChainCaller,
}
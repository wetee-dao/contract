#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Update: set code failed
    SetCodeFailed,
    /// must call by gov contract
    MustCallByGovContract,
    /// worker level not enough
    WorkerLevelNotEnough,
    /// region not match
    RegionNotMatch,
    /// worker not online
    WorkerNotOnline,
    /// not pod owner
    NotPodOwner,
    /// pod not exist when start pod
    PodKeyNotExist,
    /// pod status error
    PodStatusError,
    /// invalid side chain caller
    InvalidSideChainCaller,
    /// delete pod failed
    DelFailed,
    /// noet found
    NotFound,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// Update: set code failed
    SetCodeFailed,
    /// must call by gov contract
    MustCallByGovContract,
    /// Worker not found
    WorkerNotFound,
    /// worker level not enough
    WorkerLevelNotEnough,
    /// region not match
    RegionNotMatch,
    /// worker not online
    WorkerNotOnline,
}
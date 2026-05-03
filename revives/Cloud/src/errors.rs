//! 云合约错误类型（与 inks Cloud 对齐）

use parity_scale_codec::{Decode, Encode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode)]
pub enum Error {
    SetCodeFailed,
    MustCallByGovContract,
    WorkerLevelNotEnough,
    RegionNotMatch,
    WorkerNotOnline,
    NotPodOwner,
    PodKeyNotExist,
    PodStatusError,
    InvalidSideChainCaller,
    DelFailed,
    NotFound,
    PodNotFound,
    PodCodeNotFound,
    WorkerIdNotFound,
    WorkerNotFound,
    LevelPriceNotFound,
    AssetNotFound,
    BalanceNotEnough,
    PayFailed,
    PodInstantiateFailed,
    ArbitrationNotFound,
    ArbitrationAlreadyResolved,
    WorkerMortgageCheckFailed,
    InvalidFeeRate,
    InsufficientPrepayment,
    PodAlreadySettled,
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;

mod int;
pub use int::*;

#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct ListHelper<T> {
    pub list: Vec<T>,
    pub next_id: T,
}

#[derive(Clone, Default)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct VecIndex<T> {
    pub list: Vec<T>,
}

#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error);
        }
    };
}

#[macro_export]
macro_rules! ok_or_err {
    ($result:expr, $error:expr) => {
        match $result {
            Ok(val) => val,
            Err(_) => return Err($error),
        }
    };
}

#[macro_export]
macro_rules! some_or_err {
    ($option:expr, $error:expr) => {
        match $option {
            Some(val) => val,
            None => return Err($error),
        }
    };
}
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


#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return $error;
        }
    };
}

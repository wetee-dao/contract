#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::vec::Vec, scale::Output};

mod int;
pub use int::*;

#[derive(Clone)]
pub struct CallInput<'a>(pub &'a [u8]);
impl ink::scale::Encode for CallInput<'_> {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(self.0);
    }
}

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
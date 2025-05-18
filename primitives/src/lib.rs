#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::vec::Vec, scale::Output, Address, U256};

pub type CalllId = u32;
pub type Selector = [u8; 4];

#[derive(Clone)]
pub struct CallInput<'a>(pub &'a [u8]);
impl ink::scale::Encode for CallInput<'_> {
    fn encode_to<T: Output + ?Sized>(&self, dest: &mut T) {
        dest.write(self.0);
    }
}

#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Call {
    /// The address of the contract that is call in this proposal.
    pub contract: Option<Address>,
    /// The selector bytes that identifies the function of the contract that should be call.
    pub selector: Selector,
    /// The SCALE encoded parameters that are passed to the call function.
    pub input: Vec<u8>,
    /// The amount of chain balance that is transferred to the Proposalee.
    pub amount: U256,
    /// Gas limit for the execution of the call.
    pub ref_time_limit: u64,
    /// If set to true the transaction will be allowed to re-enter the multisig
    /// contract. Re-entrancy can lead to vulnerabilities. Use at your own risk.
    pub allow_reentry: bool,
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
#![cfg_attr(not(feature = "std"), no_std)]

use ink::{prelude::vec::Vec, Address, U256};

use crate::errors::Error;

#[ink::trait_definition]
pub trait Erc20 {
    #[ink(message)]
    fn name(&self) -> Vec<u8>;

    #[ink(message)]
    fn symbol(&self) -> Vec<u8>;

    #[ink(message)]
    fn decimals(&self) -> u8;

    #[ink(message)]
    fn total_supply(&self) -> U256;

    #[ink(message)]
    fn balance_of(&self, owner: Address) -> U256;

    #[ink(message)]
    fn transfer(&mut self, to: Address, value: U256) -> Result<(),Error>;

    #[ink(message)]
    fn transfer_from(&mut self, from: Address, to: Address, value: U256) -> Result<(),Error>;

    #[ink(message)]
    fn approve(&mut self, spender: Address, value: U256) -> Result<(),Error>;

    #[ink(message)]
    fn allowance(&mut self, owner: Address, spender: Address) -> U256;

    #[ink(message)]
    fn burn(&mut self, amount: U256) -> Result<(), Error>;

    #[ink(message)]
    fn lock_balance_of(&self, owner: Address) -> U256;
}

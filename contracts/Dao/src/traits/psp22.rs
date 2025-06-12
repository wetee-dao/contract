#![cfg_attr(not(feature = "std"), no_std)]

use ink::{prelude::vec::Vec, Address, U256};

use crate::errors::Error;

#[ink::trait_definition]
pub trait PSP22 {
    // other methods
    #[ink(message)]
    fn enable_transfer(&mut self) -> Result<(), Error>;

    #[ink(message)]
    fn can_transfer(&self) -> bool;

    #[ink(message)]
    fn burn(&mut self, amount: U256) -> Result<(), Error>;

    // Token info
    #[ink(message, selector = 0x3d26)]
    fn token_name(&self, asset_id: u32) -> Result<Vec<u8>, Error>;

    #[ink(message, selector = 0x3420)]
    fn token_symbol(&self, asset_id: u32) -> Result<Vec<u8>, Error>;

    #[ink(message, selector = 0x7271)]
    fn token_decimals(&self, asset_id: u32) -> Result<u8, Error>;

    // PSP22 interface queries
    #[ink(message, selector = 0x162d)]
    fn total_supply(&self, asset_id: u32) -> Result<U256, Error>;

    #[ink(message, selector = 0x6568)]
    fn balance_of(&self, asset_id: u32, owner: Address) -> Result<U256, Error>;

    #[ink(message, selector = 0x4d47)]
    fn allowance(&self, asset_id: u32, owner: Address, spender: Address) -> Result<U256, Error>;

    // PSP22 transfer
    #[ink(message, selector = 0xdb20)]
    fn transfer(&mut self, asset_id: u32, to: Address, value: U256) -> Result<(), Error>;

    // PSP22 transfer_from
    #[ink(message, selector = 0x54b3)]
    fn transfer_from(
        &mut self,
        asset_id: u32,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<(), Error>;

    // PSP22 approve
    #[ink(message, selector = 0xb20f)]
    fn approve(&mut self, asset_id: u32, spender: Address, value: U256) -> Result<(), Error>;

    // PSP22 increase_allowance
    #[ink(message, selector = 0x96d6)]
    fn increase_allowance(
        &mut self,
        asset_id: u32,
        spender: Address,
        value: U256,
    ) -> Result<(), Error>;

    // PSP22 decrease_allowance
    #[ink(message, selector = 0xfecb)]
    fn decrease_allowance(
        &mut self,
        asset_id: u32,
        spender: Address,
        value: U256,
    ) -> Result<(), Error>;
}

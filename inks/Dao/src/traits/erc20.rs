use ink::{prelude::vec::Vec, Address, U256};

use crate::errors::Error;

/// ERC20-like token interface
/// 类似 ERC20 的代币接口
/// 
/// This trait defines the standard token operations including transfer, approval, and burning.
/// 该 trait 定义了标准的代币操作，包括转账、授权和销毁。
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

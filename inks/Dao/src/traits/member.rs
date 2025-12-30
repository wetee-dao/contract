use ink::prelude::vec::Vec;
use ink::{Address, U256};

use crate::errors::Error;

/// Member management interface
/// 成员管理接口
/// 
/// This trait defines operations for managing DAO members including joining, leaving, and deletion.
/// 该 trait 定义了管理 DAO 成员的操作，包括加入、离开和删除。
#[ink::trait_definition]
pub trait Member {
    #[ink(message)]
    fn list(&self) -> Vec<Address>;

    #[ink(message)]
    fn get_public_join(&self) -> bool;

    #[ink(message)]
    fn public_join(&mut self) -> Result<(), Error>;

    #[ink(message)]
    fn set_public_join(&mut self, public_join: bool) -> Result<(), Error>;

    #[ink(message)]
    fn join(&mut self, new_user: Address, balance: U256) -> Result<(), Error>;

    #[ink(message)]
    fn levae(&mut self) -> Result<(), Error>;

    #[ink(message)]
    fn levae_with_burn(&mut self) -> Result<(), Error>;

    #[ink(message)]
    fn delete(&mut self, user: Address) -> Result<(), Error>;
}

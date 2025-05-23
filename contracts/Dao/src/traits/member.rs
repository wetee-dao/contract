use ink::prelude::vec::Vec;
use ink::{Address, U256};

use crate::errors::Error;

#[ink::trait_definition]
pub trait Member {
    #[ink(message)]
    fn members(&self) -> Vec<Address>;

    #[ink(message)]
    fn join(&mut self, new_user: Address, balance: U256) -> Result<(), Error>;

    #[ink(message)]
    fn levae(&mut self) -> Result<(), Error>;

    #[ink(message)]
    fn levae_with_burn(&mut self) -> Result<(), Error>;

    #[ink(message)]
    fn delete_member(&mut self, user: Address) -> Result<(), Error>;
}

use ink::{Address, U256};
use ink::prelude::vec::Vec;

#[ink::trait_definition]
pub trait Member {
    #[ink(message)]
    fn members(&self) -> Vec<Address>;

    #[ink(message)]
    fn join(&mut self, new_user: Address, balance: U256);

    #[ink(message)]
    fn levae(&mut self);

    #[ink(message)]
    fn levae_with_burn(&mut self);

    #[ink(message)]
    fn delete_member(&mut self, user: Address);
}

use ink::{Address, U256};

#[ink::trait_definition]
pub trait Erc20 {
    #[ink(message)]
    fn enable_transfer(&mut self);

    #[ink(message)]
    fn transfer(&mut self, to: Address, amount: U256);

    #[ink(message)]
    fn burn(&mut self, amount: U256);

    #[ink(message)]
    fn balance_of(&self, user: Address) -> (U256, U256);
}

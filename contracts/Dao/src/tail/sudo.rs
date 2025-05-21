use ink::prelude::vec::Vec;
use primitives::Call;

use crate::errors::Error;

#[ink::trait_definition]
pub trait SudoTrait {
    #[ink(message)]
    fn sudo(&mut self, call: Call) -> Result<Vec<u8>, Error>;

    #[ink(message)]
    fn remove_sudo(&mut self);
}
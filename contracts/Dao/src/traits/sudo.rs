use ink::prelude::vec::Vec;

use crate::{datas::Call, errors::Error};

#[ink::trait_definition]
pub trait Sudo {
    #[ink(message)]
    fn sudo(&mut self, call: Call) -> Result<Vec<u8>, Error>;

    #[ink(message)]
    fn remove_sudo(&mut self);
}
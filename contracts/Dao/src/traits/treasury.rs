use ink::{Address, U256};

use crate::errors::Error;

#[ink::trait_definition]
pub trait Treasury {
    #[ink(message)]
    fn spend(
        &mut self,
        track_id: u16,
        to: Address,
        assert_id: u32,
        amount: U256,
    ) -> Result<u64, Error>;

    #[ink(message)]
    fn payout(&mut self, spend_index: u64) -> Result<(), Error>;
}

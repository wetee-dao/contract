use ink::{Address, U256};

use crate::errors::Error;

/// Treasury management interface
/// 国库管理接口
/// 
/// This trait defines operations for managing DAO treasury including spending and payouts.
/// 该 trait 定义了管理 DAO 国库的操作，包括支出和支付。
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

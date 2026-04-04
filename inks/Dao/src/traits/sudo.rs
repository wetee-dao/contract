use ink::prelude::vec::Vec;

use crate::{datas::Call, errors::Error};

/// Sudo (superuser) interface
/// Sudo（超级用户）接口
/// 
/// This trait defines operations for sudo account which can execute any function without governance.
/// 该 trait 定义了 sudo 账户的操作，可以在无需治理的情况下执行任何函数。
#[ink::trait_definition]
pub trait Sudo {
    #[ink(message)]
    fn sudo(&mut self, call: Call) -> Result<Vec<u8>, Error>;

    #[ink(message)]
    fn remove_sudo(&mut self) -> Result<(), Error>;
}
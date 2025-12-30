
use ink::prelude::vec::Vec;


#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct TestItem {
    /// id
    pub id: u64,
    /// Pod name
    pub name: Vec<u8>,
}

primitives::define_map!(TestMap, u64, TestItem);

primitives::double_u64_map!(WorkerItems, u64, u64);

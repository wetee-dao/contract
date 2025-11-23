use ink::prelude::vec::Vec;
pub use ink_precompiles::erc20::{
    AssetId,
    erc20,
};

#[derive(Clone, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(Debug, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum AssetInfo {
    Native(Vec<u8>),
    ERC20(Vec<u8>, AssetId),
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod int;
mod mapping;
mod mapping_key;
mod types;

pub use int::*;
pub use mapping::*;
pub use types::*;

#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !$condition {
            return Err($error);
        }
    };
}

#[macro_export]
macro_rules! ok_or_err {
    ($result:expr, $error:expr) => {
        match $result {
            Ok(val) => val,
            Err(_) => return Err($error),
        }
    };
}


pub fn u64_to_u8_32(value: u64) -> [u8; 32] {
    let mut arr = [0u8; 32];
    let bytes = value.to_be_bytes();

    arr[24..32].copy_from_slice(&bytes);
    arr
}

#[cfg(test)]
mod tests;
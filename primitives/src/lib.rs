#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod int;
mod mapping;

pub use int::*;
pub use mapping::*;

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

#[cfg(test)]
mod tests;
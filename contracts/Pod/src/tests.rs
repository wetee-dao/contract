use ink::env::test::default_accounts;

use super::pod::*;

fn _init() -> Pod {
    Pod::new(0,default_accounts().alice)
}

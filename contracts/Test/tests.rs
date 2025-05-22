/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.

/// Imports all the definitions from the outer scope so we can use them here.
use super::test::*;

/// We test if the default constructor does its job.
#[ink::test]
fn default_works() {
    let xxx = TestCase::new();
    assert_eq!(xxx.get(), false);
}

/// We test a simple use case of our contract.
#[ink::test]
fn it_works() {
    let mut xxx = TestCase::new();
    assert_eq!(xxx.get(), false);
    xxx.set();
    assert_eq!(xxx.get(), true);
}

#[ink::test]
fn test_panic() {
    let mut xxx = TestCase::new();
    xxx.test_panic();
}

#[ink::test]
fn test_error() {
    let mut xxx = TestCase::new();
    let err = xxx.test_error();
    println!("{:?}", err)
}

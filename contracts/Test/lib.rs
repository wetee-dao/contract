#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod test {
    use ink::{
        prelude::vec::Vec,
        // scale::Output,
        storage::Mapping,
    };

    type TransactionId = u32;
    
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InvalidProposal
    }

    #[ink(storage)]
    pub struct TestCase {
        ts: Mapping<u32,u64>,
    }

    impl TestCase {
        #[ink(constructor)]
        pub fn new() -> Self {
            let ts = Mapping::default();
            Self {  ts }
        }

        #[ink(message)]
        pub fn get_transaction(&self, i: u32) -> u64 {
            let t = self.ts.get(i).expect("xxxxxxxxxxxxxx");

            t
        }

        #[ink(message)]
        pub fn test_panic(&mut self) {
            // panic!("test x panic");
        }

        #[ink(message)]
        pub fn test_error(&mut self) -> Result<(), Error> {
            Err(Error::InvalidProposal)
        }


        #[ink(message)]
        pub fn set(&mut self) {
            let t = self.ts.insert(1, &2);
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            let t = self.ts.get(1);

            if t.is_some() {
                return true;
            }

            false
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
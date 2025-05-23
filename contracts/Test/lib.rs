#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod test {
    use ink::{
        // scale::Output,
        storage::Mapping,
    };
    use primitives::ensure;
    
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
            ensure!(1<0, Err(Error::InvalidProposal));

            Ok(())
        }


        #[ink(message)]
        pub fn set(&mut self) {
            self.ts.insert(1, &2);
        }

        #[ink(message)]
        pub fn get(&self) -> u64 {
            let t = self.ts.get(1).unwrap_or(0);

            t
        }
    }
}

#[cfg(test)]
mod tests;

// #[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
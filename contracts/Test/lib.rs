#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod test {
    use ink::{
        env::{
        //     call::{
        //         build_call,
        //         ExecutionInput,
        //     },
        //     CallFlags,
        },
        prelude::vec::Vec,
        // scale::Output,
        storage::Mapping,
    };

    type TransactionId = u32;

    #[derive(Clone)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Transaction {
        pub i: u64,
    }

    #[derive(Clone, Default)]
    #[cfg_attr(
        feature = "std",
        derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
    )]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Transactions {
        transactions: Vec<TransactionId>,
        next_id: TransactionId,
    }

    #[ink(storage)]
    pub struct TestCase {
        transaction_list: Transactions,
        ts: Mapping<u32,Transactions>,
    }

    impl TestCase {
        #[ink(constructor)]
        pub fn new() -> Self {
            let transaction_list:Transactions = Default::default();
            let ts = Mapping::default();
            Self { transaction_list, ts }
        }

        #[ink(message)]
        pub fn set(&mut self) {
            // let caller = self::env().caller();
            let t = self.ts.get(1);
            let transaction_list:Transactions = Default::default();

            if t.is_some() {
                let mut rt = t.unwrap();
                rt.transactions.push(2);
                self.ts.insert(1,&rt);
            } else {
                self.ts.insert(1,&transaction_list);
            }          

            // let message: [u8; 49] = [
            //     60, 66, 121, 116, 101, 115, 62, 48, 120, 52, 54, 102, 98, 55, 52, 48, 56,
            //     100, 52, 102, 50, 56, 53, 50, 50, 56, 102, 52, 97, 102, 53, 49, 54, 101,
            //     97, 50, 53, 56, 53, 49, 98, 60, 47, 66, 121, 116, 101, 115, 62,
            // ];

            // // alice
            // let public_key: [u8; 32] = [
            //     212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214,
            //     130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162,
            //     125,
            // ];

            // // alice's signature of the message
            // let signature: [u8; 64] = [
            //     10, 125, 162, 182, 49, 112, 76, 220, 254, 147, 199, 64, 228, 18, 23, 185,
            //     172, 102, 122, 12, 135, 85, 216, 218, 26, 130, 50, 219, 82, 127, 72, 124,
            //     135, 231, 128, 210, 237, 193, 137, 106, 235, 107, 27, 239, 11, 199, 195,
            //     141, 157, 242, 19, 91, 99, 62, 171, 139, 251, 23, 119, 232, 47, 173, 58,
            //     143,
            // ];

            // let _result = ink::env::sr25519_verify(&signature, &message, &public_key);
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            let t = self.ts.get(1);

            if t.is_some() {
                return true;
            }

            false
        }

        #[ink(message)]
        pub fn list(&self) -> Option<Transactions>{
            self.ts.get(1)
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

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
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TestCaseRef::default();

            // When
            let contract = client
                .instantiate("xxx", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<TestCase>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TestCaseRef::new(false);
            let contract = client
                .instantiate("xxx", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<TestCase>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}

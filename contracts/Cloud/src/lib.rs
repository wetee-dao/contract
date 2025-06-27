#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod cloud {
    use crate::{datas::*, errors::Error};
    use ink::{prelude::vec::Vec,H256};
    use primitives::{ensure, ok_or_err};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Cloud {
        /// parent contract ==> Dao contract/user
        parent_contract: Address,

        /// pods
        pods: Pods,

        /// pod of user
        pod_of_user: UserPods,
    }

    impl Cloud {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let mut ins: Cloud = Default::default();
            ins.parent_contract = caller;

            ins
        }

        /// Update contract with gov
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_parent()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        #[ink(message)]
        pub fn pod_len(&mut self) -> Result<u64, Error> {
            // let caller = self.env().caller();

            Ok(self.pods.len())
        }

        #[ink(message)]
        pub fn create_user_pod(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();

            let podid = self.pods.insert(&Pod::default());
            let _id = self.pod_of_user.insert(caller, &podid);

            Ok(())
        }

        #[ink(message)]
        pub fn user_pods(&self) -> Vec<(u32, Pod)> {
            let caller = self.env().caller();
            let ids = self.pod_of_user.list(caller, 1, 10).unwrap();
            let mut list = Vec::new();
            for (id, podid) in ids {
               let pod = self.pods.get(podid);
               if pod.is_some() {
                   list.push((id,pod.unwrap()));
               }
            }

            return list;
        }

        /// Gov call only call from contract
        fn ensure_from_parent(&self) -> Result<(), Error> {
            ensure!(
                self.env().caller() == self.parent_contract,
                Error::MustCallByMainContract
            );

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

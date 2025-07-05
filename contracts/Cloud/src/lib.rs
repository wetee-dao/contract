#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod cloud {
    use crate::{datas::*, errors::Error};
    use ink::{prelude::vec::Vec, H256};
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

        /// create pod
        #[ink(message)]
        pub fn create_user_pod(&mut self) -> Result<(), Error> {
            let caller = self.env().caller();

            let podid = self.pods.insert(&Pod::default());
            self.pod_of_user.insert(caller, &podid);

            Ok(())
        }

        /// all pod length
        #[ink(message)]
        pub fn pod_len(&self) -> Result<u64, Error> {
            Ok(self.pods.len())
        }

        /// List pods
        #[ink(message)]
        pub fn pods(&self, page: u32, size: u32) -> Vec<(u64, Pod)> {
            self.pods.desc_list(page, size)
        }

        /// len of pods owned by user
        #[ink(message)]
        pub fn user_pod_len(&self) -> Result<u64, Error> {
            let caller = self.env().caller();
            Ok(self.pod_of_user.len(caller))
        }

        /// pods of user
        #[ink(message)]
        pub fn user_pods(&self, page: u32, size: u32) -> Vec<(u64, Pod)> {
            let caller = self.env().caller();
            let ids = self.pod_of_user.desc_list(caller, page, size);

            let mut list = Vec::new();
            for (_k2, podid) in ids {
                let pod = self.pods.get(podid);
                if pod.is_some() {
                    list.push((podid, pod.unwrap()));
                }
            }

            return list;
        }

        /// update contract with gov
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_parent()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
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

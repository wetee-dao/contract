#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

pub use self::pod::{Pod, PodRef};

#[ink::contract]
mod pod {
    use crate::errors::Error;
    use ink::{H256, U256};
    use primitives::{ensure, ok_or_err};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Pod {
        /// parent contract ==> Dao contract/user
        cloud_contract: Address,
        /// pod ID
        pod_id: u64,
        /// owner
        owner: Address,
        /// balance of pod
        balance: U256,
        /// balance allow to pay for computing power
        allowance: Option<U256>,
    }

    impl Pod {
        #[ink(constructor)]
        pub fn new(id: u64, owner: Address) -> Self {
            let caller = Self::env().caller();
            let mut ins: Pod = Default::default();
            let transferred = Self::env().transferred_value();

            ins.cloud_contract = caller;
            ins.pod_id = id;
            ins.balance = transferred;
            ins.owner = owner;
            ins.allowance = None;

            ins
        }

        /// Create pod
        #[ink(message)]
        pub fn cloud(&mut self) -> Address {
            self.cloud_contract
        }

        #[ink(message)]
        pub fn approve(&mut self, value: Option<U256>) -> Result<(), Error> {
            let caller = self.env().caller();

            ensure!(self.owner == caller, Error::NotOwner);

            self.allowance = value;
            Ok(())
        }

        #[ink(message)]
        pub fn pay_for_woker(&mut self, worker: Address, amount: U256) -> Result<(), Error> {
            self.ensure_from_cloud()?;

            let allowance = self.allowance;
            if allowance.is_some() {
                ensure!(allowance.unwrap() >= amount, Error::NotEnoughAllowance);
                self.allowance =  Some(allowance.unwrap() - amount);
            }
            
            ensure!(self.balance >= amount, Error::NotEnoughBalance);
            ok_or_err!(self.env().transfer(worker, amount), Error::TransferFailed);

            self.balance -= amount;
            Ok(())
        }

        /// Charge balance
        #[ink(message, payable)]
        pub fn charge(&mut self) -> Result<(), Error> {
            let transferred = Self::env().transferred_value();
            self.balance += transferred;
            
            Ok(())
        }

        /// Withdraw balance
        #[ink(message)]
        pub fn withdraw(&mut self, amount: U256) -> Result<(), Error> {
            let caller = self.env().caller();

            ensure!(self.owner == caller, Error::NotOwner);
            ensure!(self.balance >= amount, Error::InsufficientBalance);

            ok_or_err!(
                self.env().transfer(self.cloud_contract, amount),
                Error::TransferFailed
            );

            Ok(())
        }

        /// Update contract with gov
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_cloud()?;
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        /// Gov call only call from contract
        fn ensure_from_cloud(&self) -> Result<(), Error> {
            ensure!(
                self.env().caller() == self.cloud_contract,
                Error::MustCallByCloudContract
            );

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

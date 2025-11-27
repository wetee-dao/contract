#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod pod {
    use crate::errors::Error;
    use ink::{H256, U256};
    use ink_precompiles::erc20::{erc20, Erc20};
    use primitives::{ensure, ok_or_err, AssetInfo};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Pod {
        /// parent contract ==> Dao contract/user
        cloud_contract: Address,
        /// sidechain Multi-sig account
        side_chain_multi_key: Address,
        /// pod ID
        pod_id: u64,
        /// owner
        owner: Address,
    }

    impl Pod {
        /// Constructor
        #[ink(constructor)]
        pub fn new(id: u64, owner: Address, side_chain_multi_key: Address) -> Self {
            let caller = Self::env().caller();
            let mut ins: Pod = Default::default();

            ins.cloud_contract = caller;
            ins.side_chain_multi_key = side_chain_multi_key;
            ins.pod_id = id;
            ins.owner = owner;

            ins
        }

        /// Charge
        #[ink(message, default, payable)]
        pub fn charge(&mut self) {
            let _transferred = self.env().transferred_value();
        }


        #[ink(message)]
        pub fn account_id(&self) -> AccountId {
            self.env().account_id()
        }
        
        /// Get pod ID
        #[ink(message)]
        pub fn id(&self) -> u64 {
            self.pod_id
        }

        /// Create pod
        #[ink(message)]
        pub fn cloud(&mut self) -> Address {
            self.cloud_contract
        }

        /// Get owner
        #[ink(message)]
        pub fn owner(&self) -> Address {
            self.owner
        }

        /// pay for cloud
        #[ink(message)]
        pub fn pay_for_woker(
            &mut self,
            to: Address,
            asset: AssetInfo,
            amount: U256,
        ) -> Result<(), Error> {
            self.ensure_from_cloud()?;

            match asset {
                AssetInfo::Native(_) => {
                    // check pod balance
                    ensure!(self.env().balance() >= amount, Error::NotEnoughBalance);

                    // transfer to worker
                    match self.env().transfer(to, amount) {
                        Ok(_) => Ok(()),
                        Err(_e) => Err(Error::PayFailed),
                    }
                }
                AssetInfo::ERC20(_, asset_id) => {
                    let mut asset = erc20(TRUST_BACKED_ASSETS_PRECOMPILE_INDEX, asset_id);

                    // check pod balance
                    ensure!(
                        asset.balanceOf(self.env().address()) >= amount,
                        Error::NotEnoughBalance
                    );

                    // transfer to worker
                    let ok = asset.transfer(to, amount);
                    if !ok {
                        return Err(Error::PayFailed);
                    }
                    Ok(())
                }
            }
        }

        // /// Call contract
        // #[ink(message)]
        // pub fn call(&mut self, method: &str, data: Vec<u8>) -> Result<(), Error> {
        //     let caller = self.env().caller();
        //     Ok(())
        // }

        // /// Call back to contract
        // #[ink(message)]
        // pub fn call_back(&mut self) -> Result<(), Error> {
        //     self.ensure_from_side_chain()?;

        //     Ok(())
        // }

        /// Withdraw balance
        #[ink(message)]
        pub fn withdraw(
            &mut self,
            asset: AssetInfo,
            to: Address,
            amount: U256,
        ) -> Result<(), Error> {
            let caller = self.env().caller();

            ensure!(self.owner == caller, Error::NotOwner);

            match asset {
                AssetInfo::Native(_) => {
                    // check pod balance
                    ensure!(self.env().balance() >= amount, Error::InsufficientBalance);

                    // transfer to cloud contract
                    match self.env().transfer(to, amount) {
                        Ok(_) => Ok(()),
                        Err(_e) => Err(Error::PayFailed),
                    }
                }
                AssetInfo::ERC20(_, asset_id) => {
                    let mut asset = erc20(TRUST_BACKED_ASSETS_PRECOMPILE_INDEX, asset_id);

                    // check pod balance
                    ensure!(
                        asset.balanceOf(self.env().address()) >= amount,
                        Error::InsufficientBalance
                    );

                    // transfer to worker
                    let ok = asset.transfer(to, amount);
                    if !ok {
                        return Err(Error::PayFailed);
                    }
                    Ok(())
                }
            }
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

        /// ensure the caller is from side chain
        fn ensure_from_side_chain(&self) -> Result<(), Error> {
            let caller = self.env().caller();
            ensure!(
                caller == self.side_chain_multi_key,
                Error::InvalidSideChainCaller
            );

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

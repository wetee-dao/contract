#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod pod {
    use crate::errors::Error;
    use ink::{H256, U256};
    use ink_precompiles::erc20::{erc20, Erc20};
    use primitives::{ensure, ok_or_err, AssetInfo};

    /// Pod Contract Storage
    /// Pod 合约存储结构
    /// 
    /// This contract represents a single pod (compute instance) in the cloud.
    /// It manages the pod's balance, payments to workers, and owner permissions.
    /// 该合约代表云中的单个 Pod（计算实例）。
    /// 它管理 Pod 的余额、向工作节点的支付和所有者权限。
    #[ink(storage)]
    #[derive(Default)]
    pub struct Pod {
        /// Cloud contract address (parent contract) / 云合约地址（父合约）
        cloud_contract: Address,
        /// Sidechain multi-signature account / 侧链多重签名账户
        side_chain_multi_key: Address,
        /// Pod ID / Pod ID
        pod_id: u64,
        /// Pod owner address / Pod 所有者地址
        owner: Address,
    }

    impl Pod {
        /// Create a new Pod contract
        /// 创建新的 Pod 合约
        /// 
        /// # Arguments
        /// * `id` - Pod ID / Pod ID
        /// * `owner` - Pod owner address / Pod 所有者地址
        /// * `side_chain_multi_key` - Sidechain multi-signature account / 侧链多重签名账户
        /// 
        /// # Returns
        /// * `Self` - New Pod contract instance / 新的 Pod 合约实例
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

        /// Charge native tokens to pod (payable)
        /// 向 Pod 充值原生代币（可支付）
        /// 
        /// The transferred value is added to the pod's balance.
        /// 转账的金额将添加到 Pod 的余额中。
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

        /// Pay worker for computing resources (cloud contract only)
        /// 向工作节点支付计算资源费用（仅云合约）
        /// 
        /// # Arguments
        /// * `to` - Worker address to pay / 要支付的工作节点地址
        /// * `asset` - Asset type (Native or ERC20) / 资产类型（原生或 ERC20）
        /// * `amount` - Amount to pay / 支付金额
        /// 
        /// # Returns
        /// * `Result<(), Error>` - Ok if successful / 成功返回 Ok
        /// 
        /// # Errors
        /// * `MustCallByCloudContract` - Must be called by cloud contract / 必须由云合约调用
        /// * `NotEnoughBalance` - Insufficient pod balance / Pod 余额不足
        /// * `NotEnoughAllowance` - Insufficient allowance (if set) / 授权额度不足（如果已设置）
        /// * `PayFailed` - Payment transfer failed / 支付转账失败
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

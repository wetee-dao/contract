#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod cloud {
    use crate::{datas::*, errors::Error};
    use ink::{prelude::vec::Vec, H256};
    use pod::PodRef;
    use primitives::{ensure, ok_or_err, u64_to_u8_32};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Cloud {
        /// parent contract ==> Dao contract/user
        gov_contract: Address,
        /// pod contract code hash
        pod_contract_code_hash: H256,

        /// pods
        pods: Pods,
        /// pod
        pod_status: u8,
        /// pod of user
        pod_of_user: UserPods,
        /// worker of user
        pod_of_worker: WorkerPods,

        /// pod's containers
        containers: Containers,
    }

    impl Cloud {
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            let mut ins: Cloud = Default::default();
            ins.gov_contract = caller;

            ins
        }

        /// Create pod
        #[ink(message, payable)]
        pub fn create_user_pod(
            &mut self,
            name: Vec<u8>,
            pod_type: PodType, // Type of pod,Different pods will be called to different clusters.
            tee_type: TEEType, // tee version
            containers: Vec<Container>, // containers
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let pay_value = self.env().transferred_value();

            // init new pod contract
            let pod_id = self.pods.next_id();
            let contract = PodRef::new(pod_id,caller)
                .endowment(pay_value)
                .code_hash(self.pod_contract_code_hash)
                .salt_bytes(Some(u64_to_u8_32(pod_id)))
                .instantiate();

            let now = self.env().block_number();

            // save pod base config
            self.pods.insert(&Pod {
                name: name,
                owner: caller,
                contract: contract,
                ptype: pod_type,
                start_block: now,
                tee_type: tee_type,
            });
            self.pod_of_user.insert(caller, &pod_id);

            // save pod containers
            for i in 0..containers.len() {
                self.containers.insert(pod_id, &containers[i]);
            }

            Ok(())
        }

        /// All pod length
        #[ink(message)]
        pub fn pod_len(&self) -> u64 {
            self.pods.len()
        }

        /// List pods
        #[ink(message)]
        pub fn pods(&self, page: u64, size: u64) -> Vec<(u64, Pod, Vec<Container>)> {
            let list = self.pods.desc_list(page, size);

            let mut pods = Vec::new();
            for (k, v) in list.iter() {
                let containers = self.containers.desc_list(*k, 1, 20);
                pods.push((
                    *k,
                    v.clone(),
                    containers.iter().map(|(_, v)| v.clone()).collect(),
                ));
            }

            pods
        }

        /// Len of pods owned by user
        #[ink(message)]
        pub fn user_pod_len(&self) -> u32 {
            let caller = self.env().caller();
            self.pod_of_user.len(caller)
        }

        /// Pods of user
        #[ink(message)]
        pub fn user_pods(&self, page: u32, size: u32) -> Vec<(u64, Pod, Vec<Container>)> {
            let caller = self.env().caller();
            let ids = self.pod_of_user.desc_list(caller, page, size);

            let mut pods = Vec::new();
            for (_k2, podid) in ids {
                let pod = self.pods.get(podid);
                if pod.is_some() {
                    let containers = self.containers.desc_list(podid, 1, 20);
                    pods.push((
                        podid,
                        pod.unwrap(),
                        containers.iter().map(|(_, v)| v.clone()).collect(),
                    ));
                }
            }

            return pods;
        }

        /// Update contract with gov
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_gov()?;
            
            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        /// Gov call only call from contract
        fn ensure_from_gov(&self) -> Result<(), Error> {
            ensure!(
                self.env().caller() == self.gov_contract,
                Error::MustCallByMainContract
            );

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

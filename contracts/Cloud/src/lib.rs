#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod cloud {
    use crate::{datas::*, errors::Error};
    use ink::{env::call::FromAddr, prelude::vec::Vec, storage::Mapping, H256};
    use pod::PodRef;
    use primitives::{ensure, ok_or_err, u64_to_u8_32};
    use subnet::SubnetRef;

    #[ink(storage)]
    pub struct Cloud {
        /// parent contract ==> Dao contract/user
        gov_contract: Address,
        /// subnet_contract
        subnet: SubnetRef,
        /// pod contract code hash
        pod_contract_code_hash: H256,

        /// pods
        pods: Pods,
        /// pod
        pod_status: Mapping<u64, u8>,
        /// pod of user
        pod_of_user: UserPods,
        /// worker of user
        pod_of_worker: WorkerPods,

        /// pod's containers
        containers: Containers,
    }

    impl Cloud {
        #[ink(constructor)]
        pub fn new(subnet_addr: Address, pod_contract_code_hash: H256) -> Self {
            let caller = Self::env().caller();

            // init subnet contract
            let subnet = SubnetRef::from_addr(subnet_addr);

            // init cloud contract
            let ins = Cloud {
                gov_contract: caller,
                subnet,
                pods: Default::default(),
                pod_status: Default::default(),
                pod_of_user: Default::default(),
                pod_of_worker: Default::default(),
                containers: Default::default(),
                pod_contract_code_hash: pod_contract_code_hash,
            };

            ins
        }

        /// set new pod code hash
        #[ink(message)]
        pub fn set_pod_contract(&mut self, pod_contract: H256) -> Result<(), Error> {
            self.ensure_from_gov()?;
            self.pod_contract_code_hash = pod_contract;

            Ok(())
        }

        #[ink(message)]
        pub fn subnet_address(&self) -> Address { 
            self.subnet.as_ref().clone()
        }

        /// Create pod
        #[ink(message, payable)]
        pub fn create_user_pod(
            &mut self,
            name: Vec<u8>,
            pod_type: PodType,
            tee_type: TEEType, 
            containers: Vec<Container>,
            region_id: u32,
            level: u8,
            worker_id: u64,
        ) -> Result<(), Error> {
            let caller = self.env().caller();

            // check worker status
            let worker = self.subnet.worker(worker_id).ok_or(Error::WorkerNotFound)?;
            ensure!(worker.level >= level, Error::WorkerLevelNotEnough);
            ensure!(worker.region_id == region_id, Error::RegionNotMatch);
            // ensure!(worker.status == 1, Error::WorkerNotOnline);

            let pay_value = self.env().transferred_value();

            // init new pod contract
            let pod_id = self.pods.next_id();
            let contract = PodRef::new(pod_id, caller)
                .endowment(pay_value)
                .code_hash(self.pod_contract_code_hash)
                .salt_bytes(Some(u64_to_u8_32(pod_id)))
                .instantiate();

            // save pod base config
            self.pods.insert(&Pod {
                name: name,
                owner: caller,
                contract: contract,
                ptype: pod_type,
                start_block: self.env().block_number(),
                tee_type: tee_type,
            });
            self.pod_of_user.insert(caller, &pod_id);
            self.pod_of_worker.insert(worker_id, &pod_id);

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
                Error::MustCallByGovContract
            );

            Ok(())
        }
    }
}

// #[cfg(all(test, feature = "e2e-tests"))]
#[cfg(test)]
mod e2e_tests;

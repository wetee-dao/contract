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
        /// pod envs
        pod_envs: Mapping<u64, Vec<Env>>,
        /// pod last block number
        pod_version: Mapping<u64, BlockNumber>,
        /// pod status 0=>created  1=>deoloying 2=>error  3=>stop
        pod_status: Mapping<u64, u8>,
        /// last mint block number of pod
        last_mint_block: Mapping<u64, BlockNumber>,
        /// tee mint interval
        mint_interval: BlockNumber,
        /// tee mint hash of pod
        pod_report: Mapping<u64, H256>,
        /// pod deployed ed25519 key,Generate within the TEE for each deployment
        pod_key: Mapping<u64, AccountId>,
        /// pod of user
        pod_of_user: UserPods,
        /// pods of worker
        pods_of_worker: WorkerPods,
        /// pod id to worker
        worker_of_pod: Mapping<u64, u64>,

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
                pod_envs: Default::default(),
                pod_version: Default::default(),
                pod_status: Default::default(),
                pod_of_user: Default::default(),
                pods_of_worker: Default::default(),
                worker_of_pod: Default::default(),
                containers: Default::default(),
                last_mint_block: Default::default(),
                pod_report: Default::default(),
                pod_key: Default::default(),
                mint_interval: 14400u32.into(),
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
        pub fn set_mint_interval(&mut self, t: BlockNumber) -> Result<(), Error> {
            self.mint_interval = t;
            Ok(())
        }

        #[ink(message)]
        pub fn mint_interval(&self) -> BlockNumber {
            self.mint_interval
        }

        #[ink(message)]
        pub fn subnet_address(&self) -> Address {
            self.subnet.as_ref().clone()
        }

        /// Create pod
        #[ink(message, payable)]
        pub fn create_pod(
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
            let now = self.env().block_number();
            self.pods.insert(&Pod {
                name: name,
                owner: caller,
                contract: contract,
                ptype: pod_type,
                start_block: now,
                tee_type: tee_type,
            });
            self.pod_of_user.insert(caller, &pod_id);
            self.pods_of_worker.insert(worker_id, &pod_id);
            self.worker_of_pod.insert(pod_id, &worker_id);
            self.last_mint_block.insert(pod_id, &now);

            // save pod containers
            for i in 0..containers.len() {
                self.containers.insert(pod_id, &containers[i]);
            }

            Ok(())
        }

        /// start pod
        #[ink(message)]
        pub fn start_pod(
            &mut self,
            pod_id: u64,
            pod_key: Option<AccountId>,
            report: H256,
        ) -> Result<(), Error> {
            self.ensure_from_side_chain()?;

            let status = self.pod_status.get(pod_id).unwrap_or_default();
            if status != 0 && status != 1 {
                return Err(Error::PodStatusError);
            }

            self.pod_report.insert(pod_id, &report);
            let now = self.env().block_number();
            let last_mint = self.last_mint_block.get(pod_id).unwrap_or_default();

            // check pod key
            if status == 0 {
                ensure!(pod_key.is_some(), Error::PodKeyNotExist);
                self.pod_status.insert(pod_id, &1);
                self.last_mint_block.insert(pod_id, &now);
                return Ok(());
            }

            // check mint
            if now < last_mint + self.mint_interval {
                return Ok(());
            }

            // mint pod
            self.last_mint_block
                .insert(pod_id, &(last_mint + self.mint_interval));

            // pay for pod to worker

            Ok(())
        }

        /// stop pod
        #[ink(message)]
        pub fn stop_pod(&mut self, pod_id: u64) -> Result<(), Error> {
            let caller = self.env().caller();

            // check pod owner
            let pod = self.pods.get(pod_id).ok_or(Error::PodNotFound)?;
            ensure!(pod.owner == caller, Error::NotPodOwner);

            // stop pod
            self.pod_status.insert(pod_id, &3);
            let worker_id = self.worker_of_pod.get(pod_id).ok_or(Error::PodNotFound)?;

            // delete pod in worker
            let all_pod = self.pods_of_worker.list_all(worker_id);
            let mut ok: bool = false;
            if let Some(&index) = all_pod.iter().find(|&&x| x.1 == pod_id) {
                ok = self
                    .pods_of_worker
                    .delete_replace_by_last_key(worker_id, index.0);
            }

            if !ok {
                return Err(Error::DelFailed);
            }

            // pay for pod to worker

            Ok(())
        }

        /// restart pod
        #[ink(message)]
        pub fn restart_pod(&mut self, pod_id: u64) -> Result<(), Error> {
            let caller = self.env().caller();

            // check pod owner
            let pod = self.pods.get(pod_id).ok_or(Error::PodNotFound)?;
            ensure!(pod.owner == caller, Error::NotPodOwner);

            // check pod status
            let status = self.pod_status.get(pod_id).unwrap_or_default();
            if status != 1 && status != 3 {
                return Err(Error::PodStatusError);
            }

            // if status == 3, restart pod
            if status == 3 {
                let worker_id = self.worker_of_pod.get(pod_id).ok_or(Error::PodNotFound)?;
                // restart pod
                self.pod_status.insert(pod_id, &0);
                self.pods_of_worker.insert(worker_id, &pod_id);
                let now = self.env().block_number();
                self.last_mint_block.insert(pod_id, &now);
            }

            // update pod version
            self.pod_version.insert(pod_id, &self.env().block_number());
            Ok(())
        }

        // add update remove container
        #[ink(message)]
        pub fn edit_container(
            &mut self,
            pod_id: u64,
            containers: Vec<ContainerInput>,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let pod = self.pods.get(pod_id).ok_or(Error::PodNotFound)?;
            ensure!(pod.owner == caller, Error::NotPodOwner);

            for container in containers.iter() {
                match container.etype {
                    EditType::INSERT => {
                        self.add_container(pod_id, container.container.clone())?;
                    }
                    EditType::UPDATE(container_id) => {
                        self.update_container(pod_id, container_id, container.container.clone())?;
                    }
                    EditType::REMOVE(container_id) => {
                        self.del_container(pod_id, container_id)?;
                    }
                }
            }

            self.pod_version.insert(pod_id, &self.env().block_number());
            Ok(())
        }

        /// All pod length
        #[ink(message)]
        pub fn pod_len(&self) -> u64 {
            self.pods.len()
        }

        /// List pods
        #[ink(message)]
        pub fn pods(&self, page: u64, size: u64) -> Vec<(u64, Pod, Vec<(u64, Container)>)> {
            let list = self.pods.desc_list(page, size);

            let mut pods = Vec::new();
            for (k, v) in list.iter() {
                let containers = self.containers.desc_list(*k, 1, 20);
                pods.push((*k, v.clone(), containers));
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
        pub fn user_pods(&self, page: u32, size: u32) -> Vec<(u64, Pod, Vec<(u64, Container)>)> {
            let caller = self.env().caller();
            let ids = self.pod_of_user.desc_list(caller, page, size);

            let mut pods = Vec::new();
            for (_k2, pod_id) in ids {
                let pod = self.pods.get(pod_id);
                if pod.is_some() {
                    let containers = self.containers.desc_list(pod_id, 1, 20);
                    pods.push((pod_id, pod.unwrap(), containers));
                }
            }

            pods
        }

        /// Pods version of worker
        #[ink(message)]
        pub fn worker_pods_version(
            &self,
            worker_id: u64,
        ) -> Vec<(u64, BlockNumber, BlockNumber, u8)> {
            let ids = self.pods_of_worker.desc_list(worker_id, 1, 10000);

            let mut pods = Vec::new();
            for (_k2, pod_id) in ids {
                let version = self.pod_version.get(pod_id).unwrap_or_default();
                let status = self.pod_status.get(pod_id).unwrap_or_default();
                let last_mint = self.last_mint_block.get(pod_id).unwrap_or_default();
                pods.push((pod_id, version, last_mint, status));
            }

            pods
        }

        /// Pods of worker
        #[ink(message)]
        pub fn worker_pods(
            &self,
            worker_id: u64,
            page: u64,
            size: u64,
        ) -> Vec<(u64, Pod, Vec<(u64, Container)>)> {
            let ids = self.pods_of_worker.desc_list(worker_id, page, size);
            let mut pods = Vec::new();
            for (_k2, pod_id) in ids {
                let pod = self.pods.get(pod_id);
                if pod.is_some() {
                    let containers = self.containers.desc_list(pod_id, 1, 20);
                    pods.push((pod_id, pod.unwrap(), containers))
                }
            }

            pods
        }

        /// Get pod info
        #[ink(message)]
        pub fn pod(&self, pod_id: u64) -> Option<(Pod, Vec<(u64, Container)>, BlockNumber, u8)> {
            let pod_wrap = self.pods.get(pod_id);
            if pod_wrap.is_none() {
                return None;
            }
            let pod = pod_wrap.unwrap();
            let containers = self.containers.desc_list(pod_id, 1, 20);
            let version = self.pod_version.get(pod_id).unwrap_or_default();
            let status = self.pod_status.get(pod_id).unwrap_or_default();

            Some((pod, containers, version, status))
        }

        /// Get pods info
        #[ink(message)]
        pub fn pods_by_ids(
            &self,
            pod_ids: Vec<u64>,
        ) -> Vec<(
            u64,
            Pod,
            Vec<(u64, Container)>,
            BlockNumber,
            BlockNumber,
            u8,
        )> {
            let mut pods = Vec::new();
            for pod_id in pod_ids {
                let pod_wrap = self.pods.get(pod_id);
                if pod_wrap.is_none() {
                    continue;
                }
                let pod = pod_wrap.unwrap();
                let containers = self.containers.desc_list(pod_id, 1, 20);
                let version = self.pod_version.get(pod_id).unwrap_or_default();
                let status = self.pod_status.get(pod_id).unwrap_or_default();
                let last_mint = self.last_mint_block.get(pod_id).unwrap_or_default();

                pods.push((pod_id, pod, containers, version, last_mint, status));
            }

            pods
        }

        /// Len of pods by worker
        #[ink(message)]
        pub fn worker_pod_len(&self, worker_id: u64) -> u64 {
            self.pods_of_worker.next_id(worker_id)
        }

        /// Update contract with gov
        #[ink(message)]
        pub fn set_code(&mut self, code_hash: H256) -> Result<(), Error> {
            self.ensure_from_gov()?;

            ok_or_err!(self.env().set_code_hash(&code_hash), Error::SetCodeFailed);

            Ok(())
        }

        pub fn add_container(&mut self, pod_id: u64, container: Container) -> Result<(), Error> {
            let caller = self.env().caller();
            let pod = self.pods.get(pod_id).ok_or(Error::PodNotFound)?;
            ensure!(pod.owner == caller, Error::NotPodOwner);

            self.containers.insert(pod_id, &container);
            Ok(())
        }

        pub fn update_container(
            &mut self,
            pod_id: u64,
            container_id: u64,
            container: Container,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let pod = self.pods.get(pod_id).ok_or(Error::PodNotFound)?;
            ensure!(pod.owner == caller, Error::NotPodOwner);

            self.containers.update(pod_id, container_id, &container);
            Ok(())
        }

        pub fn del_container(&mut self, pod_id: u64, container_id: u64) -> Result<bool, Error> {
            let caller = self.env().caller();
            let pod = self.pods.get(pod_id).ok_or(Error::PodNotFound)?;
            ensure!(pod.owner == caller, Error::NotPodOwner);

            let ok = self
                .containers
                .delete_replace_by_last_key(pod_id, container_id);

            if !ok {
                return Err(Error::DelFailed);
            }
            Ok(ok)
        }

        /// ensure the caller is from side chain
        fn ensure_from_side_chain(&self) -> Result<(), Error> {
            let caller = self.env().caller();
            ensure!(
                caller == self.subnet.side_chain_key(),
                Error::InvalidSideChainCaller
            );

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

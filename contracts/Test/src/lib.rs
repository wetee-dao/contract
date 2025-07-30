#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod datas;
mod errors;

#[ink::contract]
mod test {
    use core::u64;

    use crate::{datas::*, errors::Error};
    use ink::{prelude::vec::Vec, storage::Mapping};

    #[ink(storage)]
    #[derive(Default)]
    pub struct Test {
        tests: TestMap,
        /// pods of worker
        tests_of_worker: WorkerItems,
        /// pod id to worker
        worker_of_test: Mapping<u64, u64>,
    }

    impl Test {
        #[ink(constructor)]
        pub fn new() -> Self {
            let ins: Test = Default::default();

            ins
        }

        #[ink(message)]
        pub fn add(&mut self,  worker_id: u64, value: TestItem) -> Result<(), Error> {
            let id = self.tests.insert(&value);

            self.tests_of_worker.insert(worker_id, &id);
            self.worker_of_test.insert(id, &worker_id);

            Ok(())
        }

        #[ink(message)]
        pub fn list(&self, t: u64, start: Option<u64>, size: u64) -> Vec<(u64, TestItem)> {
            let ids = self.tests_of_worker.desc_list(t, start, size);
            let mut lists = Vec::new();
            for id in ids.iter(){
                let t = self.tests.get(id.1).unwrap();
                lists.push((id.1,t));
            }

            lists
        }

        #[ink(message)]
        pub fn del(&mut self, id: u64) -> Result<(), Error> {
            let worker_id = self.worker_of_test.get(id).unwrap();

            let all = self.tests_of_worker.list_all(worker_id);
            let mut ok: bool = false;
            if let Some(&index) = all.iter().find(|&&x| x.1 == id) {
                ok = self
                    .tests_of_worker
                    .delete_by_key(worker_id, index.0);
            }

            if !ok {
                return Err(Error::DelFailed);
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests;

// #[cfg(all(test, feature = "e2e-tests"))]
#[cfg(test)]
mod e2e_tests;

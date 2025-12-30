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

        /// Charge
        #[ink(message, default, payable)]
        pub fn charge(&mut self) {
            let _transferred = self.env().transferred_value();
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
                if let Some(test_item) = self.tests.get(id.1) {
                    lists.push((id.1, test_item));
                }
            }

            lists
        }

        #[ink(message)]
        pub fn del(&mut self, id: u64) -> Result<(), Error> {
            let worker_id = match self.worker_of_test.get(id) {
                Some(wid) => wid,
                None => return Err(Error::DelFailed),
            };

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

        // ========== 测试 define_map! 宏的方法 ==========
        
        /// 测试 TestMap::len
        #[ink(message)]
        pub fn test_map_len(&self) -> u64 {
            self.tests.len()
        }

        /// 测试 TestMap::next_id
        #[ink(message)]
        pub fn test_map_next_id(&mut self) -> u64 {
            self.tests.next_id()
        }

        /// 测试 TestMap::insert
        #[ink(message)]
        pub fn test_map_insert(&mut self, value: TestItem) -> u64 {
            self.tests.insert(&value)
        }

        /// 测试 TestMap::get
        #[ink(message)]
        pub fn test_map_get(&self, key: u64) -> Option<TestItem> {
            self.tests.get(key)
        }

        /// 测试 TestMap::contains
        #[ink(message)]
        pub fn test_map_contains(&self, key: u64) -> bool {
            self.tests.contains(&key)
        }

        /// 测试 TestMap::update
        #[ink(message)]
        pub fn test_map_update(&mut self, key: u64, value: TestItem) -> Option<u32> {
            self.tests.update(key, &value)
        }

        /// 测试 TestMap::list
        #[ink(message)]
        pub fn test_map_list(&self, start_key: u64, size: u64) -> Vec<(u64, TestItem)> {
            self.tests.list(start_key, size)
        }

        /// 测试 TestMap::desc_list
        #[ink(message)]
        pub fn test_map_desc_list(&self, start_key: Option<u64>, size: u64) -> Vec<(u64, TestItem)> {
            self.tests.desc_list(start_key, size)
        }

        // ========== 测试 double_u64_map! 宏的方法 ==========

        /// 测试 WorkerItems::next_id
        #[ink(message)]
        pub fn test_double_map_next_id(&self, k: u64) -> u64 {
            self.tests_of_worker.next_id(k)
        }

        /// 测试 WorkerItems::len
        #[ink(message)]
        pub fn test_double_map_len(&self, k: u64) -> u64 {
            self.tests_of_worker.len(k)
        }

        /// 测试 WorkerItems::insert
        #[ink(message)]
        pub fn test_double_map_insert(&mut self, k1: u64, v: u64) -> u64 {
            self.tests_of_worker.insert(k1, &v)
        }

        /// 测试 WorkerItems::get
        #[ink(message)]
        pub fn test_double_map_get(&self, k1: u64, k2: u64) -> Option<u64> {
            self.tests_of_worker.get(k1, k2)
        }

        /// 测试 WorkerItems::update
        #[ink(message)]
        pub fn test_double_map_update(&mut self, k1: u64, k2: u64, v: u64) -> Option<u32> {
            self.tests_of_worker.update(k1, k2, &v)
        }

        /// 测试 WorkerItems::list
        #[ink(message)]
        pub fn test_double_map_list(&self, k1: u64, start_key: u64, size: u64) -> Vec<(u64, u64)> {
            self.tests_of_worker.list(k1, start_key, size)
        }

        /// 测试 WorkerItems::desc_list
        #[ink(message)]
        pub fn test_double_map_desc_list(&self, k1: u64, start_key: Option<u64>, size: u64) -> Vec<(u64, u64)> {
            self.tests_of_worker.desc_list(k1, start_key, size)
        }

        /// 测试 WorkerItems::list_all
        #[ink(message)]
        pub fn test_double_map_list_all(&self, k1: u64) -> Vec<(u64, u64)> {
            self.tests_of_worker.list_all(k1)
        }

        /// 测试 WorkerItems::delete_by_key
        #[ink(message)]
        pub fn test_double_map_delete_by_key(&mut self, k1: u64, k2: u64) -> bool {
            self.tests_of_worker.delete_by_key(k1, k2)
        }
    }
}

#[cfg(test)]
mod tests;

// #[cfg(all(test, feature = "e2e-tests"))]
#[cfg(test)]
mod e2e_tests;

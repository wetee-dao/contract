pub use crate::mapping_key::*;

#[macro_export]
macro_rules! define_map {
    ($name:ident, $key_ty:ty, $value_ty:ty) => {
        #[ink::storage_item(derive = false)]
        #[derive(
            ink::storage::traits::Storable,
            ink::storage::traits::StorableHint,
            ink::storage::traits::StorageKey,
            Default,
            Debug,
        )]
        #[cfg_attr(
            feature = "std",
            derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
        )]
        pub struct $name {
            next_id: $key_ty,
            store: ink::storage::Mapping<$key_ty, $value_ty, ink::storage::traits::AutoKey>,
        }

        impl $name {
            // get map length
            pub fn len(&self) -> $key_ty {
                self.next_id
            }

            // get next id
            pub fn next_id(&mut self) -> $key_ty {
                self.next_id
            }

            pub fn contains(&self, key: &$key_ty) -> bool {
                self.store.contains(key)
            }

            // insert value
            pub fn insert(&mut self, value: &$value_ty) -> $key_ty {
                let key = self.next_id;
                // Use checked_add to prevent overflow
                self.next_id = self.next_id.checked_add(1).expect("next_id overflow");
                self.store.insert(key, value);

                key
            }

            // get value
            pub fn get(&self, key: $key_ty) -> Option<$value_ty> {
                self.store.get(key)
            }

            pub fn update(&mut self, key: $key_ty, value: &$value_ty) -> Option<u32> {
                self.store.insert(key, value)
            }

            // get list by k1 desc
            pub fn desc_list(
                &self,
                start_key_: Option<$key_ty>,
                size: $key_ty,
            ) -> ink::prelude::vec::Vec<($key_ty, $value_ty)> {
                let total_len = self.next_id;
                let mut list = ink::prelude::vec::Vec::new();
                if total_len == 0 || size == 0 {
                    return list;
                }

                let start = start_key_.unwrap_or(total_len);
                // Use saturating_add to prevent overflow in range
                let max_i = size.saturating_add(1);
                for i in 1..max_i {
                    // Use checked_sub to prevent underflow
                    if let Some(k) = start.checked_sub(i) {
                        if let Some(v) = self.store.get(k) {
                            list.push((k, v));
                        }
                    } else {
                        break;
                    }
                }

                return list;
            }

            // get list by page and page asc
            pub fn list(
                &self,
                start_key: $key_ty,
                size: $key_ty,
            ) -> ink::prelude::vec::Vec<($key_ty, $value_ty)> {
                let total_len = self.next_id;
                let mut list = ink::prelude::vec::Vec::new();
                if total_len == 0 || size == 0 {
                    return list;
                }

                let start = start_key;
                // Check for potential overflow
                let max_size = size.saturating_add(1);
                for i in 0..max_size {
                    // Use checked_add to prevent overflow
                    if let Some(k) = start.checked_add(i) {
                        if k >= total_len {
                            break;
                        }
                        if let Some(v) = self.store.get(k) {
                            list.push((k, v));
                        }
                    } else {
                        break;
                    }
                }

                return list;
            }
        }
    };
}

#[macro_export]
macro_rules! define_double_map_base {
    ($name:ident, $k1_ty:ty, $value_ty:ty, $mid_ty:ty, $realk_ty:ty) => {
        #[ink::storage_item(derive = false)]
        #[derive(
            ink::storage::traits::Storable,
            ink::storage::traits::StorableHint,
            ink::storage::traits::StorageKey,
            Default,
            Debug,
        )]
        #[cfg_attr(
            feature = "std",
            derive(ink::scale_info::TypeInfo, ink::storage::traits::StorageLayout)
        )]
        pub struct $name {
            // k1 to small
            k1: ink::storage::Mapping<$k1_ty, $mid_ty>,
            // k1 length
            k1_length: $mid_ty,
            // k2 next id
            k2_next_id: ink::storage::Mapping<$mid_ty, $mid_ty>,
            // relation k1 k2 to value
            store: ink::storage::Mapping<$realk_ty, $value_ty>,
        }

        impl $name {
            // next id of k2 for k1
            pub fn next_id(&self, k: $k1_ty) -> $mid_ty {
                let id = self.k1.get(&k);
                if id.is_none() {
                    return 0;
                }
                self.k2_next_id.get(id.unwrap()).unwrap_or_default()
            }

            // next id of k2 for k1
            pub fn len(&self, k: $k1_ty) -> $mid_ty {
                let id = self.k1.get(&k);
                if id.is_none() {
                    return 0;
                }
                self.k2_next_id.get(id.unwrap()).unwrap_or_default()
            }

            // insert value with k1 require k2
            pub fn insert(&mut self, k: $k1_ty, v: &$value_ty) -> $mid_ty {
                // get id
                let mut id = self.k1.get(&k);
                if id.is_none() {
                    let len = self.k1_length;
                    id = Some(len);

                    // save key in
                    self.k1.insert(&k, &len);
                    // Use checked_add to prevent overflow
                    self.k1_length = self.k1_length.checked_add(1).expect("k1_length overflow");
                }

                // save next id
                let id_val = id.unwrap();
                let next_id = self.k2_next_id.get(id_val).unwrap_or_default();
                // Use checked_add to prevent overflow
                let new_next_id = next_id.checked_add(1).expect("next_id overflow");
                self.k2_next_id.insert(id_val, &new_next_id);

                let key = primitives::combine(id_val, next_id);
                self.store.insert(key, v);

                next_id
            }

            // replace value for k1 and k2
            pub fn update(&mut self, k1: $k1_ty, k2: $mid_ty, v: &$value_ty) -> Option<u32> {
                let id = self.k1.get(&k1);
                if id.is_none() {
                    return None;
                }

                let key = primitives::combine(id.unwrap(), k2);
                self.store.insert(key, v)
            }

            // get value by k1 and k2
            pub fn get(&self, k1: $k1_ty, k2: $mid_ty) -> Option<$value_ty> {
                let id = self.k1.get(&k1);
                if id.is_none() {
                    return None;
                }

                let key = primitives::combine(id.unwrap(), k2);
                self.store.get(key)
            }

            // get list by k1 and page desc
            pub fn desc_list(
                &self,
                k1: $k1_ty,
                start_key_: Option<$mid_ty>,
                size: $mid_ty,
            ) -> ink::prelude::vec::Vec<($mid_ty, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let id_val = id.unwrap();
                let total_len = self.k2_next_id.get(&id_val).unwrap_or_default();
                if total_len == 0 || size == 0 {
                    return list;
                }

                let start_key = start_key_.unwrap_or(if total_len > 0 { total_len - 1 } else { 0 });
                for i in 0..size {
                    // Use checked_sub to prevent underflow
                    if let Some(k2) = start_key.checked_sub(i) {
                        let k = primitives::combine(id_val, k2);
                        if let Some(v) = self.store.get(k) {
                            list.push((k2, v));
                        }
                    } else {
                        break;
                    }
                }

                return list;
            }

            // get list by page and page asc
            pub fn list(
                &self,
                k1: $k1_ty,
                start_key: $mid_ty,
                size: $mid_ty,
            ) -> ink::prelude::vec::Vec<($mid_ty, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let id_val = id.unwrap();
                let total_len = self.k2_next_id.get(&id_val).unwrap_or_default();
                if total_len == 0 || size == 0 || start_key >= total_len {
                    return list;
                }

                // Check for potential overflow
                let max_size = size.saturating_add(1);
                for i in 0..max_size {
                    // Use checked_add to prevent overflow
                    if let Some(k2) = start_key.checked_add(i) {
                        if k2 >= total_len {
                            break;
                        }
                        let k = primitives::combine(id_val, k2);
                        if let Some(v) = self.store.get(k) {
                            list.push((k2, v));
                        }
                    } else {
                        break;
                    }
                }

                list
            }

            pub fn list_all(&self, k1: $k1_ty) -> ink::prelude::vec::Vec<($mid_ty, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let id_val = id.unwrap();
                let total_len = self.k2_next_id.get(&id_val).unwrap_or_default();
                for i in 0..total_len {
                    let k = primitives::combine(id_val, i);
                    let v = self.store.get(k);
                    if v.is_some() {
                        list.push((i, v.unwrap()));
                    }
                }

                list
            }

            // replace deleted item with last item, delete last item
            pub fn delete_by_key(&mut self, k1: $k1_ty, k2: $mid_ty) -> bool {
                let id_wrap = self.k1.get(&k1);
                if id_wrap.is_none() {
                    return false;
                }
                let id = id_wrap.unwrap();

                // Verify that k2 is within valid range
                let total_len = self.k2_next_id.get(id).unwrap_or_default();
                if k2 >= total_len {
                    return false;
                }

                // Verify that the key exists before deleting
                let key = primitives::combine(id, k2);
                if !self.store.contains(key) {
                    return false;
                }

                self.store.remove(key);
                true
            }
        }
    };
}

#[macro_export]
macro_rules! double_u8_map {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        primitives::define_double_map_base!($name, $k1_ty, $value_ty, u8, u16);
    };
}

#[macro_export]
macro_rules! double_u16_map {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        primitives::define_double_map_base!($name, $k1_ty, $value_ty, u16, u32);
    };
}

#[macro_export]
macro_rules! double_u32_map {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        primitives::define_double_map_base!($name, $k1_ty, $value_ty, u32, u64);
    };
}

#[macro_export]
macro_rules! double_u64_map {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
        primitives::define_double_map_base!($name, $k1_ty, $value_ty, u64, u128);
    };
}

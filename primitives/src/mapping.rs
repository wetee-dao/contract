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
                self.next_id += 1;
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

            // get list by k1 and page desc
            pub fn desc_list(
                &self,
                page: $key_ty,
                size: $key_ty,
            ) -> ink::prelude::vec::Vec<($key_ty, $value_ty)> {
                let total_len = self.next_id;
                let mut list = ink::prelude::vec::Vec::new();
                if total_len == 0 || page == 0 || size == 0 {
                    return list;
                }

                let start = total_len;
                let total = if total_len > page * size {
                    page * size
                } else {
                    total_len
                };

                for i in 0..total + 1 {
                    let k = start - i;
                    let v = self.store.get(k);
                    if v.is_some() {
                        list.push((k, v.unwrap()));
                    }
                }

                return list;
            }

            // get list by page and page asc
            pub fn list(
                &self,
                page: $key_ty,
                size: $key_ty,
            ) -> ink::prelude::vec::Vec<($key_ty, $value_ty)> {
                let total_len = self.next_id;
                let mut list = ink::prelude::vec::Vec::new();
                if total_len == 0 || page == 0 || size == 0 {
                    return list;
                }

                let start = 0;
                let total = if total_len > page * size {
                    page * size
                } else {
                    total_len
                };

                for i in 0..total {
                    let v = self.store.get(start + i);
                    if v.is_some() {
                        list.push((i, v.unwrap()));
                    }
                }
                return list;
            }
        }
    };
}

#[macro_export]
macro_rules! define_double_map {
    ($name:ident, $k1_ty:ty, $value_ty:ty) => {
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
            // k1 to u32
            k1: ink::storage::Mapping<$k1_ty, u32>,
            // k1 length
            k1_length: u32,
            // k2 next id
            k2_next_id: ink::storage::Mapping<u32, u32>,
            // relation k1 k2 to value
            store: ink::storage::Mapping<u64, $value_ty>,
        }

        impl $name {
            // next id of k2 for k1
            pub fn next_id(&self, k: $k1_ty) -> u32 {
                let id = self.k1.get(&k);
                if id.is_none() {
                    return 0;
                }
                self.k2_next_id.get(id.unwrap()).unwrap_or_default()
            }

            // next id of k2 for k1
            pub fn len(&self, k: $k1_ty) -> u32 {
                let id = self.k1.get(&k);
                if id.is_none() {
                    return 0;
                }
                self.k2_next_id.get(id.unwrap()).unwrap_or_default()
            }

            // insert value with k1 require k2
            pub fn insert(&mut self, k: $k1_ty, v: &$value_ty) -> Option<u32> {
                // get id
                let mut id = self.k1.get(&k);
                if id.is_none() {
                    let len = self.k1_length;
                    id = Some(len);

                    // save key in
                    self.k1.insert(&k, &len);
                    self.k1_length += 1;
                }

                // save next id
                let next_id = self.k2_next_id.get(id.unwrap()).unwrap_or_default();
                self.k2_next_id.insert(id.unwrap(), &(next_id + 1));

                let key = primitives::combine_u32_to_u64(id.unwrap(), next_id);
                self.store.insert(key, v)
            }

            // replace value for k1 and k2
            pub fn update(&mut self, k1: $k1_ty, k2: u32, v: &$value_ty) -> Option<u32> {
                let id = self.k1.get(&k1);
                if id.is_none() {
                    return None;
                }

                let key = primitives::combine_u32_to_u64(id.unwrap(), k2);
                self.store.insert(key, v)
            }

            // get value by k1 and k2
            pub fn get(&self, k1: $k1_ty, k2: u32) -> Option<$value_ty> {
                let id = self.k1.get(&k1);
                if id.is_none() {
                    return None;
                }

                let key = primitives::combine_u32_to_u64(id.unwrap(), k2);
                self.store.get(key)
            }

            // get list by k1 and page desc
            pub fn desc_list(
                &self,
                k1: $k1_ty,
                page: u32,
                size: u32,
            ) -> ink::prelude::vec::Vec<(u32, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let total_len = self.k2_next_id.get(&id.unwrap()).unwrap_or_default();
                if total_len == 0 || page == 0 || size == 0 {
                    return list;
                }

                let start = primitives::combine_u32_to_u64(id.unwrap(), total_len);
                let total = if total_len > page * size {
                    page * size
                } else {
                    total_len
                };

                for i in 0..total + 1 {
                    let k = start - i as u64;
                    let v = self.store.get(k);
                    let (_, k2) = primitives::split_u64_to_u32(k);
                    if v.is_some() {
                        list.push((k2, v.unwrap()));
                    }
                }

                return list;
            }

            // get list by page and page asc
            pub fn list(
                &self,
                k1: $k1_ty,
                page: u32,
                size: u32,
            ) -> ink::prelude::vec::Vec<(u32, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let total_len = self.k2_next_id.get(&id.unwrap()).unwrap_or_default();
                if total_len == 0 || page == 0 || size == 0 {
                    return list;
                }

                let start = primitives::combine_u32_to_u64(id.unwrap(), 0);
                let total = if total_len > page * size {
                    page * size
                } else {
                    total_len
                };

                let mut list = ink::prelude::vec::Vec::new();
                for i in 0..total {
                    let k = start + i as u64;
                    let v = self.store.get(k);
                    let (_, k2) = primitives::split_u64_to_u32(k);
                    if v.is_some() {
                        list.push((k2, v.unwrap()));
                    }
                }

                return list;
            }
        }
    };
}

/// combine u32 to u64
pub fn combine_u32_to_u64(k1: u32, k2: u32) -> u64 {
    ((k1 as u64) << 32) | (k2 as u64)
}

/// split u64 to u32
pub fn split_u64_to_u32(key: u64) -> (u32, u32) {
    let k1 = (key >> 32) as u32;
    let k2 = key as u32;
    (k1, k2)
}

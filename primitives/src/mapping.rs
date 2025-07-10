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
                    self.k1_length += 1;
                }

                // save next id
                let next_id = self.k2_next_id.get(id.unwrap()).unwrap_or_default();
                self.k2_next_id.insert(id.unwrap(), &(next_id + 1));

                let key = primitives::combine(id.unwrap(), next_id);
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
                page: $mid_ty,
                size: $mid_ty,
            ) -> ink::prelude::vec::Vec<($mid_ty, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let total_len = self.k2_next_id.get(&id.unwrap()).unwrap_or_default();
                if total_len == 0 || page == 0 || size == 0 {
                    return list;
                }

                let start = primitives::combine(id.unwrap(), total_len);
                let total = if total_len > page * size {
                    page * size
                } else {
                    total_len
                };

                for i in 0..total + 1 {
                    let k = start - i as $realk_ty;
                    let v = self.store.get(k);
                    let (_, k2) = primitives::split(k);
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
                page: $mid_ty,
                size: $mid_ty,
            ) -> ink::prelude::vec::Vec<($mid_ty, $value_ty)> {
                let id = self.k1.get(&k1);
                let mut list = ink::prelude::vec::Vec::new();
                if id.is_none() {
                    return list;
                }

                let total_len = self.k2_next_id.get(&id.unwrap()).unwrap_or_default();
                if total_len == 0 || page == 0 || size == 0 {
                    return list;
                }

                let start = primitives::combine(id.unwrap(), 0);
                let total = if total_len > page * size {
                    page * size
                } else {
                    total_len
                };

                let mut list = ink::prelude::vec::Vec::new();
                for i in 0..total {
                    let k = start + i as $realk_ty;
                    let v = self.store.get(k);
                    let (_, k2) = primitives::split(k);
                    if v.is_some() {
                        list.push((k2, v.unwrap()));
                    }
                }

                return list;
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

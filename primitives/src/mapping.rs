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

/// combine two type to one
pub trait CombineKey {
    type Output;
    fn combine(high: Self, low: Self) -> Self::Output;
}

/// split one type to two
pub trait SplitKey {
    type Small;
    fn split(self) -> (Self::Small, Self::Small);
}

impl CombineKey for u32 {
    type Output = u64;

    fn combine(high: u32, low: u32) -> u64 {
        ((high as u64) << 32) | (low as u64)
    }
}

impl SplitKey for u64 {
    type Small = u32;

    fn split(self) -> (u32, u32) {
        let k1 = (self >> 32) as u32;
        let k2 = self as u32;
        (k1, k2)
    }
}

impl CombineKey for u64 {
    type Output = u128;

    fn combine(high: u64, low: u64) -> u128 {
        ((high as u128) << 64) | (low as u128)
    }
}

impl SplitKey for u128 {
    type Small = u64;

    fn split(self) -> (u64, u64) {
        let k1 = (self >> 64) as u64;
        let k2 = self as u64;
        (k1, k2)
    }
}

impl CombineKey for u8 {
    type Output = u16;

    fn combine(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | (low as u16)
    }
}

impl SplitKey for u16 {
    type Small = u8;

    fn split(self) -> (u8, u8) {
        let k1 = (self >> 8) as u8;
        let k2 = self as u8;
        (k1, k2)
    }
}

impl CombineKey for u16 {
    type Output = u32;

    fn combine(high: u16, low: u16) -> u32 {
        ((high as u32) << 16) | (low as u32)
    }
}

impl SplitKey for u32 {
    type Small = u16;

    fn split(self) -> (u16, u16) {
        let k1 = (self >> 16) as u16;
        let k2 = self as u16;
        (k1, k2)
    }
}

// combine two key to one
pub fn combine<T: CombineKey>(high: T, low: T) -> T::Output {
    T::combine(high, low)
}

// split one key to two
pub fn split<T: SplitKey>(val: T) -> (T::Small, T::Small) {
    val.split()
}

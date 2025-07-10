
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

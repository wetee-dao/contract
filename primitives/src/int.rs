const MATH_UNIT: i128 = 1_000_000;

pub type FixedI64 = i128;
pub const fn fixed_from_i64(u: i64) -> FixedI64 {
    u as i128 * MATH_UNIT
}

pub const fn fixed_from_u64(u: u64) -> FixedI64 {
    u as i128 * MATH_UNIT
}

pub const fn u32_from_fixed(i: FixedI64) -> u32 {
    (i / MATH_UNIT) as u32
}

/// Percent v/1000
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Percent {
    pub v: u32,
}

impl Percent {
    pub const fn from(v: u32) -> Self {
        if v > 100 {
            panic!("percent overflow")
        }
        Self { v: v * 10 }
    }

    pub fn mul_fixed(&self, i: FixedI64) -> FixedI64 {
        i * self.v as i128 / 1000
    }

    pub fn mul_u32(&self, u: u32) -> u32 {
        u * self.v / 1000
    }

    pub fn mul_u64(&self, u: u64) -> u64 {
        u * self.v as u64 / 1000
    }

    pub fn mul_i64(&self, u: i64) -> i64 {
        u * self.v as i64 / 1000
    }
}

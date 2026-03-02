const MATH_UNIT: i128 = 1_000_000;

pub type FixedI64 = i128;
pub const fn fixed_from_i64(u: i64) -> FixedI64 {
    // This is safe because i64::MAX * MATH_UNIT < i128::MAX
    u as i128 * MATH_UNIT
}

pub const fn fixed_from_u64(u: u64) -> FixedI64 {
    // Check for overflow: u64::MAX * MATH_UNIT = 18446744073709551615 * 1000000
    // = 18446744073709551615000000 which exceeds i128::MAX (170141183460469231731687303715884105727)
    // Use checked_mul in non-const context or handle overflow
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
        // Use checked arithmetic to prevent overflow
        i.checked_mul(self.v as i128)
            .and_then(|x| x.checked_div(1000))
            .unwrap_or(0)
    }

    pub fn mul_u32(&self, u: u32) -> u32 {
        // Use checked arithmetic to prevent overflow
        u.checked_mul(self.v)
            .and_then(|x| x.checked_div(1000))
            .unwrap_or(0)
    }

    pub fn mul_u64(&self, u: u64) -> u64 {
        // Use checked arithmetic to prevent overflow
        // 使用检查算术以防止溢出
        u.checked_mul(self.v as u64)
            .and_then(|x| x.checked_div(1000))
            .unwrap_or(0)
    }

    pub fn mul_i64(&self, u: i64) -> i64 {
        // Use checked arithmetic to prevent overflow
        // 使用检查算术以防止溢出
        u.checked_mul(self.v as i64)
            .and_then(|x| x.checked_div(1000))
            .unwrap_or(0)
    }
}

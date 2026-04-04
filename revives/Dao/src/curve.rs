use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use wrevive_api::BlockNumber;

const FIXED_ONE: i128 = 1_000_000;

fn fixed_from_i64(v: i64) -> i128 {
    (v as i128) * FIXED_ONE
}

fn fixed_from_u64(v: u64) -> i128 {
    (v as i128) * FIXED_ONE
}

fn u32_from_fixed(v: i128) -> u32 {
    if v <= 0 {
        0
    } else {
        (v / FIXED_ONE) as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct Percent {
    pub v: u32,
}

impl Percent {
    pub fn mul_i64(&self, value: i64) -> i64 {
        ((value as i128) * (self.v as i128) / 10_000) as i64
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum CurveArg {
    LinearDecreasing { begin: u32, end: u32, length: BlockNumber },
    SteppedDecreasing { begin: u32, end: u32, step: u32, period: BlockNumber },
    Reciprocal {
        x_offset_percent: Percent,
        x_scale_arg: u32,
        begin: u32,
        end: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum Curve {
    LinearDecreasing { begin: u32, end: u32, length: BlockNumber },
    SteppedDecreasing { begin: u32, end: u32, step: u32, period: BlockNumber },
    Reciprocal { factor: u32, x_scale: u32, x_offset: i64, y_offset: i64 },
}

impl Curve {
    pub fn y(&self, x: BlockNumber) -> u32 {
        match self {
            Curve::LinearDecreasing { begin, end, length } => {
                if x >= *length {
                    return *end;
                }
                let slope = fixed_from_i64((begin - end) as i64) * fixed_from_i64(x as i64)
                    / fixed_from_i64(*length as i64);
                u32_from_fixed(fixed_from_i64(*begin as i64) - slope)
            }
            Curve::SteppedDecreasing {
                begin,
                end,
                step,
                period,
            } => {
                if *period == 0 || x < *period {
                    return *begin;
                }
                let num_steps = x / *period;
                let sub_value = num_steps * *step;
                if sub_value >= *begin || begin.saturating_sub(sub_value) <= *end {
                    return *end;
                }
                begin - sub_value
            }
            Curve::Reciprocal {
                factor,
                x_scale,
                x_offset,
                y_offset,
            } => {
                let denom = fixed_from_u64(x as u64) / (*x_scale as i128) + fixed_from_i64(*x_offset);
                if denom <= 0 {
                    return 0;
                }
                let value = fixed_from_i64(*factor as i64) / denom - (*y_offset as i128);
                value.max(0) as u32
            }
        }
    }
}

pub fn arg_to_curve(arg: CurveArg) -> Curve {
    match arg {
        CurveArg::LinearDecreasing { begin, end, length } => {
            Curve::LinearDecreasing { begin, end, length }
        }
        CurveArg::SteppedDecreasing {
            begin,
            end,
            step,
            period,
        } => Curve::SteppedDecreasing {
            begin,
            end,
            step,
            period,
        },
        CurveArg::Reciprocal {
            begin,
            end,
            x_offset_percent,
            x_scale_arg,
        } => {
            let x_scale = if x_scale_arg == 0 { 1 } else { x_scale_arg };
            let mut slot = begin - end;
            let mut x_offset: i64 = 0;
            let y_offset: i64 = -(end as i64);

            if x_offset_percent.v > 0 {
                let x = x_offset_percent.mul_i64(slot as i64);
                let y = fixed_from_i64(slot as i64)
                    / (fixed_from_u64(x as u64) + fixed_from_i64(x_offset));
                let ratio = if y == 0 { 1 } else { slot / u32_from_fixed(y).max(1) };
                slot *= ratio.max(1);
                x_offset = x;
            } else {
                x_offset = if x_scale > 1 { x_scale as i64 } else { 1 };
            }

            Curve::Reciprocal {
                factor: slot,
                x_scale,
                x_offset,
                y_offset,
            }
        }
    }
}

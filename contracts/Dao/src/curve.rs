use ink::env::BlockNumber;
use primitives::{fixed_from_i64, fixed_from_u64, u32_from_fixed, Percent};

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum CurveArg {
    LinearDecreasing {
        // 开始 最大
        begin: u32,
        // 结束 最小
        end: u32,
        // 轨道长度
        length: BlockNumber,
    },
    SteppedDecreasing {
        // 开始 最大
        begin: u32,
        // 结束 最小
        end: u32,
        // 下降步长
        step: u32,
        // 下降周期
        period: BlockNumber,
    },
    Reciprocal{
        x_offset_percent: Percent, 
        x_scale_arg: u32, 
        begin: u32, 
        end: u32,
    }
}

/// 投票轨道
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Curve {
    /// Linear curve starting at `(0, ceil)`, proceeding linearly to `(length, floor)`, then
    /// remaining at `floor` until the end of the period.
    LinearDecreasing {
        // 开始 最大
        begin: u32,
        // 结束 最小
        end: u32,
        // 轨道长度
        length: BlockNumber,
    },

    /// Stepped curve, beginning at `(0, begin)`, then remaining constant for `period`, at which
    /// point it steps down to `(period, begin - step)`. It then remains constant for another
    /// `period` before stepping down to `(period * 2, begin - step * 2)`. This pattern continues
    /// but the `y` component has a lower limit of `end`.
    SteppedDecreasing {
        // 开始 最大
        begin: u32,
        // 结束 最小
        end: u32,
        // 下降步长
        step: u32,
        // 下降周期
        period: BlockNumber,
    },

    /// A recipocal (`K/(x+S)-T`) curve: `factor` is `K` and `x_offset` is `S`, `y_offset` is `T`.
    Reciprocal {
        // 轨道系数
        factor: u32,
        // x轴缩放系数
        x_scale: u32,
        // 轨道偏移量
        x_offset: i64,
        // 轨道偏移量
        y_offset: i64,
    },
}

impl Curve {
    pub fn y(&self, x: BlockNumber) -> u32 {
        match self {
            Curve::LinearDecreasing { begin, end, length } => {
                if x >= *length {
                    return *end;
                }

                // 计算斜率
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

                let num_steps = x / (*period);
                let sub_value = num_steps * (*step as u32);

                if sub_value > 255 || sub_value as u32 >= *begin || begin - sub_value as u32 <= *end
                {
                    return *end;
                }

                begin - sub_value as u32
            }
            Curve::Reciprocal {
                factor,
                x_scale,
                x_offset,
                y_offset,
            } => {
                // A recipocal (`K/(x+S)-T`) curve: `factor` is `K` and `x_offset` is `S`, `y_offset` is `T`.
                ((fixed_from_i64(*factor as i64)
                    / (fixed_from_u64(x as u64) / (*x_scale as i128)
                        + fixed_from_u64(*x_offset as u64)))
                    - *y_offset as i128) as u32
            }
        }
    }
}

pub fn arg_to_curve(arg: CurveArg) -> Curve {
    match arg {
        CurveArg::LinearDecreasing { begin, end, length } => Curve::LinearDecreasing {
            begin: begin,
            end: end,
            length: length,
        },
        CurveArg::SteppedDecreasing {
            begin,
            end,
            step,
            period,
        } => Curve::SteppedDecreasing {
            begin: begin,
            end: end,
            step: step,
            period: period,
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
                let y = (fixed_from_i64(slot as i64)
                    / (fixed_from_u64(x as u64) + fixed_from_i64(x_offset)))
                    as u32;

                let ratio = slot / y;
                slot = slot * ratio;

                x_offset = x as i64;
            } else {
                x_offset = 1;
                if x_scale > 1 {
                    x_offset = x_scale as i64;
                }
            }

            Curve::Reciprocal {
                factor: slot,
                x_scale,
                x_offset: x_offset,
                y_offset: y_offset,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_decreasing_curve() {
        println!("linear_decreasing_curve:");
        let curve = Curve::LinearDecreasing {
            begin: 10000,
            end: 50,
            length: 30,
        };

        for x in 0..=50 {
            let y = curve.y(x);
            println!("x = {}, y = {}", x, y);
        }
    }

    #[test]
    fn test_stepped_decreasing_curve() {
        println!("stepped_decreasing_curve:");
        let curve = Curve::SteppedDecreasing {
            begin: 100,
            end: 50,
            step: 10,
            period: 10,
        };
        for x in 0..=100 {
            let y = curve.y(x);
            println!("x = {}, y = {}", x, y);
        }
    }

    // factor / (x + x_offset) - y_offset
    #[test]
    fn test_reciprocal_curve() {
        println!("reciprocal_curve:");
        let curve = arg_to_curve(
            CurveArg::Reciprocal {
                begin: 10000,
                end: 2000,
                x_offset_percent: Percent::from(2),
                x_scale_arg: 100,
            },
        );

        for x in 0..=300 {
            let y = curve.y(x);
            println!("x = {}, y = {}", x, y);
        }
    }
}

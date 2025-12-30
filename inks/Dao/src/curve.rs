use ink::env::BlockNumber;
use primitives::{fixed_from_i64, fixed_from_u64, u32_from_fixed, Percent};

/// Curve argument for creating voting curves
/// 用于创建投票曲线的曲线参数
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum CurveArg {
    /// Linear decreasing curve / 线性递减曲线
    LinearDecreasing {
        /// Starting value (maximum) / 起始值（最大值）
        begin: u32,
        /// Ending value (minimum) / 结束值（最小值）
        end: u32,
        /// Curve length in blocks / 曲线长度（区块数）
        length: BlockNumber,
    },
    /// Stepped decreasing curve / 阶梯递减曲线
    SteppedDecreasing {
        /// Starting value (maximum) / 起始值（最大值）
        begin: u32,
        /// Ending value (minimum) / 结束值（最小值）
        end: u32,
        /// Step size for each decrease / 每次下降的步长
        step: u32,
        /// Period between steps in blocks / 步进之间的周期（区块数）
        period: BlockNumber,
    },
    /// Reciprocal curve (K/(x+S)-T) / 倒数曲线 (K/(x+S)-T)
    Reciprocal{
        /// X-axis offset percentage / X 轴偏移百分比
        x_offset_percent: Percent, 
        /// X-axis scale argument / X 轴缩放参数
        x_scale_arg: u32, 
        /// Starting value (maximum) / 起始值（最大值）
        begin: u32, 
        /// Ending value (minimum) / 结束值（最小值）
        end: u32,
    }
}

/// Voting curve types for calculating approval/support thresholds
/// 用于计算批准/支持阈值的投票曲线类型
#[derive(Clone)]
#[cfg_attr(
    feature = "std",
    derive(Debug, PartialEq, Eq, ink::storage::traits::StorageLayout)
)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Curve {
    /// Linear decreasing curve
    /// Starts at `(0, begin)`, proceeds linearly to `(length, end)`, then remains at `end`.
    /// 线性递减曲线
    /// 从 `(0, begin)` 开始，线性递减到 `(length, end)`，然后保持在 `end`。
    LinearDecreasing {
        /// Starting value (maximum) / 起始值（最大值）
        begin: u32,
        /// Ending value (minimum) / 结束值（最小值）
        end: u32,
        /// Curve length in blocks / 曲线长度（区块数）
        length: BlockNumber,
    },

    /// Stepped decreasing curve
    /// Begins at `(0, begin)`, remains constant for `period`, then steps down to `(period, begin - step)`.
    /// This pattern continues with a lower limit of `end`.
    /// 阶梯递减曲线
    /// 从 `(0, begin)` 开始，保持 `period` 个区块不变，然后下降到 `(period, begin - step)`。
    /// 此模式持续进行，下限为 `end`。
    SteppedDecreasing {
        /// Starting value (maximum) / 起始值（最大值）
        begin: u32,
        /// Ending value (minimum) / 结束值（最小值）
        end: u32,
        /// Step size for each decrease / 每次下降的步长
        step: u32,
        /// Period between steps in blocks / 步进之间的周期（区块数）
        period: BlockNumber,
    },

    /// Reciprocal curve: `K/(x+S)-T`
    /// Where `factor` is `K`, `x_offset` is `S`, and `y_offset` is `T`.
    /// 倒数曲线：`K/(x+S)-T`
    /// 其中 `factor` 是 `K`，`x_offset` 是 `S`，`y_offset` 是 `T`。
    Reciprocal {
        /// Curve factor (K) / 曲线系数 (K)
        factor: u32,
        /// X-axis scale factor / X 轴缩放系数
        x_scale: u32,
        /// X-axis offset (S) / X 轴偏移量 (S)
        x_offset: i64,
        /// Y-axis offset (T) / Y 轴偏移量 (T)
        y_offset: i64,
    },
}

impl Curve {
    /// Calculate the Y value of the curve at a given X (block number)
    /// 计算曲线在给定 X（区块号）处的 Y 值
    /// 
    /// # Arguments
    /// * `x` - Block number / 区块号
    /// 
    /// # Returns
    /// * `u32` - Y value (threshold percentage) / Y 值（阈值百分比）
    pub fn y(&self, x: BlockNumber) -> u32 {
        match self {
            Curve::LinearDecreasing { begin, end, length } => {
                if x >= *length {
                    return *end;
                }

                // Calculate slope / 计算斜率
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
                // Calculate reciprocal curve: K/(x/S + x_offset) - y_offset
                // 计算倒数曲线：K/(x/S + x_offset) - y_offset
                // Where factor is K, x_offset is S, y_offset is T
                // 其中 factor 是 K，x_offset 是 S，y_offset 是 T
                ((fixed_from_i64(*factor as i64)
                    / (fixed_from_u64(x as u64) / (*x_scale as i128)
                        + fixed_from_u64(*x_offset as u64)))
                    - *y_offset as i128) as u32
            }
        }
    }
}

/// Convert CurveArg to Curve
/// 将 CurveArg 转换为 Curve
/// 
/// # Arguments
/// * `arg` - Curve argument / 曲线参数
/// 
/// # Returns
/// * `Curve` - Converted curve / 转换后的曲线
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

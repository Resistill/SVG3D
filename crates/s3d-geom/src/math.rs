//! 确定性数学（PLAN 原则 18；spec/01 1.1.3）：全部超越函数走 libm，native 与 wasm 同一份实现；
//! 禁止直接调用 std 的 `f64::sin` 等（clippy.toml 的 disallowed-methods）。角度一律用度。

pub use glam::{DMat3 as Mat3, DVec2 as Vec2, DVec3 as Vec3};

/// 度 → 弧度。
#[inline]
pub fn rad(deg: f64) -> f64 {
    deg * (core::f64::consts::PI / 180.0)
}

/// 弧度 → 度。
#[inline]
pub fn deg(rad: f64) -> f64 {
    rad * (180.0 / core::f64::consts::PI)
}

#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}
#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}
#[inline]
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}
#[inline]
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}
#[inline]
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}
#[inline]
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}
#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}
#[inline]
pub fn exp2(x: f64) -> f64 {
    libm::exp2(x)
}
#[inline]
pub fn ln(x: f64) -> f64 {
    libm::log(x)
}
#[inline]
pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}
#[inline]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}
#[inline]
pub fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
#[inline]
pub fn hypot(x: f64, y: f64) -> f64 {
    libm::hypot(x, y)
}
#[inline]
pub fn cbrt(x: f64) -> f64 {
    libm::cbrt(x)
}
/// `sqrt` 是 IEEE 754 要求正确舍入的运算，直接用硬件指令在任何平台结果相同。
#[inline]
pub fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

/// 整数幂：逐次相乘（平方求幂），不用 `powi`（后者在不同平台可能走不同的 compiler-rt 实现）。
pub fn powi(mut x: f64, mut n: i32) -> f64 {
    if n < 0 {
        x = 1.0 / x;
        n = -n;
    }
    let mut acc = 1.0;
    while n > 0 {
        if n & 1 == 1 {
            acc *= x;
        }
        x *= x;
        n >>= 1;
    }
    acc
}

/// 按度的正弦 / 余弦：对 0、90、180、270 的整数倍给出精确值（spec/03 要求轴对齐的旋转矩阵元素是精确的 0 / ±1）。
pub fn sin_deg(d: f64) -> f64 {
    let r = d.rem_euclid(360.0);
    if r == 0.0 || r == 180.0 {
        0.0
    } else if r == 90.0 {
        1.0
    } else if r == 270.0 {
        -1.0
    } else {
        sin(rad(d))
    }
}

pub fn cos_deg(d: f64) -> f64 {
    let r = d.rem_euclid(360.0);
    if r == 0.0 {
        1.0
    } else if r == 180.0 {
        -1.0
    } else if r == 90.0 || r == 270.0 {
        0.0
    } else {
        cos(rad(d))
    }
}

/// 向量长度（不用 hypot 的三参数版本，逐项相加后开方，顺序固定）。
#[inline]
pub fn length(v: Vec3) -> f64 {
    sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
}

/// 单位化；零向量（长度 < eps）返回 None。
pub fn unit(v: Vec3, eps: f64) -> Option<Vec3> {
    let l = length(v);
    if l < eps {
        None
    } else {
        Some(v / l)
    }
}

/// 绕单位轴 `axis` 转 `angle_deg` 度的旋转矩阵（Rodrigues；spec/03 3.5.3）。调用方保证 `axis` 已单位化。
pub fn rot_axis_deg(axis: Vec3, angle_deg: f64) -> Mat3 {
    let c = cos_deg(angle_deg);
    let s = sin_deg(angle_deg);
    let t = 1.0 - c;
    let (x, y, z) = (axis.x, axis.y, axis.z);
    // 列主序：from_cols 的参数是三列
    Mat3::from_cols(
        Vec3::new(c + x * x * t, y * x * t + z * s, z * x * t - y * s),
        Vec3::new(x * y * t - z * s, c + y * y * t, z * y * t + x * s),
        Vec3::new(x * z * t + y * s, y * z * t - x * s, c + z * z * t),
    )
}

/// 绕 y 轴转 `d` 度（turn，spec/03 3.5.2）。
pub fn rot_y_deg(d: f64) -> Mat3 {
    rot_axis_deg(Vec3::Y, d)
}
/// 绕 x 轴转 `d` 度（tilt）。
pub fn rot_x_deg(d: f64) -> Mat3 {
    rot_axis_deg(Vec3::X, d)
}
/// 绕 z 轴转 `d` 度。
pub fn rot_z_deg(d: f64) -> Mat3 {
    rot_axis_deg(Vec3::Z, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_axis_angles() {
        assert_eq!(sin_deg(0.0), 0.0);
        assert_eq!(cos_deg(90.0), 0.0);
        assert_eq!(sin_deg(-90.0), -1.0);
        assert_eq!(cos_deg(180.0), -1.0);
        assert_eq!(cos_deg(-270.0), 0.0);
        let r = rot_y_deg(90.0);
        // y 转 90°：x 轴 → -z
        assert_eq!(r * Vec3::X, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn powi_matches_multiplication() {
        assert_eq!(powi(2.0, 10), 1024.0);
        assert_eq!(powi(2.0, -2), 0.25);
        assert_eq!(powi(3.5, 0), 1.0);
    }

    #[test]
    fn rodrigues_is_orthonormal() {
        let axis = unit(Vec3::new(1.0, 2.0, 3.0), 1e-12).unwrap();
        let r = rot_axis_deg(axis, 37.0);
        let i = r * r.transpose();
        for c in 0..3 {
            for k in 0..3 {
                let want = if c == k { 1.0 } else { 0.0 };
                assert!((i.col(c)[k] - want).abs() < 1e-14);
            }
        }
    }
}

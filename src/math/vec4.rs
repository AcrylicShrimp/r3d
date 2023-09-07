use super::{Vec2, Vec3};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use zerocopy::AsBytes;

#[repr(C)]
#[derive(AsBytes, Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        w: 1.0,
    };
    pub const FORWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    pub const BACKWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
        w: 0.0,
    };
    pub const UP: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };
    pub const DOWN: Self = Self {
        x: 0.0,
        y: -1.0,
        z: 0.0,
        w: 0.0,
    };
    pub const LEFT: Self = Self {
        x: -1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const RIGHT: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_vec2(v: Vec2, z: f32, w: f32) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z,
            w,
        }
    }

    pub fn from_vec3(v: Vec3, w: f32) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
    }

    pub fn len(self) -> f32 {
        self.len_square().sqrt()
    }

    pub fn len_square(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    pub fn normalized(self) -> Self {
        let len = self.len();
        if len < f32::EPSILON {
            return Self::ZERO;
        }
        self / len
    }

    pub fn distance(lhs: Self, rhs: Self) -> f32 {
        (lhs - rhs).len()
    }

    pub fn distance_square(lhs: Self, rhs: Self) -> f32 {
        (lhs - rhs).len_square()
    }

    pub fn dot(lhs: Self, rhs: Self) -> f32 {
        lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z + lhs.w * rhs.w
    }

    pub fn project(lhs: Self, normal: Self) -> Self {
        normal * Self::projected_len(lhs, normal)
    }

    pub fn projected_len(lhs: Self, normal: Self) -> f32 {
        Self::dot(lhs, normal) / normal.len()
    }

    pub fn reflect(lhs: Self, normal: Self) -> Self {
        lhs - (2f32 * Self::project(lhs, normal))
    }

    pub fn lerp(from: Self, to: Self, t: f32) -> Self {
        match t {
            t if t <= 0f32 => from,
            t if 1f32 <= t => to,
            t => Self::lerp_unclamped(from, to, t),
        }
    }

    pub fn lerp_unclamped(from: Self, to: Self, t: f32) -> Self {
        from + (to - from) * t
    }

    pub fn floor(lhs: Self) -> Self {
        Self {
            x: lhs.x.floor(),
            y: lhs.y.floor(),
            z: lhs.z.floor(),
            w: lhs.w.floor(),
        }
    }

    pub fn round(lhs: Self) -> Self {
        Self {
            x: lhs.x.round(),
            y: lhs.y.round(),
            z: lhs.z.round(),
            w: lhs.w.round(),
        }
    }

    pub fn ceil(lhs: Self) -> Self {
        Self {
            x: lhs.x.ceil(),
            y: lhs.y.ceil(),
            z: lhs.z.ceil(),
            w: lhs.w.ceil(),
        }
    }

    pub fn abs(lhs: Self) -> Self {
        Self {
            x: lhs.x.abs(),
            y: lhs.y.abs(),
            z: lhs.z.abs(),
            w: lhs.w.abs(),
        }
    }

    pub fn fract(lhs: Self) -> Self {
        Self {
            x: lhs.x.fract(),
            y: lhs.y.fract(),
            z: lhs.z.fract(),
            w: lhs.w.fract(),
        }
    }

    pub fn powi(lhs: Self, n: i32) -> Self {
        Self {
            x: lhs.x.powi(n),
            y: lhs.y.powi(n),
            z: lhs.z.powi(n),
            w: lhs.w.powi(n),
        }
    }

    pub fn powf(lhs: Self, n: f32) -> Self {
        Self {
            x: lhs.x.powf(n),
            y: lhs.y.powf(n),
            z: lhs.z.powf(n),
            w: lhs.w.powf(n),
        }
    }

    pub fn sqrt(lhs: Self) -> Self {
        Self {
            x: lhs.x.sqrt(),
            y: lhs.y.sqrt(),
            z: lhs.z.sqrt(),
            w: lhs.w.sqrt(),
        }
    }

    pub fn exp(lhs: Self) -> Self {
        Self {
            x: lhs.x.exp(),
            y: lhs.y.exp(),
            z: lhs.z.exp(),
            w: lhs.w.exp(),
        }
    }

    pub fn exp2(lhs: Self) -> Self {
        Self {
            x: lhs.x.exp2(),
            y: lhs.y.exp2(),
            z: lhs.z.exp2(),
            w: lhs.w.exp2(),
        }
    }

    pub fn ln(lhs: Self) -> Self {
        Self {
            x: lhs.x.ln(),
            y: lhs.y.ln(),
            z: lhs.z.ln(),
            w: lhs.w.ln(),
        }
    }

    pub fn log(lhs: Self, base: f32) -> Self {
        Self {
            x: lhs.x.log(base),
            y: lhs.y.log(base),
            z: lhs.z.log(base),
            w: lhs.w.log(base),
        }
    }

    pub fn log2(lhs: Self) -> Self {
        Self {
            x: lhs.x.log2(),
            y: lhs.y.log2(),
            z: lhs.z.log2(),
            w: lhs.w.log2(),
        }
    }

    pub fn log10(lhs: Self) -> Self {
        Self {
            x: lhs.x.log10(),
            y: lhs.y.log10(),
            z: lhs.z.log10(),
            w: lhs.w.log10(),
        }
    }

    pub fn min(lhs: Self, rhs: Self) -> Self {
        Self {
            x: f32::min(lhs.x, rhs.x),
            y: f32::min(lhs.y, rhs.y),
            z: f32::min(lhs.z, rhs.z),
            w: f32::min(lhs.w, rhs.w),
        }
    }

    pub fn max(lhs: Self, rhs: Self) -> Self {
        Self {
            x: f32::max(lhs.x, rhs.x),
            y: f32::max(lhs.y, rhs.y),
            z: f32::max(lhs.z, rhs.z),
            w: f32::max(lhs.w, rhs.w),
        }
    }

    pub fn recip(lhs: Self) -> Self {
        Self {
            x: lhs.x.recip(),
            y: lhs.y.recip(),
            z: lhs.z.recip(),
            w: lhs.w.recip(),
        }
    }
}

impl Default for Vec4 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for Vec4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl AddAssign for Vec4 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl Sub for Vec4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl SubAssign for Vec4 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl Mul for Vec4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl MulAssign for Vec4 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
            w: self * rhs.w,
        }
    }
}

impl Div for Vec4 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }
}

impl DivAssign for Vec4 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self.w /= rhs.w;
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl DivAssign<f32> for Vec4 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}

impl Neg for Vec4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl Display for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Vec4(x={}, y={}, z={}, w={})",
            self.x, self.y, self.z, self.w
        )
    }
}

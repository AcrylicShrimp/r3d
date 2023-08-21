use super::{Vec2, Vec4};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use zerocopy::AsBytes;

#[repr(C)]
#[derive(AsBytes, Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const FORWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const BACKWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };
    pub const UP: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const DOWN: Self = Self {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };
    pub const LEFT: Self = Self {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const RIGHT: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn from_vec2(vec2: Vec2, z: f32) -> Self {
        Self {
            x: vec2.x,
            y: vec2.y,
            z,
        }
    }

    pub fn from_vec4(vec4: Vec4) -> Self {
        Self {
            x: vec4.x,
            y: vec4.y,
            z: vec4.z,
        }
    }

    pub fn len(self) -> f32 {
        self.len_square().sqrt()
    }

    pub fn len_square(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
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
        lhs.x * rhs.x + lhs.y * rhs.y + lhs.z * rhs.z
    }

    pub fn cross(lhs: Self, rhs: Self) -> Self {
        Self {
            x: lhs.y * rhs.z - lhs.z * rhs.y,
            y: lhs.z * rhs.x - lhs.x * rhs.z,
            z: lhs.x * rhs.y - lhs.y * rhs.x,
        }
    }

    pub fn project(lhs: Self, normal: Self) -> Self {
        normal * Self::projected_len(lhs, normal)
    }

    pub fn projected_len(lhs: Self, normal: Self) -> f32 {
        Self::dot(lhs, normal) / normal.len()
    }

    pub fn angle(from: Self, to: Self) -> f32 {
        (Self::dot(from, to) / (from.len() * to.len())).acos()
    }

    pub fn angle_signed(from: Self, to: Self, normal: Self) -> f32 {
        let angle = Self::angle(from, to);
        let perpendicular = Self::cross(normal, from);
        let projected_length = Self::dot(perpendicular, to);

        if 0f32 <= projected_length {
            angle
        } else {
            -angle
        }
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

    pub fn slerp(from: Self, to: Self, t: f32) -> Self {
        match t {
            t if t <= 0f32 => from,
            t if 1f32 <= t => to,
            t => Self::slerp_unclamped(from, to, t),
        }
    }

    pub fn slerp_unclamped(from: Self, to: Self, t: f32) -> Self {
        let angle = Self::angle(from, to);
        let sin = angle.sin();

        if sin < f32::EPSILON {
            return Self::lerp_unclamped(from, to, t);
        }

        let inv_sin = sin.recip();
        let lhs = (from * (angle * (1f32 - t)).sin()) * inv_sin;
        let rhs = (to * (angle * t).sin()) * inv_sin;

        lhs + rhs
    }

    pub fn floor(lhs: Self) -> Self {
        Self {
            x: lhs.x.floor(),
            y: lhs.y.floor(),
            z: lhs.z.floor(),
        }
    }

    pub fn round(lhs: Self) -> Self {
        Self {
            x: lhs.x.round(),
            y: lhs.y.round(),
            z: lhs.z.round(),
        }
    }

    pub fn ceil(lhs: Self) -> Self {
        Self {
            x: lhs.x.ceil(),
            y: lhs.y.ceil(),
            z: lhs.z.ceil(),
        }
    }

    pub fn abs(lhs: Self) -> Self {
        Self {
            x: lhs.x.abs(),
            y: lhs.y.abs(),
            z: lhs.z.abs(),
        }
    }

    pub fn fract(lhs: Self) -> Self {
        Self {
            x: lhs.x.fract(),
            y: lhs.y.fract(),
            z: lhs.z.fract(),
        }
    }

    pub fn powi(lhs: Self, n: i32) -> Self {
        Self {
            x: lhs.x.powi(n),
            y: lhs.y.powi(n),
            z: lhs.z.powi(n),
        }
    }

    pub fn powf(lhs: Self, n: f32) -> Self {
        Self {
            x: lhs.x.powf(n),
            y: lhs.y.powf(n),
            z: lhs.z.powf(n),
        }
    }

    pub fn sqrt(lhs: Self) -> Self {
        Self {
            x: lhs.x.sqrt(),
            y: lhs.y.sqrt(),
            z: lhs.z.sqrt(),
        }
    }

    pub fn exp(lhs: Self) -> Self {
        Self {
            x: lhs.x.exp(),
            y: lhs.y.exp(),
            z: lhs.z.exp(),
        }
    }

    pub fn exp2(lhs: Self) -> Self {
        Self {
            x: lhs.x.exp2(),
            y: lhs.y.exp2(),
            z: lhs.z.exp2(),
        }
    }

    pub fn ln(lhs: Self) -> Self {
        Self {
            x: lhs.x.ln(),
            y: lhs.y.ln(),
            z: lhs.z.ln(),
        }
    }

    pub fn log(lhs: Self, base: f32) -> Self {
        Self {
            x: lhs.x.log(base),
            y: lhs.y.log(base),
            z: lhs.z.log(base),
        }
    }

    pub fn log2(lhs: Self) -> Self {
        Self {
            x: lhs.x.log2(),
            y: lhs.y.log2(),
            z: lhs.z.log2(),
        }
    }

    pub fn log10(lhs: Self) -> Self {
        Self {
            x: lhs.x.log10(),
            y: lhs.y.log10(),
            z: lhs.z.log10(),
        }
    }

    pub fn min(lhs: Self, rhs: Self) -> Self {
        Self {
            x: f32::min(lhs.x, rhs.x),
            y: f32::min(lhs.y, rhs.y),
            z: f32::min(lhs.z, rhs.z),
        }
    }

    pub fn max(lhs: Self, rhs: Self) -> Self {
        Self {
            x: f32::max(lhs.x, rhs.x),
            y: f32::max(lhs.y, rhs.y),
            z: f32::max(lhs.z, rhs.z),
        }
    }

    pub fn recip(lhs: Self) -> Self {
        Self {
            x: lhs.x.recip(),
            y: lhs.y.recip(),
            z: lhs.z.recip(),
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<Vec4> for Vec3 {
    fn from(value: Vec4) -> Self {
        Self::from_vec4(value)
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Div for Vec3 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec3(x={}, y={}, z={})", self.x, self.y, self.z)
    }
}

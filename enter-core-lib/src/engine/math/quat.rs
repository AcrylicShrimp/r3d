use super::{Mat4, Vec3};
use std::{
    fmt::Display,
    ops::{Mul, MulAssign, Neg},
};
use zerocopy::AsBytes;

#[repr(C)]
#[derive(AsBytes, Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn from_eular(x: f32, y: f32, z: f32) -> Self {
        let half_x = x * 0.5;
        let half_y = y * 0.5;
        let half_z = z * 0.5;

        let sin_x = half_x.sin();
        let cos_x = half_x.cos();
        let sin_y = half_y.sin();
        let cos_y = half_y.cos();
        let sin_z = half_z.sin();
        let cos_z = half_z.cos();

        Self {
            x: sin_x * cos_y * cos_z + cos_x * sin_y * sin_z,
            y: cos_x * sin_y * cos_z - sin_x * cos_y * sin_z,
            z: cos_x * cos_y * sin_z - sin_x * sin_y * cos_z,
            w: cos_x * cos_y * cos_z + sin_x * sin_y * sin_z,
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let s = half_angle.sin();

        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: half_angle.cos(),
        }
    }

    pub fn from_mat4(mat: &Mat4) -> Self {
        let elements = &mat.elements;
        let trace = elements[0] + elements[5] + elements[10];
        let quat = if 0.0 < trace {
            let s = (trace + 1.0).sqrt() * 2.0;
            let inv_s = s.recip();
            Self {
                x: (elements[6] - elements[9]) * inv_s,
                y: (elements[8] - elements[2]) * inv_s,
                z: (elements[1] - elements[4]) * inv_s,
                w: 0.25 * s,
            }
        } else if elements[0] > elements[5] && elements[0] > elements[10] {
            let s = (1.0 + elements[0] - elements[5] - elements[10]).sqrt() * 2.0;
            let inv_s = s.recip();
            Self {
                x: 0.25 * s,
                y: (elements[4] + elements[1]) * inv_s,
                z: (elements[2] + elements[8]) * inv_s,
                w: (elements[6] - elements[9]) * inv_s,
            }
        } else if elements[5] > elements[10] {
            let s = (1.0 + elements[5] - elements[0] - elements[10]).sqrt() * 2.0;
            let inv_s = s.recip();
            Self {
                x: (elements[4] + elements[1]) * inv_s,
                y: 0.25 * s,
                z: (elements[9] + elements[6]) * inv_s,
                w: (elements[8] - elements[2]) * inv_s,
            }
        } else {
            let s = (1.0 + elements[10] - elements[0] - elements[5]).sqrt() * 2.0;
            let inv_s = s.recip();
            Self {
                x: (elements[2] + elements[8]) * inv_s,
                y: (elements[9] + elements[6]) * inv_s,
                z: 0.25 * s,
                w: (elements[1] - elements[4]) * inv_s,
            }
        };
        quat.normalized()
    }

    pub fn normalize(&mut self) -> &mut Self {
        let len = self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w;
        if len != 1.0 && len != 0.0 {
            let len = len.sqrt();
            self.x /= len;
            self.y /= len;
            self.z /= len;
            self.w /= len;
        }
        self
    }

    pub fn normalized(self) -> Self {
        let mut result = self;
        result.normalize();
        result
    }

    pub fn conjugate(&mut self) -> &mut Self {
        self.x = -self.x;
        self.y = -self.y;
        self.z = -self.z;
        self
    }

    pub fn conjugated(self) -> Self {
        let mut result = self;
        result.conjugate();
        result
    }

    pub fn invert(&mut self) -> &mut Self {
        self.conjugate().normalize();
        self
    }

    pub fn inverted(self) -> Self {
        let mut result = self;
        result.invert();
        result
    }

    pub fn into_eular(self) -> Vec3 {
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if 1.0 <= sinp.abs() {
            sinp.signum() * std::f32::consts::PI / 2.0
        } else {
            sinp.asin()
        };

        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        Vec3::new(roll, pitch, yaw)
    }

    pub fn into_mat4(self) -> Mat4 {
        let x2 = self.x + self.x;
        let y2 = self.y + self.y;
        let z2 = self.z + self.z;

        let xx = self.x * x2;
        let xy = self.x * y2;
        let xz = self.x * z2;

        let yy = self.y * y2;
        let yz = self.y * z2;
        let zz = self.z * z2;

        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        Mat4::new([
            1.0 - (yy + zz),
            xy + wz,
            xz - wy,
            0.0,
            xy - wz,
            1.0 - (xx + zz),
            yz + wx,
            0.0,
            xz + wy,
            yz - wx,
            1.0 - (xx + yy),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ])
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y + self.y * rhs.w + self.z * rhs.x - self.x * rhs.z,
            z: self.w * rhs.z + self.z * rhs.w + self.x * rhs.y - self.y * rhs.x,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl MulAssign for Quat {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Vec3> for Quat {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let qvec = Vec3::new(self.x, self.y, self.z);
        let uv = Vec3::cross(qvec, rhs);
        let uuv = Vec3::cross(qvec, uv);
        rhs + ((self.w * uv) + uuv) * 2.0
    }
}

impl Mul<Quat> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Quat) -> Self::Output {
        let qvec = Vec3::new(rhs.x, rhs.y, rhs.z);
        let uv = Vec3::cross(qvec, self);
        let uuv = Vec3::cross(qvec, uv);
        self + ((rhs.w * uv) + uuv) * 2.0
    }
}

impl MulAssign<Quat> for Vec3 {
    fn mul_assign(&mut self, rhs: Quat) {
        *self = *self * rhs;
    }
}

impl Neg for Quat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }
}

impl Display for Quat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let eular = self.into_eular();
        write!(
            f,
            "Quat(x={}, y={}, z={})",
            eular.x.to_degrees(),
            eular.y.to_degrees(),
            eular.z.to_degrees()
        )
    }
}

use super::{Quat, Vec3, Vec4};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign},
};
use zerocopy::AsBytes;

#[repr(C)]
#[derive(AsBytes, Debug, Clone, PartialEq)]
pub struct Mat4 {
    pub elements: [f32; 16],
}

impl Mat4 {
    pub fn new(elements: [f32; 16]) -> Self {
        Self { elements }
    }

    pub fn compose_rows(row_0: Vec4, row_1: Vec4, row_2: Vec4, row_3: Vec4) -> Self {
        Self::new([
            row_0.x, row_0.y, row_0.z, row_0.w, //
            row_1.x, row_1.y, row_1.z, row_1.w, //
            row_2.x, row_2.y, row_2.z, row_2.w, //
            row_3.x, row_3.y, row_3.z, row_3.w, //
        ])
    }

    pub fn zero() -> Self {
        Self::new([
            0.0, 0.0, 0.0, 0.0, //
            0.0, 0.0, 0.0, 0.0, //
            0.0, 0.0, 0.0, 0.0, //
            0.0, 0.0, 0.0, 0.0, //
        ])
    }

    pub fn one() -> Self {
        Self::new([
            1.0, 1.0, 1.0, 1.0, //
            1.0, 1.0, 1.0, 1.0, //
            1.0, 1.0, 1.0, 1.0, //
            1.0, 1.0, 1.0, 1.0, //
        ])
    }

    pub fn identity() -> Self {
        Self::new([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ])
    }

    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::new([
            2.0 / (right - left),
            0.0,
            0.0,
            0.0, //
            0.0,
            2.0 / (top - bottom),
            0.0,
            0.0, //
            0.0,
            0.0,
            1.0 / (far - near),
            0.0, //
            (left + right) / (left - right),
            (bottom + top) / (bottom - top),
            near / (near - far),
            1.0, //
        ])
    }

    pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = (fov * 0.5).tan().recip();

        Self::new([
            f / aspect,
            0.0,
            0.0,
            0.0, //
            0.0,
            f,
            0.0,
            0.0, //
            0.0,
            0.0,
            (far + near) / (near - far),
            -1.0, //
            0.0,
            0.0,
            (2.0 * far * near) / (near - far),
            0.0, //
        ])
    }

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let z = (eye - target).normalized();
        let x = Vec3::cross(z, up).normalized();
        let y = Vec3::cross(x, z).normalized();

        Self::new([
            x.x,
            y.x,
            z.x,
            0.0, //
            x.y,
            y.y,
            z.y,
            0.0, //
            x.z,
            y.z,
            z.z,
            0.0, //
            Vec3::dot(-x, eye),
            Vec3::dot(-y, eye),
            Vec3::dot(-z, eye),
            1.0, //
        ])
    }

    pub fn trs(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let translate = Mat4::translation(position);
        let rotation = Mat4::rotation(rotation);
        let scale = Mat4::scale(scale);
        translate * rotation * scale
    }

    pub fn srt(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        let scale = Mat4::scale(scale);
        let rotation = Mat4::rotation(rotation);
        let translate = Mat4::translation(position);
        scale * rotation * translate
    }

    pub fn translation(position: Vec3) -> Self {
        Self::new([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            position.x, position.y, position.z, 1.0, //
        ])
    }

    pub fn rotation(rotation: Quat) -> Self {
        rotation.into_mat4()
    }

    pub fn scale(scale: Vec3) -> Self {
        Self::new([
            scale.x, 0.0, 0.0, 0.0, //
            0.0, scale.y, 0.0, 0.0, //
            0.0, 0.0, scale.z, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ])
    }

    pub fn row(&self, index: usize) -> Vec4 {
        Vec4::new(
            self.elements[index * 4 + 0],
            self.elements[index * 4 + 1],
            self.elements[index * 4 + 2],
            self.elements[index * 4 + 3],
        )
    }

    pub fn column(&self, index: usize) -> Vec4 {
        Vec4::new(
            self.elements[index + 0],
            self.elements[index + 4],
            self.elements[index + 8],
            self.elements[index + 12],
        )
    }

    pub fn split(&self) -> (Vec3, Quat, Vec3) {
        let position = self.row(3).into();
        let scale = Vec3::new(
            Vec3::new(self.elements[0], self.elements[1], self.elements[2]).len(),
            Vec3::new(self.elements[4], self.elements[5], self.elements[6]).len(),
            Vec3::new(self.elements[8], self.elements[9], self.elements[10]).len(),
        );
        let scale_removed = Mat4::compose_rows(
            Vec4::from_vec3(Vec3::from(self.row(0)) / scale.x, 0.0),
            Vec4::from_vec3(Vec3::from(self.row(1)) / scale.y, 0.0),
            Vec4::from_vec3(Vec3::from(self.row(2)) / scale.z, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        let rotation = Quat::from_mat4(&scale_removed);

        (position, rotation, scale)
    }

    pub fn determinant(&self) -> f32 {
        let a = self.elements[0 * 4 + 0];
        let b = self.elements[0 * 4 + 1];
        let c = self.elements[0 * 4 + 2];
        let d = self.elements[0 * 4 + 3];
        let e = self.elements[1 * 4 + 0];
        let f = self.elements[1 * 4 + 1];
        let g = self.elements[1 * 4 + 2];
        let h = self.elements[1 * 4 + 3];
        let i = self.elements[2 * 4 + 0];
        let j = self.elements[2 * 4 + 1];
        let k = self.elements[2 * 4 + 2];
        let l = self.elements[2 * 4 + 3];
        let m = self.elements[3 * 4 + 0];
        let n = self.elements[3 * 4 + 1];
        let o = self.elements[3 * 4 + 2];
        let p = self.elements[3 * 4 + 3];

        a * (f * (k * p - l * o) - g * (j * p - l * n) + h * (j * o - k * n))
            - b * (e * (k * p - l * o) - g * (i * p - l * m) + h * (i * o - k * m))
            + c * (e * (j * p - l * n) - f * (i * p - l * m) + h * (i * n - j * m))
            - d * (e * (j * o - k * n) - f * (i * o - k * m) + g * (i * n - j * m))
    }

    pub fn inverse(&mut self) -> &mut Self {
        let a = self.elements[0 * 4 + 0];
        let b = self.elements[0 * 4 + 1];
        let c = self.elements[0 * 4 + 2];
        let d = self.elements[0 * 4 + 3];
        let e = self.elements[1 * 4 + 0];
        let f = self.elements[1 * 4 + 1];
        let g = self.elements[1 * 4 + 2];
        let h = self.elements[1 * 4 + 3];
        let i = self.elements[2 * 4 + 0];
        let j = self.elements[2 * 4 + 1];
        let k = self.elements[2 * 4 + 2];
        let l = self.elements[2 * 4 + 3];
        let m = self.elements[3 * 4 + 0];
        let n = self.elements[3 * 4 + 1];
        let o = self.elements[3 * 4 + 2];
        let p = self.elements[3 * 4 + 3];

        let det = self.determinant();

        if det.abs() <= f32::EPSILON {
            return self;
        }

        let inv_det = det.recip();

        self.elements[0 * 4 + 0] =
            inv_det * (f * (k * p - l * o) - g * (j * p - l * n) + h * (j * o - k * n));
        self.elements[0 * 4 + 1] =
            -inv_det * (b * (k * p - l * o) - c * (j * p - l * n) + d * (j * o - k * n));
        self.elements[0 * 4 + 2] =
            inv_det * (b * (g * p - h * o) - c * (f * p - h * n) + d * (f * o - g * n));
        self.elements[0 * 4 + 3] =
            -inv_det * (b * (g * l - h * k) - c * (f * l - h * j) + d * (f * k - g * j));
        self.elements[1 * 4 + 0] =
            -inv_det * (e * (k * p - l * o) - g * (i * p - l * m) + h * (i * o - k * m));
        self.elements[1 * 4 + 1] =
            inv_det * (a * (k * p - l * o) - c * (i * p - l * m) + d * (i * o - k * m));
        self.elements[1 * 4 + 2] =
            -inv_det * (a * (g * p - h * o) - c * (e * p - h * m) + d * (e * o - g * m));
        self.elements[1 * 4 + 3] =
            inv_det * (a * (g * l - h * k) - c * (e * l - h * i) + d * (e * k - g * i));
        self.elements[2 * 4 + 0] =
            inv_det * (e * (j * p - l * n) - f * (i * p - l * m) + h * (i * n - j * m));
        self.elements[2 * 4 + 1] =
            -inv_det * (a * (j * p - l * n) - b * (i * p - l * m) + d * (i * n - j * m));
        self.elements[2 * 4 + 2] =
            inv_det * (a * (f * p - h * n) - b * (e * p - h * m) + d * (e * n - f * m));
        self.elements[2 * 4 + 3] =
            -inv_det * (a * (f * l - h * j) - b * (e * l - h * i) + d * (e * j - f * i));
        self.elements[3 * 4 + 0] =
            -inv_det * (e * (j * o - k * n) - f * (i * o - k * m) + g * (i * n - j * m));
        self.elements[3 * 4 + 1] =
            inv_det * (a * (j * o - k * n) - b * (i * o - k * m) + c * (i * n - j * m));
        self.elements[3 * 4 + 2] =
            -inv_det * (a * (f * o - g * n) - b * (e * o - g * m) + c * (e * n - f * m));
        self.elements[3 * 4 + 3] =
            inv_det * (a * (f * k - g * j) - b * (e * k - g * i) + c * (e * j - f * i));

        self
    }

    pub fn inversed(&self) -> Self {
        let mut result = self.clone();
        result.inverse();
        result
    }

    pub fn transpose(&mut self) -> &mut Self {
        let b = self.elements[0 * 4 + 1];
        let c = self.elements[0 * 4 + 2];
        let d = self.elements[0 * 4 + 3];
        let e = self.elements[1 * 4 + 0];
        let g = self.elements[1 * 4 + 2];
        let h = self.elements[1 * 4 + 3];
        let i = self.elements[2 * 4 + 0];
        let j = self.elements[2 * 4 + 1];
        let l = self.elements[2 * 4 + 3];
        let m = self.elements[3 * 4 + 0];
        let n = self.elements[3 * 4 + 1];
        let o = self.elements[3 * 4 + 2];

        self.elements[0 * 4 + 1] = e;
        self.elements[0 * 4 + 2] = i;
        self.elements[0 * 4 + 3] = m;
        self.elements[1 * 4 + 0] = b;
        self.elements[1 * 4 + 2] = j;
        self.elements[1 * 4 + 3] = n;
        self.elements[2 * 4 + 0] = c;
        self.elements[2 * 4 + 1] = g;
        self.elements[2 * 4 + 3] = o;
        self.elements[3 * 4 + 0] = d;
        self.elements[3 * 4 + 1] = h;
        self.elements[3 * 4 + 2] = l;

        self
    }

    pub fn transposed(&self) -> Self {
        let mut result = self.clone();
        result.transpose();
        result
    }
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Add for Mat4 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new([
            self.elements[0] + other.elements[0],
            self.elements[1] + other.elements[1],
            self.elements[2] + other.elements[2],
            self.elements[3] + other.elements[3],
            self.elements[4] + other.elements[4],
            self.elements[5] + other.elements[5],
            self.elements[6] + other.elements[6],
            self.elements[7] + other.elements[7],
            self.elements[8] + other.elements[8],
            self.elements[9] + other.elements[9],
            self.elements[10] + other.elements[10],
            self.elements[11] + other.elements[11],
            self.elements[12] + other.elements[12],
            self.elements[13] + other.elements[13],
            self.elements[14] + other.elements[14],
            self.elements[15] + other.elements[15],
        ])
    }
}

impl Add<&Mat4> for Mat4 {
    type Output = Self;

    fn add(self, other: &Mat4) -> Self {
        Self::new([
            self.elements[0] + other.elements[0],
            self.elements[1] + other.elements[1],
            self.elements[2] + other.elements[2],
            self.elements[3] + other.elements[3],
            self.elements[4] + other.elements[4],
            self.elements[5] + other.elements[5],
            self.elements[6] + other.elements[6],
            self.elements[7] + other.elements[7],
            self.elements[8] + other.elements[8],
            self.elements[9] + other.elements[9],
            self.elements[10] + other.elements[10],
            self.elements[11] + other.elements[11],
            self.elements[12] + other.elements[12],
            self.elements[13] + other.elements[13],
            self.elements[14] + other.elements[14],
            self.elements[15] + other.elements[15],
        ])
    }
}

impl AddAssign for Mat4 {
    fn add_assign(&mut self, rhs: Self) {
        self.elements[0] += rhs.elements[0];
        self.elements[1] += rhs.elements[1];
        self.elements[2] += rhs.elements[2];
        self.elements[3] += rhs.elements[3];
        self.elements[4] += rhs.elements[4];
        self.elements[5] += rhs.elements[5];
        self.elements[6] += rhs.elements[6];
        self.elements[7] += rhs.elements[7];
        self.elements[8] += rhs.elements[8];
        self.elements[9] += rhs.elements[9];
        self.elements[10] += rhs.elements[10];
        self.elements[11] += rhs.elements[11];
        self.elements[12] += rhs.elements[12];
        self.elements[13] += rhs.elements[13];
        self.elements[14] += rhs.elements[14];
        self.elements[15] += rhs.elements[15];
    }
}

impl AddAssign<&Mat4> for Mat4 {
    fn add_assign(&mut self, rhs: &Mat4) {
        self.elements[0] += rhs.elements[0];
        self.elements[1] += rhs.elements[1];
        self.elements[2] += rhs.elements[2];
        self.elements[3] += rhs.elements[3];
        self.elements[4] += rhs.elements[4];
        self.elements[5] += rhs.elements[5];
        self.elements[6] += rhs.elements[6];
        self.elements[7] += rhs.elements[7];
        self.elements[8] += rhs.elements[8];
        self.elements[9] += rhs.elements[9];
        self.elements[10] += rhs.elements[10];
        self.elements[11] += rhs.elements[11];
        self.elements[12] += rhs.elements[12];
        self.elements[13] += rhs.elements[13];
        self.elements[14] += rhs.elements[14];
        self.elements[15] += rhs.elements[15];
    }
}

impl Sub for Mat4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new([
            self.elements[0] - rhs.elements[0],
            self.elements[1] - rhs.elements[1],
            self.elements[2] - rhs.elements[2],
            self.elements[3] - rhs.elements[3],
            self.elements[4] - rhs.elements[4],
            self.elements[5] - rhs.elements[5],
            self.elements[6] - rhs.elements[6],
            self.elements[7] - rhs.elements[7],
            self.elements[8] - rhs.elements[8],
            self.elements[9] - rhs.elements[9],
            self.elements[10] - rhs.elements[10],
            self.elements[11] - rhs.elements[11],
            self.elements[12] - rhs.elements[12],
            self.elements[13] - rhs.elements[13],
            self.elements[14] - rhs.elements[14],
            self.elements[15] - rhs.elements[15],
        ])
    }
}

impl Sub<&Mat4> for Mat4 {
    type Output = Self;

    fn sub(self, rhs: &Mat4) -> Self::Output {
        Self::new([
            self.elements[0] - rhs.elements[0],
            self.elements[1] - rhs.elements[1],
            self.elements[2] - rhs.elements[2],
            self.elements[3] - rhs.elements[3],
            self.elements[4] - rhs.elements[4],
            self.elements[5] - rhs.elements[5],
            self.elements[6] - rhs.elements[6],
            self.elements[7] - rhs.elements[7],
            self.elements[8] - rhs.elements[8],
            self.elements[9] - rhs.elements[9],
            self.elements[10] - rhs.elements[10],
            self.elements[11] - rhs.elements[11],
            self.elements[12] - rhs.elements[12],
            self.elements[13] - rhs.elements[13],
            self.elements[14] - rhs.elements[14],
            self.elements[15] - rhs.elements[15],
        ])
    }
}

impl SubAssign for Mat4 {
    fn sub_assign(&mut self, rhs: Self) {
        self.elements[0] -= rhs.elements[0];
        self.elements[1] -= rhs.elements[1];
        self.elements[2] -= rhs.elements[2];
        self.elements[3] -= rhs.elements[3];
        self.elements[4] -= rhs.elements[4];
        self.elements[5] -= rhs.elements[5];
        self.elements[6] -= rhs.elements[6];
        self.elements[7] -= rhs.elements[7];
        self.elements[8] -= rhs.elements[8];
        self.elements[9] -= rhs.elements[9];
        self.elements[10] -= rhs.elements[10];
        self.elements[11] -= rhs.elements[11];
        self.elements[12] -= rhs.elements[12];
        self.elements[13] -= rhs.elements[13];
        self.elements[14] -= rhs.elements[14];
        self.elements[15] -= rhs.elements[15];
    }
}

impl SubAssign<&Mat4> for Mat4 {
    fn sub_assign(&mut self, rhs: &Mat4) {
        self.elements[0] -= rhs.elements[0];
        self.elements[1] -= rhs.elements[1];
        self.elements[2] -= rhs.elements[2];
        self.elements[3] -= rhs.elements[3];
        self.elements[4] -= rhs.elements[4];
        self.elements[5] -= rhs.elements[5];
        self.elements[6] -= rhs.elements[6];
        self.elements[7] -= rhs.elements[7];
        self.elements[8] -= rhs.elements[8];
        self.elements[9] -= rhs.elements[9];
        self.elements[10] -= rhs.elements[10];
        self.elements[11] -= rhs.elements[11];
        self.elements[12] -= rhs.elements[12];
        self.elements[13] -= rhs.elements[13];
        self.elements[14] -= rhs.elements[14];
        self.elements[15] -= rhs.elements[15];
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new([
            self.elements[0] * rhs.elements[0]
                + self.elements[1] * rhs.elements[4]
                + self.elements[2] * rhs.elements[8]
                + self.elements[3] * rhs.elements[12],
            self.elements[0] * rhs.elements[1]
                + self.elements[1] * rhs.elements[5]
                + self.elements[2] * rhs.elements[9]
                + self.elements[3] * rhs.elements[13],
            self.elements[0] * rhs.elements[2]
                + self.elements[1] * rhs.elements[6]
                + self.elements[2] * rhs.elements[10]
                + self.elements[3] * rhs.elements[14],
            self.elements[0] * rhs.elements[3]
                + self.elements[1] * rhs.elements[7]
                + self.elements[2] * rhs.elements[11]
                + self.elements[3] * rhs.elements[15],
            self.elements[4] * rhs.elements[0]
                + self.elements[5] * rhs.elements[4]
                + self.elements[6] * rhs.elements[8]
                + self.elements[7] * rhs.elements[12],
            self.elements[4] * rhs.elements[1]
                + self.elements[5] * rhs.elements[5]
                + self.elements[6] * rhs.elements[9]
                + self.elements[7] * rhs.elements[13],
            self.elements[4] * rhs.elements[2]
                + self.elements[5] * rhs.elements[6]
                + self.elements[6] * rhs.elements[10]
                + self.elements[7] * rhs.elements[14],
            self.elements[4] * rhs.elements[3]
                + self.elements[5] * rhs.elements[7]
                + self.elements[6] * rhs.elements[11]
                + self.elements[7] * rhs.elements[15],
            self.elements[8] * rhs.elements[0]
                + self.elements[9] * rhs.elements[4]
                + self.elements[10] * rhs.elements[8]
                + self.elements[11] * rhs.elements[12],
            self.elements[8] * rhs.elements[1]
                + self.elements[9] * rhs.elements[5]
                + self.elements[10] * rhs.elements[9]
                + self.elements[11] * rhs.elements[13],
            self.elements[8] * rhs.elements[2]
                + self.elements[9] * rhs.elements[6]
                + self.elements[10] * rhs.elements[10]
                + self.elements[11] * rhs.elements[14],
            self.elements[8] * rhs.elements[3]
                + self.elements[9] * rhs.elements[7]
                + self.elements[10] * rhs.elements[11]
                + self.elements[11] * rhs.elements[15],
            self.elements[12] * rhs.elements[0]
                + self.elements[13] * rhs.elements[4]
                + self.elements[14] * rhs.elements[8]
                + self.elements[15] * rhs.elements[12],
            self.elements[12] * rhs.elements[1]
                + self.elements[13] * rhs.elements[5]
                + self.elements[14] * rhs.elements[9]
                + self.elements[15] * rhs.elements[13],
            self.elements[12] * rhs.elements[2]
                + self.elements[13] * rhs.elements[6]
                + self.elements[14] * rhs.elements[10]
                + self.elements[15] * rhs.elements[14],
            self.elements[12] * rhs.elements[3]
                + self.elements[13] * rhs.elements[7]
                + self.elements[14] * rhs.elements[11]
                + self.elements[15] * rhs.elements[15],
        ])
    }
}

impl Mul<Mat4> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        Mat4::new([
            self.elements[0] * rhs.elements[0]
                + self.elements[1] * rhs.elements[4]
                + self.elements[2] * rhs.elements[8]
                + self.elements[3] * rhs.elements[12],
            self.elements[0] * rhs.elements[1]
                + self.elements[1] * rhs.elements[5]
                + self.elements[2] * rhs.elements[9]
                + self.elements[3] * rhs.elements[13],
            self.elements[0] * rhs.elements[2]
                + self.elements[1] * rhs.elements[6]
                + self.elements[2] * rhs.elements[10]
                + self.elements[3] * rhs.elements[14],
            self.elements[0] * rhs.elements[3]
                + self.elements[1] * rhs.elements[7]
                + self.elements[2] * rhs.elements[11]
                + self.elements[3] * rhs.elements[15],
            self.elements[4] * rhs.elements[0]
                + self.elements[5] * rhs.elements[4]
                + self.elements[6] * rhs.elements[8]
                + self.elements[7] * rhs.elements[12],
            self.elements[4] * rhs.elements[1]
                + self.elements[5] * rhs.elements[5]
                + self.elements[6] * rhs.elements[9]
                + self.elements[7] * rhs.elements[13],
            self.elements[4] * rhs.elements[2]
                + self.elements[5] * rhs.elements[6]
                + self.elements[6] * rhs.elements[10]
                + self.elements[7] * rhs.elements[14],
            self.elements[4] * rhs.elements[3]
                + self.elements[5] * rhs.elements[7]
                + self.elements[6] * rhs.elements[11]
                + self.elements[7] * rhs.elements[15],
            self.elements[8] * rhs.elements[0]
                + self.elements[9] * rhs.elements[4]
                + self.elements[10] * rhs.elements[8]
                + self.elements[11] * rhs.elements[12],
            self.elements[8] * rhs.elements[1]
                + self.elements[9] * rhs.elements[5]
                + self.elements[10] * rhs.elements[9]
                + self.elements[11] * rhs.elements[13],
            self.elements[8] * rhs.elements[2]
                + self.elements[9] * rhs.elements[6]
                + self.elements[10] * rhs.elements[10]
                + self.elements[11] * rhs.elements[14],
            self.elements[8] * rhs.elements[3]
                + self.elements[9] * rhs.elements[7]
                + self.elements[10] * rhs.elements[11]
                + self.elements[11] * rhs.elements[15],
            self.elements[12] * rhs.elements[0]
                + self.elements[13] * rhs.elements[4]
                + self.elements[14] * rhs.elements[8]
                + self.elements[15] * rhs.elements[12],
            self.elements[12] * rhs.elements[1]
                + self.elements[13] * rhs.elements[5]
                + self.elements[14] * rhs.elements[9]
                + self.elements[15] * rhs.elements[13],
            self.elements[12] * rhs.elements[2]
                + self.elements[13] * rhs.elements[6]
                + self.elements[14] * rhs.elements[10]
                + self.elements[15] * rhs.elements[14],
            self.elements[12] * rhs.elements[3]
                + self.elements[13] * rhs.elements[7]
                + self.elements[14] * rhs.elements[11]
                + self.elements[15] * rhs.elements[15],
        ])
    }
}

impl Mul<&Self> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self::Output {
        Self::new([
            self.elements[0] * rhs.elements[0]
                + self.elements[1] * rhs.elements[4]
                + self.elements[2] * rhs.elements[8]
                + self.elements[3] * rhs.elements[12],
            self.elements[0] * rhs.elements[1]
                + self.elements[1] * rhs.elements[5]
                + self.elements[2] * rhs.elements[9]
                + self.elements[3] * rhs.elements[13],
            self.elements[0] * rhs.elements[2]
                + self.elements[1] * rhs.elements[6]
                + self.elements[2] * rhs.elements[10]
                + self.elements[3] * rhs.elements[14],
            self.elements[0] * rhs.elements[3]
                + self.elements[1] * rhs.elements[7]
                + self.elements[2] * rhs.elements[11]
                + self.elements[3] * rhs.elements[15],
            self.elements[4] * rhs.elements[0]
                + self.elements[5] * rhs.elements[4]
                + self.elements[6] * rhs.elements[8]
                + self.elements[7] * rhs.elements[12],
            self.elements[4] * rhs.elements[1]
                + self.elements[5] * rhs.elements[5]
                + self.elements[6] * rhs.elements[9]
                + self.elements[7] * rhs.elements[13],
            self.elements[4] * rhs.elements[2]
                + self.elements[5] * rhs.elements[6]
                + self.elements[6] * rhs.elements[10]
                + self.elements[7] * rhs.elements[14],
            self.elements[4] * rhs.elements[3]
                + self.elements[5] * rhs.elements[7]
                + self.elements[6] * rhs.elements[11]
                + self.elements[7] * rhs.elements[15],
            self.elements[8] * rhs.elements[0]
                + self.elements[9] * rhs.elements[4]
                + self.elements[10] * rhs.elements[8]
                + self.elements[11] * rhs.elements[12],
            self.elements[8] * rhs.elements[1]
                + self.elements[9] * rhs.elements[5]
                + self.elements[10] * rhs.elements[9]
                + self.elements[11] * rhs.elements[13],
            self.elements[8] * rhs.elements[2]
                + self.elements[9] * rhs.elements[6]
                + self.elements[10] * rhs.elements[10]
                + self.elements[11] * rhs.elements[14],
            self.elements[8] * rhs.elements[3]
                + self.elements[9] * rhs.elements[7]
                + self.elements[10] * rhs.elements[11]
                + self.elements[11] * rhs.elements[15],
            self.elements[12] * rhs.elements[0]
                + self.elements[13] * rhs.elements[4]
                + self.elements[14] * rhs.elements[8]
                + self.elements[15] * rhs.elements[12],
            self.elements[12] * rhs.elements[1]
                + self.elements[13] * rhs.elements[5]
                + self.elements[14] * rhs.elements[9]
                + self.elements[15] * rhs.elements[13],
            self.elements[12] * rhs.elements[2]
                + self.elements[13] * rhs.elements[6]
                + self.elements[14] * rhs.elements[10]
                + self.elements[15] * rhs.elements[14],
            self.elements[12] * rhs.elements[3]
                + self.elements[13] * rhs.elements[7]
                + self.elements[14] * rhs.elements[11]
                + self.elements[15] * rhs.elements[15],
        ])
    }
}

impl Mul<&Self> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: &Self) -> Self::Output {
        Mat4::new([
            self.elements[0] * rhs.elements[0]
                + self.elements[1] * rhs.elements[4]
                + self.elements[2] * rhs.elements[8]
                + self.elements[3] * rhs.elements[12],
            self.elements[0] * rhs.elements[1]
                + self.elements[1] * rhs.elements[5]
                + self.elements[2] * rhs.elements[9]
                + self.elements[3] * rhs.elements[13],
            self.elements[0] * rhs.elements[2]
                + self.elements[1] * rhs.elements[6]
                + self.elements[2] * rhs.elements[10]
                + self.elements[3] * rhs.elements[14],
            self.elements[0] * rhs.elements[3]
                + self.elements[1] * rhs.elements[7]
                + self.elements[2] * rhs.elements[11]
                + self.elements[3] * rhs.elements[15],
            self.elements[4] * rhs.elements[0]
                + self.elements[5] * rhs.elements[4]
                + self.elements[6] * rhs.elements[8]
                + self.elements[7] * rhs.elements[12],
            self.elements[4] * rhs.elements[1]
                + self.elements[5] * rhs.elements[5]
                + self.elements[6] * rhs.elements[9]
                + self.elements[7] * rhs.elements[13],
            self.elements[4] * rhs.elements[2]
                + self.elements[5] * rhs.elements[6]
                + self.elements[6] * rhs.elements[10]
                + self.elements[7] * rhs.elements[14],
            self.elements[4] * rhs.elements[3]
                + self.elements[5] * rhs.elements[7]
                + self.elements[6] * rhs.elements[11]
                + self.elements[7] * rhs.elements[15],
            self.elements[8] * rhs.elements[0]
                + self.elements[9] * rhs.elements[4]
                + self.elements[10] * rhs.elements[8]
                + self.elements[11] * rhs.elements[12],
            self.elements[8] * rhs.elements[1]
                + self.elements[9] * rhs.elements[5]
                + self.elements[10] * rhs.elements[9]
                + self.elements[11] * rhs.elements[13],
            self.elements[8] * rhs.elements[2]
                + self.elements[9] * rhs.elements[6]
                + self.elements[10] * rhs.elements[10]
                + self.elements[11] * rhs.elements[14],
            self.elements[8] * rhs.elements[3]
                + self.elements[9] * rhs.elements[7]
                + self.elements[10] * rhs.elements[11]
                + self.elements[11] * rhs.elements[15],
            self.elements[12] * rhs.elements[0]
                + self.elements[13] * rhs.elements[4]
                + self.elements[14] * rhs.elements[8]
                + self.elements[15] * rhs.elements[12],
            self.elements[12] * rhs.elements[1]
                + self.elements[13] * rhs.elements[5]
                + self.elements[14] * rhs.elements[9]
                + self.elements[15] * rhs.elements[13],
            self.elements[12] * rhs.elements[2]
                + self.elements[13] * rhs.elements[6]
                + self.elements[14] * rhs.elements[10]
                + self.elements[15] * rhs.elements[14],
            self.elements[12] * rhs.elements[3]
                + self.elements[13] * rhs.elements[7]
                + self.elements[14] * rhs.elements[11]
                + self.elements[15] * rhs.elements[15],
        ])
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(
            self.elements[0] * rhs.x
                + self.elements[1] * rhs.y
                + self.elements[2] * rhs.z
                + self.elements[3] * rhs.w,
            self.elements[4] * rhs.x
                + self.elements[5] * rhs.y
                + self.elements[6] * rhs.z
                + self.elements[7] * rhs.w,
            self.elements[8] * rhs.x
                + self.elements[9] * rhs.y
                + self.elements[10] * rhs.z
                + self.elements[11] * rhs.w,
            self.elements[12] * rhs.x
                + self.elements[13] * rhs.y
                + self.elements[14] * rhs.z
                + self.elements[15] * rhs.w,
        )
    }
}

impl Mul<Vec4> for &Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4::new(
            self.elements[0] * rhs.x
                + self.elements[1] * rhs.y
                + self.elements[2] * rhs.z
                + self.elements[3] * rhs.w,
            self.elements[4] * rhs.x
                + self.elements[5] * rhs.y
                + self.elements[6] * rhs.z
                + self.elements[7] * rhs.w,
            self.elements[8] * rhs.x
                + self.elements[9] * rhs.y
                + self.elements[10] * rhs.z
                + self.elements[11] * rhs.w,
            self.elements[12] * rhs.x
                + self.elements[13] * rhs.y
                + self.elements[14] * rhs.z
                + self.elements[15] * rhs.w,
        )
    }
}

impl Mul<Mat4> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: Mat4) -> Self::Output {
        Vec4::new(
            self.x * rhs.elements[0]
                + self.y * rhs.elements[4]
                + self.z * rhs.elements[8]
                + self.w * rhs.elements[12],
            self.x * rhs.elements[1]
                + self.y * rhs.elements[5]
                + self.z * rhs.elements[9]
                + self.w * rhs.elements[13],
            self.x * rhs.elements[2]
                + self.y * rhs.elements[6]
                + self.z * rhs.elements[10]
                + self.w * rhs.elements[14],
            self.x * rhs.elements[3]
                + self.y * rhs.elements[7]
                + self.z * rhs.elements[11]
                + self.w * rhs.elements[15],
        )
    }
}

impl Mul<&Mat4> for Vec4 {
    type Output = Self;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        Vec4::new(
            self.x * rhs.elements[0]
                + self.y * rhs.elements[4]
                + self.z * rhs.elements[8]
                + self.w * rhs.elements[12],
            self.x * rhs.elements[1]
                + self.y * rhs.elements[5]
                + self.z * rhs.elements[9]
                + self.w * rhs.elements[13],
            self.x * rhs.elements[2]
                + self.y * rhs.elements[6]
                + self.z * rhs.elements[10]
                + self.w * rhs.elements[14],
            self.x * rhs.elements[3]
                + self.y * rhs.elements[7]
                + self.z * rhs.elements[11]
                + self.w * rhs.elements[15],
        )
    }
}

impl Mul<f32> for Mat4 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new([
            self.elements[0] * rhs,
            self.elements[1] * rhs,
            self.elements[2] * rhs,
            self.elements[3] * rhs,
            self.elements[4] * rhs,
            self.elements[5] * rhs,
            self.elements[6] * rhs,
            self.elements[7] * rhs,
            self.elements[8] * rhs,
            self.elements[9] * rhs,
            self.elements[10] * rhs,
            self.elements[11] * rhs,
            self.elements[12] * rhs,
            self.elements[13] * rhs,
            self.elements[14] * rhs,
            self.elements[15] * rhs,
        ])
    }
}

impl Mul<f32> for &Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: f32) -> Self::Output {
        Mat4::new([
            self.elements[0] * rhs,
            self.elements[1] * rhs,
            self.elements[2] * rhs,
            self.elements[3] * rhs,
            self.elements[4] * rhs,
            self.elements[5] * rhs,
            self.elements[6] * rhs,
            self.elements[7] * rhs,
            self.elements[8] * rhs,
            self.elements[9] * rhs,
            self.elements[10] * rhs,
            self.elements[11] * rhs,
            self.elements[12] * rhs,
            self.elements[13] * rhs,
            self.elements[14] * rhs,
            self.elements[15] * rhs,
        ])
    }
}

impl Mul<Mat4> for f32 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Self::Output {
        Mat4::new([
            self * rhs.elements[0],
            self * rhs.elements[1],
            self * rhs.elements[2],
            self * rhs.elements[3],
            self * rhs.elements[4],
            self * rhs.elements[5],
            self * rhs.elements[6],
            self * rhs.elements[7],
            self * rhs.elements[8],
            self * rhs.elements[9],
            self * rhs.elements[10],
            self * rhs.elements[11],
            self * rhs.elements[12],
            self * rhs.elements[13],
            self * rhs.elements[14],
            self * rhs.elements[15],
        ])
    }
}

impl Mul<&Mat4> for f32 {
    type Output = Mat4;

    fn mul(self, rhs: &Mat4) -> Self::Output {
        Mat4::new([
            self * rhs.elements[0],
            self * rhs.elements[1],
            self * rhs.elements[2],
            self * rhs.elements[3],
            self * rhs.elements[4],
            self * rhs.elements[5],
            self * rhs.elements[6],
            self * rhs.elements[7],
            self * rhs.elements[8],
            self * rhs.elements[9],
            self * rhs.elements[10],
            self * rhs.elements[11],
            self * rhs.elements[12],
            self * rhs.elements[13],
            self * rhs.elements[14],
            self * rhs.elements[15],
        ])
    }
}

impl MulAssign for Mat4 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl MulAssign<&Self> for Mat4 {
    fn mul_assign(&mut self, rhs: &Self) {
        *self = self.clone() * rhs;
    }
}

impl MulAssign<Mat4> for Vec4 {
    fn mul_assign(&mut self, rhs: Mat4) {
        *self = *self * rhs;
    }
}

impl MulAssign<&Mat4> for Vec4 {
    fn mul_assign(&mut self, rhs: &Mat4) {
        *self = *self * rhs;
    }
}

impl MulAssign<f32> for Mat4 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = self.clone() * rhs;
    }
}

impl Div<f32> for Mat4 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self::new([
            self.elements[0] / rhs,
            self.elements[1] / rhs,
            self.elements[2] / rhs,
            self.elements[3] / rhs,
            self.elements[4] / rhs,
            self.elements[5] / rhs,
            self.elements[6] / rhs,
            self.elements[7] / rhs,
            self.elements[8] / rhs,
            self.elements[9] / rhs,
            self.elements[10] / rhs,
            self.elements[11] / rhs,
            self.elements[12] / rhs,
            self.elements[13] / rhs,
            self.elements[14] / rhs,
            self.elements[15] / rhs,
        ])
    }
}

impl Neg for Mat4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new([
            -self.elements[0],
            -self.elements[1],
            -self.elements[2],
            -self.elements[3],
            -self.elements[4],
            -self.elements[5],
            -self.elements[6],
            -self.elements[7],
            -self.elements[8],
            -self.elements[9],
            -self.elements[10],
            -self.elements[11],
            -self.elements[12],
            -self.elements[13],
            -self.elements[14],
            -self.elements[15],
        ])
    }
}

impl Neg for &Mat4 {
    type Output = Mat4;

    fn neg(self) -> Self::Output {
        Mat4::new([
            -self.elements[0],
            -self.elements[1],
            -self.elements[2],
            -self.elements[3],
            -self.elements[4],
            -self.elements[5],
            -self.elements[6],
            -self.elements[7],
            -self.elements[8],
            -self.elements[9],
            -self.elements[10],
            -self.elements[11],
            -self.elements[12],
            -self.elements[13],
            -self.elements[14],
            -self.elements[15],
        ])
    }
}

impl Display for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Mat4([0]={}, [1]={}, [2]={}, [3]={}, [4]={}, [5]={}, [6]={}, [7]={}, [8]={}, [9]={}, [10]={}, [11]={}, [12]={}, [13]={}, [14]={}, [15]={})",
            self.elements[0],
            self.elements[1],
            self.elements[2],
            self.elements[3],
            self.elements[4],
            self.elements[5],
            self.elements[6],
            self.elements[7],
            self.elements[8],
            self.elements[9],
            self.elements[10],
            self.elements[11],
            self.elements[12],
            self.elements[13],
            self.elements[14],
            self.elements[15],
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn equals_float(a: f32, b: f32) -> bool {
        (a - b).abs() <= f32::EPSILON
    }

    fn equals_mat4(a: &Mat4, b: &Mat4) -> bool {
        for index in 0..16 {
            if !equals_float(a.elements[index], b.elements[index]) {
                return false;
            }
        }

        true
    }

    #[test]
    fn check_determiant_and_inverse_0() {
        let m = Mat4::new([
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ]);
        let det = m.determinant();
        let inv = m.inversed();

        assert!(equals_float(det, 1.0));
        assert!(equals_mat4(
            &inv,
            &Mat4::new([
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                0.0, 0.0, 0.0, 1.0, //
            ])
        ));
    }

    #[test]
    fn check_determiant_and_inverse_1() {
        let m = Mat4::new([
            2.0, 0.0, 0.0, 0.0, //
            0.0, 3.0, 0.0, 0.0, //
            0.0, 0.0, 4.0, 0.0, //
            0.0, 0.0, 0.0, 5.0, //
        ]);
        let det = m.determinant();
        let inv = m.inversed();

        assert!(equals_float(det, 120.0));
        assert!(equals_mat4(
            &inv,
            &Mat4::new([
                0.5,
                0.0,
                0.0,
                0.0,
                0.0,
                0.3333333333333333,
                0.0,
                0.0,
                0.0,
                0.0,
                0.25,
                0.0,
                0.0,
                0.0,
                0.0,
                0.2,
            ])
        ));
    }

    #[test]
    fn check_determiant_and_inverse_2() {
        let m = Mat4::new([
            1.0, 2.0, 1.0, 3.0, //
            0.0, 1.0, 4.0, 2.0, //
            2.0, 1.0, 2.0, 1.0, //
            3.0, 0.0, 1.0, 2.0, //
        ]);
        let det = m.determinant();
        let inv = m.inversed();

        assert!(equals_float(det, 38.0));
        assert!(equals_mat4(
            &inv,
            &Mat4::new([
                -3.0 / 38.0,
                -7.0 / 38.0,
                13.0 / 38.0,
                5.0 / 38.0,
                12.0 / 38.0,
                -10.0 / 38.0,
                24.0 / 38.0,
                -20.0 / 38.0,
                -7.0 / 38.0,
                9.0 / 38.0,
                5.0 / 38.0,
                -1.0 / 38.0,
                8.0 / 38.0,
                6.0 / 38.0,
                -22.0 / 38.0,
                12.0 / 38.0,
            ])
        ));
    }

    #[test]
    fn check_determiant_and_inverse_3() {
        let m = Mat4::new([
            4.0, -2.0, 2.0, -3.0, //
            -2.0, 1.0, -1.0, 1.0, //
            1.0, -1.0, 3.0, -2.0, //
            2.0, -1.0, 2.0, -2.0, //
        ]);
        let det = m.determinant();
        let inv = m.inversed();

        assert!(equals_float(det, 1.0));
        assert!(equals_mat4(
            &inv,
            &Mat4::new([
                -1.0, -1.0, -1.0, 2.0, //
                -2.0, 0.0, -2.0, 5.0, //
                -1.0, -1.0, 0.0, 1.0, //
                -1.0, -2.0, 0.0, 0.0, //
            ])
        ));
    }
}

#![allow(dead_code)]

use std::ops::{self, Deref};

use bytemuck::{Pod, Zeroable};

/*     RGB<T>    */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rgb<T>(pub [T; 3]);

impl<T> Deref for Rgb<T> {
    type Target = [T; 3];

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

impl<T: Default> Default for Rgb<T> {
    fn default() -> Self {
        Self([T::default(), T::default(), T::default()])
    }
}

unsafe impl<T: Zeroable> Zeroable for Rgb<T> {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

unsafe impl<T: Pod> Pod for Rgb<T> {}

impl<T: Copy> From<&Rgba<T>> for Rgb<T> {
    fn from(value: &Rgba<T>) -> Self {
        let rgb: [T; 3] = value.0[..3].try_into().unwrap();
        Rgb(rgb)
    }
}

impl Rgb<u8> {
    pub const RED: Rgb<u8> = Rgb([255, 0, 0]);
    pub const GREEN: Rgb<u8> = Rgb([0, 255, 0]);
    pub const BLUE: Rgb<u8> = Rgb([0, 255, 0]);
    pub const YELLOW: Rgb<u8> = Rgb([255, 255, 0]);
    pub const MAGENTA: Rgb<u8> = Rgb([255, 0, 255]);
    pub const CYAN: Rgb<u8> = Rgb([0, 255, 255]);
    pub const WHITE: Rgb<u8> = Rgb([255, 255, 255]);
    pub const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
    pub const GRAY: Rgb<u8> = Rgb([127, 127, 127]);
}

impl Rgb<f32> {
    pub const RED: Rgb<f32> = Rgb([1.0, 0.0, 0.0]);
    pub const GREEN: Rgb<f32> = Rgb([0.0, 1.0, 0.0]);
    pub const BLUE: Rgb<f32> = Rgb([0.0, 1.0, 0.0]);
    pub const YELLOW: Rgb<f32> = Rgb([1.0, 1.0, 0.0]);
    pub const MAGENTA: Rgb<f32> = Rgb([1.0, 0.0, 1.0]);
    pub const CYAN: Rgb<f32> = Rgb([0.0, 1.0, 1.0]);
    pub const WHITE: Rgb<f32> = Rgb([1.0, 1.0, 1.0]);
    pub const BLACK: Rgb<f32> = Rgb([0.0, 0.0, 0.0]);
    pub const GRAY: Rgb<f32> = Rgb([0.5, 0.5, 0.5]);
}

impl From<Rgb<f32>> for Rgb<u8> {
    fn from(value: Rgb<f32>) -> Self {
        Self([
            (value.0[0] * 255.0) as u8,
            (value.0[1] * 255.0) as u8,
            (value.0[2] * 255.0) as u8,
        ])
    }
}

impl From<Rgb<u8>> for Rgb<f32> {
    fn from(value: Rgb<u8>) -> Self {
        Self([
            (value.0[0] as f32 / 255.0),
            (value.0[1] as f32 / 255.0),
            (value.0[2] as f32 / 255.0),
        ])
    }
}

impl Rgb<f32> {
    pub fn blend(&self, other: &Rgb<f32>, t: f32) -> Rgb<f32> {
        let r = (1.0 - t) * self.0[0] + t * other.0[0];
        let g = (1.0 - t) * self.0[1] + t * other.0[1];
        let b = (1.0 - t) * self.0[2] + t * other.0[2];

        Rgb([r, g, b])
    }
}

impl ops::Mul for Rgb<f32> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ])
    }
}

impl ops::MulAssign for Rgb<f32> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0[0] *= rhs.0[0];
        self.0[1] *= rhs.0[1];
        self.0[2] *= rhs.0[2];
    }
}

/*     RGBA<T>    */
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rgba<T>(pub [T; 4]);

impl<T> Deref for Rgba<T> {
    type Target = [T; 4];

    fn deref(&'_ self) -> &'_ Self::Target {
        &self.0
    }
}

impl<T: Default> Default for Rgba<T> {
    fn default() -> Self {
        Self([T::default(), T::default(), T::default(), T::default()])
    }
}

unsafe impl<T: Zeroable> Zeroable for Rgba<T> {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

unsafe impl<T: Pod> Pod for Rgba<T> {}

impl<T: Copy> Rgba<T> {
    pub fn from_rgb(value: &Rgb<T>, alpha: T) -> Self {
        let value: [T; 3] = value.0.to_owned();
        Rgba([value[0], value[1], value[2], alpha])
    }
}

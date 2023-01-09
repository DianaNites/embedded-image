//! Embedded image handling library
#![no_std]
#![allow(unused_imports, dead_code)]
extern crate alloc;

use crate::transforms::*;
use alloc::{vec, vec::Vec};
use core::slice::from_raw_parts;
use na::{Matrix3x1, Matrix4x1};
use nalgebra as na;

mod transforms;

pub type XY = (u32, u32);
pub type ResXY = (u32, u32);
pub type FloatXY = (f32, f32);
pub type RawPixel = [u8; 4];
pub type WorkPixel = [f32; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ColorSpace {
    /// sRGB data
    sRGB,

    /// Linear sRGB data
    sRGBLinear,

    /// So called "simple" sRGB, with a flat gamma of 2.2
    SimplesRGB,

    /// Display P3 / P3-D65
    DisplayP3,

    /// No color-space / "display" / "as-is" / "web"
    ///
    /// This just uses the RGB values as-is, no conversions,
    /// whatever happens, happens.
    ///
    /// Converting TO or FROM this profile has NO EFFECT beyond changing
    /// the color profile
    AsIs,
}

#[derive(Debug)]
pub struct Image {
    data: Vec<WorkPixel>,
    res: ResXY,
    color: ColorSpace,
}

impl Image {
    /// Read an Image from an array of pixel data of length `width * height * 4`
    ///
    /// Pixels are assumed to be in the order RGBA, 8 bits per channel
    ///
    /// Pixels will be cast as `f32` and divided by 255.
    ///
    /// # Panics
    ///
    /// - If `data` is not exactly `width * height * 4` in size
    pub fn from_bytes(data: &[u8], res: ResXY, color: ColorSpace) -> Self {
        let (width, height) = res;
        assert_eq!(data.len(), (width * height * 4) as usize);

        let data = unsafe {
            let len = (width * height) as usize;
            let data = data.as_ptr() as *const RawPixel;

            from_raw_parts(data, len)
                .iter()
                .map(|f| f.map(|f| f as f32 / 255.))
                .collect()
        };
        Self { data, res, color }
    }

    pub fn width(&self) -> u32 {
        self.res.0
    }

    pub fn height(&self) -> u32 {
        self.res.1
    }

    pub fn color(&self) -> ColorSpace {
        self.color
    }

    pub fn pixels(&self) -> &[WorkPixel] {
        &self.data
    }

    // TODO: Rendering intents?
    // jfc it really set out to write a uefi stub
    // and is now learning about color and writing an no_std image library huh
    // insane
    pub fn to_color(&mut self, color: ColorSpace) {
        for p in &mut self.data {
            let mut q = Matrix3x1::from_row_slice(&p[..3]);
            // TODO: ugh this doesn't need to be in the loop but it doesn't feel like moving it right now
            match (self.color, color) {
                (ColorSpace::sRGB, ColorSpace::sRGBLinear) => q = q.map(srgb_to_rgb),
                (ColorSpace::sRGB, ColorSpace::SimplesRGB) => {
                    q = q.map(srgb_to_rgb).map(rgb_to_gamma)
                }
                (ColorSpace::sRGB, ColorSpace::DisplayP3) => todo!(),

                (ColorSpace::sRGBLinear, ColorSpace::sRGB) => q = q.map(rgb_to_srgb),
                (ColorSpace::sRGBLinear, ColorSpace::DisplayP3) => todo!(),
                (ColorSpace::sRGBLinear, ColorSpace::SimplesRGB) => q = q.map(rgb_to_gamma),

                (ColorSpace::SimplesRGB, ColorSpace::sRGBLinear) => todo!(),
                (ColorSpace::SimplesRGB, ColorSpace::sRGB) => {
                    q = q.map(gamma_to_rgb).map(rgb_to_srgb)
                }
                (ColorSpace::SimplesRGB, ColorSpace::DisplayP3) => todo!(),

                (ColorSpace::DisplayP3, ColorSpace::sRGB) => todo!(),
                (ColorSpace::DisplayP3, ColorSpace::sRGBLinear) => todo!(),
                (ColorSpace::DisplayP3, ColorSpace::SimplesRGB) => todo!(),

                (ColorSpace::sRGB, ColorSpace::sRGB) => (),
                (ColorSpace::sRGBLinear, ColorSpace::sRGBLinear) => (),
                (ColorSpace::SimplesRGB, ColorSpace::SimplesRGB) => todo!(),
                (ColorSpace::DisplayP3, ColorSpace::DisplayP3) => (),

                (_, ColorSpace::AsIs) => (),
                (ColorSpace::AsIs, _) => (),
            }
            *p = [q[0], q[1], q[2], p[3]];
        }
        self.color = color;
    }

    pub fn scale(&mut self, new: ResXY) {
        let width = self.width();
        let height = self.height();
        let (new_width, new_height) = (new.0, new.1);
        if (width, height) == (new_width, new_height) {
            return;
        }
        let x_scale = (new_width - 1) as f32 / (width - 1) as f32;
        let y_scale = (new_height - 1) as f32 / (height - 1) as f32;

        let pixels = self.pixels();
        let mut out: Vec<WorkPixel> = vec![Default::default(); (new_height * new_width) as usize];

        for y in 0..new_height {
            for x in 0..new_width {
                let res = bilinear((x, y), (x_scale, y_scale), (width, height), pixels);

                let index = ((y * new_width) + x) as usize;
                out[index] = res;
            }
        }
        self.data = out;
        self.res = new;
    }
}

#[allow(unused_variables)]
fn bilinear(xy: XY, scale: FloatXY, src: ResXY, pixels: &[WorkPixel]) -> WorkPixel {
    todo!("fuck this")
}

/// Helper for no_std float methods
pub trait F32 {
    fn powf(self, n: f32) -> f32;

    fn round(self) -> f32;

    fn floor(self) -> f32;

    fn ceil(self) -> f32;
}

impl F32 for f32 {
    #[inline]
    fn powf(self, n: f32) -> f32 {
        libm::powf(self, n)
    }

    #[inline]
    fn round(self) -> f32 {
        libm::roundf(self)
    }

    #[inline]
    fn floor(self) -> f32 {
        libm::floorf(self)
    }

    #[inline]
    fn ceil(self) -> f32 {
        libm::ceilf(self)
    }
}

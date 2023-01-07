//! Embedded image handling library
#![no_std]
#![allow(unused_imports, dead_code)]
extern crate alloc;
use alloc::{vec, vec::Vec};
use core::slice::from_raw_parts;
use na::{Matrix3, Matrix3x1, Matrix4x1};

use nalgebra as na;

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

    /// linear light sRGB data
    sRGBLinear,

    /// No color-space / "display" / "as-is" / "web"
    AsIs,
}

#[derive(Debug)]
pub struct Image {
    data: Vec<WorkPixel>,
    res: ResXY,
}

impl Image {
    /// Read an Image from an array of pixel data of length `width * height * 4`
    ///
    /// Pixels are assumed to be in the order RGBA888
    ///
    /// Pixels will be cast as `f32` on load, but are otherwise as-is.
    ///
    /// # Panics
    ///
    /// - If `data` is not exactly `width * height * 4` in size
    pub fn from_bytes(data: &[u8], res: ResXY) -> Self {
        let (width, height) = res;
        let size = (width * height * 4) as usize;
        assert_eq!(data.len(), size);

        let data = unsafe {
            let len = (width * height) as usize;
            let data = data.as_ptr() as *const RawPixel;

            from_raw_parts(data, len)
                .iter()
                // .map(|f| f.map(|f| f as f32 / 255.))
                .map(|f| f.map(|f| f as f32))
                .collect()
        };
        Self { data, res }
    }

    fn width(&self) -> u32 {
        self.res.0
    }

    fn height(&self) -> u32 {
        self.res.1
    }

    fn pixels(&self) -> &[WorkPixel] {
        &self.data
    }
}

impl Image {
    #[cfg(no)]
    fn scale(&mut self, new: XY) {
        let width = self.width();
        let height = self.height();
        let (new_width, new_height) = (new.0, new.1);
        if (width, height) == (new_width, new_height) {
            return;
        }
        let x_scale = new_width as f32 / width as f32;
        let y_scale = new_height as f32 / height as f32;

        let pixels = self.pixels();
        let mut out: Vec<[f32; 4]> = vec![[0.; 4]; (new_height * new_width) as usize];

        for y in 0..new_height {
            for x in 0..new_width {
                let res = bilinear((x, y), (x_scale, y_scale), (width, height), pixels);

                let index = ((y * new_width) + x) as usize;
                out[index] = res;
            }
        }
        self.data = out;
        self.res = (new_width, new_height);
    }

    /// Convert rgb image to bgr
    fn swap_endian(&mut self) {
        for p in &mut self.data {
            *p = [p[0], p[1], p[2], p[3]];
        }
    }

    fn swap_rgb(&mut self) {
        for p in &mut self.data {
            *p = [p[2], p[1], p[0], p[3]]
            // *p = [p[2], p[1], p[0], 1.]
        }
    }

    fn srgb_to_rgb(&mut self) {
        const CIE: [f32; 9] = [
            0.4124, 0.3576, 0.1805, // R
            0.2126, 0.7152, 0.0722, // G
            0.0193, 0.1192, 0.9505, // B
        ];

        let cie = Matrix3::from_row_slice(&CIE);

        for p in &mut self.data {
            let q = Matrix3x1::from_column_slice(p);
            // info!("R");
            // panic!();
            let n = q.map(srgb_to_rgb);
            let n = cie * n;
            *p = [n[0], n[1], n[2], p[3]];
        }
    }

    fn rgb_to_srgb(&mut self) {
        const CIE: [f32; 9] = [
            // 3.2406255, -1.5372208, -0.4986286, // R
            // -0.9689307, 1.8757561, 0.0415175, // G
            // 0.0557101, -0.2040211, 1.0569959, // B
            3.2406, -1.5372, -0.4986, // R
            -0.9689, 1.8758, 0.0415, // G
            0.0557, -0.2040, 1.0570, // B
        ];

        let cie = Matrix3::from_row_slice(&CIE);

        for p in &mut self.data {
            let q = Matrix3x1::from_column_slice(p);
            let q = cie * q;
            let n = q.map(rgb_to_srgb);
            // FIXME: Probably isnt right?
            // Column vs row order??
            *p = [n[0], n[1], n[2], p[3]];
        }
    }

    // #[cfg(no)]
    fn gamma_to_rgb(&mut self) {
        for p in &mut self.data {
            *p = [
                p[0], //.powf(2.2),
                p[1], //.powf(2.2),
                p[2], //.powf(2.2),
                p[3], // .powf(2.2),
            ]
        }
    }

    // #[cfg(no)]
    fn rgb_to_gamma(&mut self) {
        for p in &mut self.data {
            *p = [
                p[0], //.powf(1.0 / 2.2),
                p[1], //.powf(1.0 / 2.2),
                p[2], //.powf(1.0 / 2.2),
                p[3], // .powf(1.0 / 2.2),
            ]
        }
    }

    /// Modify the image according to filter
    ///
    /// Filter receives (x, y) pixel data and returns
    #[cfg(no)]
    fn filter<F: FnMut(XY)>(&mut self, f: F) {
        let x = 0;
        let y = 0;
        let index = ((y * self.width) + x) as usize;

        self.data[index] = RawPixel::new(0, 0, 0);
        #[cfg(no)]
        {
            let x_scale = res.0 as f32 / hdr.width as f32;
            let y_scale = res.1 as f32 / hdr.height as f32;
            // Nearest Neighbor
            let x_near = (x as f32 / x_scale).floor() as u32;
            let y_near = (y as f32 / y_scale).floor() as u32;

            let index = ((y_near * hdr.width) + x_near) as usize * pixel_size;

            let pixel = &data[index..][..pixel_size];

            let index = ((y * res.0) + x) as usize * pixel_size;

            let red = pixel[0];
            let green = pixel[1];
            let blue = pixel[2];
            let alpha = pixel[3];
            let pixel = &if keep {
                [red, green, blue, 0]
            } else {
                [blue, green, red, 0]
            };

            out[index..][..pixel_size].copy_from_slice(&pixel[..pixel_size]);
        }
    }
}

#[cfg(no)]
#[allow(non_snake_case)]
fn bilinear(xy: XY, scale: FloatXY, src: ResXY, pixels: &[[f32; 4]]) -> [f32; 4] {
    let (x, y) = xy;
    let (x_scale, y_scale) = scale;
    let (width, height) = src;
    let x_ = x as f32 / x_scale as f32;
    let y_ = y as f32 / y_scale as f32;
    let alpha = pixels[((y * width) + x) as usize][3];
    // let alpha = 0.;

    let x1 = 0.; //(x_.floor() as u32).min(width - 1) as f32;
    let y1 = 0.; //(y_.floor() as u32).min(height - 1) as f32;
    let x2 = 0.; //(x_.ceil() as u32).min(width - 1) as f32;
    let y2 = 0.; //(y_.ceil() as u32).min(height - 1) as f32;

    let Q11 = ((y1 as u32 * width) + x1 as u32) as usize;
    let Q12 = ((y1 as u32 * width) + x2 as u32) as usize;
    let Q21 = ((y2 as u32 * width) + x1 as u32) as usize;
    let Q22 = ((y2 as u32 * width) + x2 as u32) as usize;

    let Q11 = Matrix4x1::from_column_slice(&pixels[Q11]);
    let Q12 = Matrix4x1::from_column_slice(&pixels[Q12]);
    let Q21 = Matrix4x1::from_column_slice(&pixels[Q21]);
    let Q22 = Matrix4x1::from_column_slice(&pixels[Q22]);

    let mut P1 = ((x2 - x_) * Q11) + ((x_ - x1) * Q12);
    let mut P2 = ((x2 - x_) * Q12) + ((x_ - x1) * Q22);

    if x1 == x2 {
        P1 = Q11;
        P2 = Q22;
    }

    let P = (((y2 - y_) * P1) + ((y_ - y1) * P2)); //.map(|f| f.round());
    (*P.as_slice()).try_into().unwrap()
    // [P[0], P[1], P[2], alpha]
}

fn srgb_to_rgb(f: f32) -> f32 {
    const A: f32 = 0.055;
    const GAMMA: f32 = 2.4;
    const PHI: f32 = 12.92;
    const C: f32 = 0.04045;
    if f <= C {
        f / PHI
    } else {
        ((f + A) / (1. + A)) //.powf(GAMMA)
    }
}

fn rgb_to_srgb(f: f32) -> f32 {
    const A: f32 = 0.055;
    const GAMMA: f32 = 2.4;
    const PHI: f32 = 12.92;
    const C: f32 = 0.0031308;
    if f <= C {
        PHI * f
    } else {
        (1. + A) * f //.powf(1. / GAMMA) - A
    }
}

/// Res: (width, height)
///
/// Returned image bytes will be in BGR, unless `keep` is true
#[allow(non_snake_case)]
#[cfg(no)]
fn fix_image(res: ResXY, size: ResXY, data: &[u8], keep: bool) -> (ResXY, Vec<u8>) {
    let mut out = vec![0; res.0 as usize * res.1 as usize * 4];

    let x_scale = res.0 as f32 / size.0 as f32;
    let y_scale = res.1 as f32 / size.1 as f32;
    // let downscale = (x_scale < 1.) || (y_scale < 1.);
    let downscale = false;
    let pixel_size = 4; //size.color_type.sample_multiplier();

    for y in 0..res.1 {
        for x in 0..res.0 {
            if downscale {
            } else {
                // Nearest Neighbor
                let x_near = (x as f32 / x_scale).floor() as u32;
                let y_near = (y as f32 / y_scale).floor() as u32;

                let index = ((y_near * size.width) + x_near) as usize * pixel_size;

                let pixel = &data[index..][..pixel_size];

                let index = ((y * res.0) + x) as usize * pixel_size;

                let red = pixel[0];
                let green = pixel[1];
                let blue = pixel[2];
                let alpha = pixel[3];
                let pixel = &if keep {
                    [red, green, blue, 0]
                } else {
                    [blue, green, red, 0]
                };

                out[index..][..pixel_size].copy_from_slice(&pixel[..pixel_size]);
            }
        }
    }
    let mut hdr = size.clone();
    hdr.width = res.0;
    hdr.height = res.1;
    (hdr, out)
}

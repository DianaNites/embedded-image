//! Image transformations
use crate::F32;

/// Transform sRGB into linear RGB
pub fn srgb_to_rgb(c: f32) -> f32 {
    const A: f32 = 0.055;
    const GAMMA: f32 = 2.4;
    const PHI: f32 = 12.92;
    const C: f32 = 0.04045;
    if c <= C {
        c / PHI
    } else {
        ((c + A) / (1. + A)).powf(GAMMA)
    }
}

/// Transform linear RGB into sRGB
pub fn rgb_to_srgb(c: f32) -> f32 {
    const A: f32 = 0.055;
    const GAMMA: f32 = 2.4;
    const PHI: f32 = 12.92;
    const C: f32 = 0.0031308;
    if c <= C {
        PHI * c
    } else {
        ((1. + A) * c.powf(1. / GAMMA)) - A
    }
}

/// Gamma / "simple" RGB to linear RGB
pub fn gamma_to_rgb(c: f32) -> f32 {
    c.powf(2.2)
}

/// Gamma / "simple" RGB from linear RGB
pub fn rgb_to_gamma(c: f32) -> f32 {
    c.powf(1.0 / 2.2)
}

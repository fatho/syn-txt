// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! This is the namespace for all parts dealing with data in sampled waves.

use std::ops;

/// A buffer holding floating point audio data.
pub struct AudioBuffer {
    samples: Vec<Stereo<f64>>,
}

#[allow(clippy::len_without_is_empty)]
impl AudioBuffer {
    pub fn new(sample_count: usize) -> Self {
        Self {
            samples: vec![
                Stereo {
                    left: 0.0,
                    right: 0.0
                };
                sample_count
            ],
        }
    }

    /// Set all samples to zero.
    pub fn fill_zero(&mut self) {
        self.samples
            .iter_mut()
            .for_each(|s| *s = Stereo::new(0.0, 0.0));
    }

    /// Size of the buffer in samples.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Size of the buffer in bytes.
    pub fn byte_len(&self) -> usize {
        self.len() * 2 * std::mem::size_of::<f64>()
    }

    pub fn samples(&self) -> &[Stereo<f64>] {
        &self.samples
    }

    pub fn samples_mut(&mut self) -> &mut [Stereo<f64>] {
        &mut self.samples
    }

    pub fn iter(&self) -> impl Iterator<Item = &Stereo<f64>> {
        self.samples.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Stereo<f64>> {
        self.samples.iter_mut()
    }

    /// Copy the stereo `f64` samples to bytes, interleaving the left and right samples.
    ///
    /// Could probably be implemented with some sort of unsafe transmute,
    /// but copying is safe and likely not the bottleneck.
    ///
    /// Returns the number of samples that were actually copied.
    /// Might be less than the number of input samples if the output buffer was not large enough.
    pub fn copy_bytes_to(&self, bytes: &mut [u8]) -> usize {
        let mut processed = 0;
        for (sample, target) in self.samples.iter().zip(bytes.chunks_exact_mut(16)) {
            target[0..8].copy_from_slice(&sample.left.to_le_bytes());
            target[8..16].copy_from_slice(&sample.right.to_le_bytes());
            processed += 1;
        }
        processed
    }
}

/// Convenience type for making things stereo, e.g. individual samples or whole buffers.
///
/// ```
/// use syn_txt::wave::*;
///
/// let stereo = Stereo::new(0.25, 0.5);
/// let stereo2 = stereo + Stereo::new(0.5, -0.25);
/// assert_eq!(stereo2 * 2.0, Stereo::new(1.5, 0.5));
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Stereo<T> {
    pub left: T,
    pub right: T,
}

impl<T> Stereo<T> {
    pub fn new(left: T, right: T) -> Self {
        Self { left, right }
    }

    pub fn mono(mono: T) -> Self
    where
        T: Copy,
    {
        Self::new(mono, mono)
    }
}

impl Stereo<f64> {
    /// Linearly spread a mono signal onto stereo channels, by keeping
    /// one channel at 100% while linearly attenuating the other.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syn_txt::wave::*;
    ///
    /// assert_eq!(Stereo::panned_mono(1.0, 0.0), Stereo::new(1.0, 1.0));
    /// assert_eq!(Stereo::panned_mono(1.0, -1.0), Stereo::new(1.0, 0.0));
    /// assert_eq!(Stereo::panned_mono(1.0, 1.0), Stereo::new(0.0, 1.0));
    /// ```
    pub fn panned_mono(mono: f64, pan: f64) -> Self {
        let left = mono * 1.0f64.min(1.0 - pan);
        let right = mono * 1.0f64.min(1.0 + pan);
        Stereo::new(left, right)
    }
}

impl std::iter::Sum for Stereo<f64> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut out = Stereo::new(0.0, 0.0);
        for x in iter {
            out += x;
        }
        out
    }
}

impl<T: ops::Add> ops::Add for Stereo<T> {
    type Output = Stereo<T::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Stereo {
            left: self.left + rhs.left,
            right: self.right + rhs.right,
        }
    }
}

impl<T: ops::AddAssign> ops::AddAssign for Stereo<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.left += rhs.left;
        self.right += rhs.right;
    }
}

impl<T: ops::Sub> ops::Sub for Stereo<T> {
    type Output = Stereo<T::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Stereo {
            left: self.left - rhs.left,
            right: self.right - rhs.right,
        }
    }
}

impl<T: ops::SubAssign> ops::SubAssign for Stereo<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.left -= rhs.left;
        self.right -= rhs.right;
    }
}

impl<T: ops::Mul + Copy> ops::Mul<T> for Stereo<T> {
    type Output = Stereo<T::Output>;

    fn mul(self, rhs: T) -> Self::Output {
        Stereo {
            left: self.left * rhs,
            right: self.right * rhs,
        }
    }
}

macro_rules! impl_left_Mul {
    ($t:ty) => {
        impl ops::Mul<Stereo<$t>> for $t {
            type Output = Stereo<$t>;

            fn mul(self, rhs: Stereo<$t>) -> Self::Output {
                Stereo {
                    left: self * rhs.left,
                    right: self * rhs.right,
                }
            }
        }
    };
}

impl_left_Mul!(i64);
impl_left_Mul!(i32);
impl_left_Mul!(i16);
impl_left_Mul!(i8);
impl_left_Mul!(u64);
impl_left_Mul!(u32);
impl_left_Mul!(u16);
impl_left_Mul!(u8);
impl_left_Mul!(f32);
impl_left_Mul!(f64);

impl<T: ops::MulAssign + Copy> ops::MulAssign<T> for Stereo<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.left *= rhs;
        self.right *= rhs;
    }
}

impl<T: ops::Div + Copy> ops::Div<T> for Stereo<T> {
    type Output = Stereo<T::Output>;

    fn div(self, rhs: T) -> Self::Output {
        Stereo {
            left: self.left / rhs,
            right: self.right / rhs,
        }
    }
}

impl<T: ops::DivAssign + Copy> ops::DivAssign<T> for Stereo<T> {
    fn div_assign(&mut self, rhs: T) {
        self.left /= rhs;
        self.right /= rhs;
    }
}

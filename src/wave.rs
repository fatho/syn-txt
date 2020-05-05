//! This is the namespace for all parts dealing with data in sampled waves.

use std::ops;

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

    pub fn mono(mono: T) -> Self where T: Copy {
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

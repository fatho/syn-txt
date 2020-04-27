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
        Self {
            left, right,
        }
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

impl<T: ops::Sub> ops::Sub for Stereo<T> {
    type Output = Stereo<T::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Stereo {
            left: self.left - rhs.left,
            right: self.right - rhs.right,
        }
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

impl<T: ops::Div + Copy> ops::Div<T> for Stereo<T> {
    type Output = Stereo<T::Output>;

    fn div(self, rhs: T) -> Self::Output {
        Stereo {
            left: self.left / rhs,
            right: self.right / rhs,
        }
    }
}

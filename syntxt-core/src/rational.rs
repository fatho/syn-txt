// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Rational numbers are used for designating times on the song level, e.g. note lenghts.

use std::error::Error;
use std::fmt;
use std::{cmp::Ordering, ops};

/// Underlying integral type for the rational numbers.
type Int = i64;

/// A rational number, always fully normalized.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rational {
    /// The numerator of the fraction.
    /// If the fraction is negative, the numerator will be made negative.
    num: Int,
    /// The denominator of the fraction.
    /// If the fraction is negative, the denominator will stay positive.
    denom: Int,
}

impl Rational {
    pub const ZERO: Rational = Rational { num: 0, denom: 1 };

    // ==================== Constructors ====================

    /// Create a new rational from a potentially unnormalized fraction.
    ///
    /// # Panic
    ///
    /// Panics if the denominator is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syntxt_core::rational::*;
    ///
    /// assert_eq!(Rational::new(10, 5), Rational::new(2, 1));
    /// assert_eq!(Rational::new(-10, -5), Rational::new(6, 3));
    /// assert_eq!(Rational::new(-6, 8), Rational::new(3, -4));
    /// ```
    pub fn new(num: Int, denom: Int) -> Rational {
        assert_ne!(denom, 0, "Denominator must not be zero");

        let sign = num.signum() * denom.signum();
        let div = gcd(num, denom);
        Rational {
            num: sign * num.abs() / div,
            denom: denom.abs() / div,
        }
    }

    pub const fn int(int: Int) -> Rational {
        Rational { num: int, denom: 1 }
    }

    pub const fn zero() -> Rational {
        Rational::int(0)
    }

    pub const fn one() -> Rational {
        Rational::int(1)
    }

    pub fn nth(n: Int) -> Self {
        Rational::new(1, n)
    }

    // ==================== Transformations ====================

    pub const fn recip(self) -> Rational {
        Rational {
            num: self.denom,
            denom: self.num,
        }
    }

    /// Compute an integer power of the rational.
    ///
    /// ```
    /// # use syntxt_core::rational::*;
    /// assert_eq!(Rational::int(2).powi(0), Rational::int(1));
    /// assert_eq!(Rational::int(2).powi(1), Rational::int(2));
    /// assert_eq!(Rational::int(2).powi(2), Rational::int(4));
    /// assert_eq!(Rational::int(2).powi(3), Rational::int(8));
    /// assert_eq!(Rational::int(2).powi(5), Rational::int(32));
    /// assert_eq!(Rational::int(2).powi(10), Rational::int(1024));
    /// assert_eq!(Rational::int(2).powi(11), Rational::int(2048));
    /// assert_eq!(Rational::int(2).powi(-1), Rational::new(1, 2));
    /// assert_eq!(Rational::int(2).powi(-2), Rational::new(1, 4));
    /// assert_eq!(Rational::int(2).powi(-3), Rational::new(1, 8));
    /// assert_eq!(Rational::int(2).powi(-9), Rational::new(1, 512));
    /// ```
    pub fn powi(self, power: Int) -> Rational {
        if power == 0 {
            return Self::one();
        }
        let mut accum = if power > 0 { self } else { self.recip() };
        let mut correction = Rational::one();
        let mut remaining_power = power.abs();

        while remaining_power > 1 {
            if remaining_power % 2 == 1 {
                correction *= accum;
                remaining_power -= 1;
            }
            accum = accum * accum;
            remaining_power /= 2;
        }

        accum * correction
    }

    /// Round towards zero.
    ///
    /// ```
    /// # use syntxt_core::rational::*;
    ///
    /// assert_eq!(Rational::new(10, 5).truncate(), 2);
    /// assert_eq!(Rational::new(-10, 6).truncate(), -1);
    /// assert_eq!(Rational::new(13, 7).truncate(), 1);
    /// ```
    pub const fn truncate(self) -> i64 {
        self.num / self.denom
    }

    /// Round to closed integer, half up.
    ///
    /// ```
    /// # use syntxt_core::rational::*;
    ///
    /// assert_eq!(Rational::new(10, 5).round(), 2);
    /// assert_eq!(Rational::new(-10, 5).round(), -2);
    ///
    /// assert_eq!(Rational::new(10, 4).round(), 3);
    /// assert_eq!(Rational::new(-10, 4).round(), -3);
    ///
    /// assert_eq!(Rational::new(3, 7).round(), 0);
    /// assert_eq!(Rational::new(4, 7).round(), 1);
    /// assert_eq!(Rational::new(-3, 7).round(), 0);
    /// assert_eq!(Rational::new(-4, 7).round(), -1);
    /// ```
    pub fn round(self) -> i64 {
        (self.num + self.num.signum() * self.denom / 2) / self.denom
    }

    // ==================== Predicates ====================

    pub const fn is_zero(self) -> bool {
        self.num == 0
    }

    // ==================== Destructors ====================

    pub const fn numerator(self) -> Int {
        self.num
    }

    pub const fn denominator(self) -> Int {
        self.denom
    }
}

/// # Examples
///
/// ```
/// use syntxt_core::rational::*;
///
/// assert_eq!(Rational::new(1, 2) + Rational::new(3, 4), Rational::new(5, 4));
/// assert_eq!(Rational::new(3, 4) + Rational::new(3, 4), Rational::new(3, 2));
/// assert_eq!(Rational::new(3, 4) + Rational::new(-5, 8), Rational::new(1, 8));
/// ```
impl ops::Add for Rational {
    type Output = Rational;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Rational) -> Self::Output {
        Rational::new(
            self.num * rhs.denom + self.denom * rhs.num,
            self.denom * rhs.denom,
        )
    }
}

impl ops::Sub for Rational {
    type Output = Rational;

    fn sub(self, rhs: Rational) -> Self::Output {
        self + (-rhs)
    }
}

impl ops::Mul for Rational {
    type Output = Rational;

    fn mul(self, rhs: Rational) -> Self::Output {
        Rational::new(self.num * rhs.num, self.denom * rhs.denom)
    }
}

impl ops::Div for Rational {
    type Output = Rational;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Rational) -> Self::Output {
        self * rhs.recip()
    }
}

/// # Examples
///
/// ```
/// # use syntxt_core::rational::*;
///
/// assert_eq!(Rational::new(5, 1) % Rational::new(3, 1), Rational::new(2, 1));
/// assert_eq!(Rational::new(-5, 1) % Rational::new(3, 1), Rational::new(-2, 1));
/// assert_eq!(Rational::new(7, 3) % Rational::new(1, 4), Rational::new(1, 12));
/// assert_eq!(Rational::new(-7, 3) % Rational::new(1, 4), Rational::new(-1, 12));
/// ```
impl ops::Rem for Rational {
    type Output = Rational;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn rem(self, rhs: Rational) -> Self::Output {
        let common_denom = self.denom * rhs.denom;
        let self_num = self.num * rhs.denom;
        let rhs_num = rhs.num * self.denom;
        Rational::new(self_num % rhs_num, common_denom)
    }
}

impl ops::Mul<Int> for Rational {
    type Output = Rational;

    fn mul(self, rhs: Int) -> Self::Output {
        Rational::new(self.num * rhs, self.denom)
    }
}

impl ops::Mul<Rational> for Int {
    type Output = Rational;

    fn mul(self, rhs: Rational) -> Self::Output {
        Rational::new(self * rhs.num, rhs.denom)
    }
}

/// ```
/// # use syntxt_core::rational::*;
/// assert_eq!(Rational::new(1, 4) / 2, Rational::new(1, 8));
/// assert_eq!(Rational::new(9, 13) / 3, Rational::new(3, 13));
/// ```
impl ops::Div<Int> for Rational {
    type Output = Rational;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Int) -> Self::Output {
        Rational::new(self.num, self.denom * rhs)
    }
}

impl ops::Neg for Rational {
    type Output = Rational;

    fn neg(self) -> Self::Output {
        Rational {
            num: -self.num,
            denom: self.denom,
        }
    }
}

impl ops::AddAssign for Rational {
    fn add_assign(&mut self, rhs: Rational) {
        *self = *self + rhs;
    }
}

impl ops::SubAssign for Rational {
    fn sub_assign(&mut self, rhs: Rational) {
        *self = *self - rhs;
    }
}

impl ops::MulAssign for Rational {
    fn mul_assign(&mut self, rhs: Rational) {
        *self = *self * rhs;
    }
}

impl ops::DivAssign for Rational {
    fn div_assign(&mut self, rhs: Rational) {
        *self = *self / rhs;
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Rational) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// ```
/// use syntxt_core::rational::*;
///
/// assert!(Rational::new(3,4) < Rational::new(3,2));
/// ```
impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        // a / b < c / d
        // <=>
        // a < c * b / d
        // <=>
        // a * d < c * b
        let l = self.num * other.denom;
        let r = other.num * self.denom;
        l.cmp(&r)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.num)?;
        if self.denom != 1 {
            write!(f, "/{}", self.denom)?;
        }
        Ok(())
    }
}

/// An error which can be returned when parsing a rational.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRationalError(RationalErrorKind);

impl ParseRationalError {
    pub fn kind(&self) -> RationalErrorKind {
        self.0
    }
}

impl Error for ParseRationalError {}

impl fmt::Display for ParseRationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            RationalErrorKind::InvalidInt => write!(f, "invalid integer literal"),
            RationalErrorKind::Zero => write!(f, "denominator is zero"),
            RationalErrorKind::Malformed => write!(f, "malformed fraction"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RationalErrorKind {
    /// The numerator or denominator could not be parsed as integer.
    InvalidInt,
    /// The denominator was zero
    Zero,
    /// The rational was not of the form `<int>` or `<int>/<int>
    Malformed,
}

impl std::str::FromStr for Rational {
    type Err = ParseRationalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('/');
        let numerator_str = parts.next().unwrap();
        let numerator = numerator_str
            .parse()
            .map_err(|_| ParseRationalError(RationalErrorKind::InvalidInt))?;

        if let Some(denominator_str) = parts.next() {
            let denominator = denominator_str
                .parse()
                .map_err(|_| ParseRationalError(RationalErrorKind::InvalidInt))?;
            if denominator == 0 {
                Err(ParseRationalError(RationalErrorKind::Zero))
            } else if parts.next().is_some() {
                Err(ParseRationalError(RationalErrorKind::Malformed))
            } else {
                Ok(Rational::new(numerator, denominator))
            }
        } else {
            Ok(Rational::new(numerator, 1))
        }
    }
}

/// Computes the greates common divisor of two numbers using euclids algorithm.
///
/// # Example
///
/// ```
/// use syntxt_core::rational::*;
///
/// assert_eq!(gcd(20, 15), 5);
/// assert_eq!(gcd(20, 19), 1);
/// assert_eq!(gcd(10, 0), 10);
/// assert_eq!(gcd(0, 10), 10);
/// assert_eq!(gcd(0, 0), 0);
/// assert_eq!(gcd(10, -10), 10);
/// ```
pub fn gcd(mut a: Int, mut b: Int) -> Int {
    // normalized inputs to be positive to guarantee that it terminates
    if a < 0 {
        a = -a
    }
    if b < 0 {
        b = -b
    }

    // Invariant: a >= b
    if a < b {
        std::mem::swap(&mut a, &mut b)
    }

    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

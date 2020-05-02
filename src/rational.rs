//! Rational numbers are used for designating times on the song level, e.g. note lenghts.

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
    /// use syn_txt::rational::*;
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

    pub fn from_int(int: Int) -> Rational {
        Rational { num: int, denom: 1 }
    }

    pub fn zero() -> Rational {
        Rational::new(0, 1)
    }

    pub fn one() -> Rational {
        Rational::new(1, 1)
    }

    pub fn nth(n: Int) -> Self {
        Rational::new(1, n)
    }

    // ==================== Transformations ====================

    pub fn recip(self) -> Rational {
        Rational {
            num: self.denom,
            denom: self.num,
        }
    }

    // ==================== Destructors ====================

    pub fn numerator(self) -> Int {
        self.num
    }

    pub fn denominator(self) -> Int {
        self.denom
    }
}

/// # Examples
///
/// ```
/// use syn_txt::rational::*;
///
/// assert_eq!(Rational::new(1, 2) + Rational::new(3, 4), Rational::new(5, 4));
/// assert_eq!(Rational::new(3, 4) + Rational::new(3, 4), Rational::new(3, 2));
/// assert_eq!(Rational::new(3, 4) + Rational::new(-5, 8), Rational::new(1, 8));
/// ```
impl ops::Add for Rational {
    type Output = Rational;

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

    fn div(self, rhs: Rational) -> Self::Output {
        self * rhs.recip()
    }
}

impl ops::Mul<Int> for Rational {
    type Output = Rational;

    fn mul(self, rhs: Int) -> Self::Output {
        Rational::new(self.num * rhs, self.denom)
    }
}

impl ops::Div<Int> for Rational {
    type Output = Rational;

    fn div(self, rhs: Int) -> Self::Output {
        Rational::new(self.num, self.denom / rhs)
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
/// use syn_txt::rational::*;
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

/// An error which can be returned when parsing a rational.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseRationalError(RationalErrorKind);

impl ParseRationalError {
    pub fn kind(&self) -> RationalErrorKind {
        self.0
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
                return Err(ParseRationalError(RationalErrorKind::Zero));
            } else if let Some(_) = parts.next() {
                return Err(ParseRationalError(RationalErrorKind::Malformed));
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
/// use syn_txt::rational::*;
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

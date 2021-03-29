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

//! Floats that cannot be NaN by construction, and hence are `Ord` and `Eq`.

use std::{fmt::Display, num::ParseFloatError, str::FromStr};

/// A non-nan f64.
#[derive(Debug, Clone, Copy)]
pub struct F64N(f64);

impl F64N {
    pub fn new(value: f64) -> Option<F64N> {
        if value.is_nan() {
            None
        } else {
            Some(Self(value))
        }
    }

    pub unsafe fn new_unchecked(value: f64) -> F64N {
        F64N(value)
    }

    pub fn into_inner(self) -> f64 {
        self.0
    }
}

impl PartialEq for F64N {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for F64N {}

impl PartialOrd for F64N {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for F64N {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("bug: non-nan is nan")
    }
}

impl FromStr for F64N {
    type Err = ParseNonNanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.parse::<f64>()?).ok_or(ParseNonNanError::Nan)
    }
}

/// A non-nan f32.
#[derive(Debug, Clone, Copy)]
pub struct F32N(f32);

impl F32N {
    pub fn new(value: f32) -> Option<F32N> {
        if value.is_nan() {
            None
        } else {
            Some(Self(value))
        }
    }

    pub unsafe fn new_unchecked(value: f32) -> F32N {
        F32N(value)
    }

    pub fn into_inner(self) -> f32 {
        self.0
    }
}


impl PartialEq for F32N {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for F32N {}

impl PartialOrd for F32N {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for F32N {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("bug: non-nan is nan")
    }
}

impl FromStr for F32N {
    type Err = ParseNonNanError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.parse::<f32>()?).ok_or(ParseNonNanError::Nan)
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum ParseNonNanError {
    Nan,
    Other(std::num::ParseFloatError),
}

impl Display for ParseNonNanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseNonNanError::Nan => write!(f, "float was nan"),
            ParseNonNanError::Other(error) => error.fmt(f),
        }
    }
}

impl From<ParseFloatError> for ParseNonNanError {
    fn from(err: ParseFloatError) -> Self {
        ParseNonNanError::Other(err)
    }
}

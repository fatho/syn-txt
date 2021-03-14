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

use snafu::Snafu;

/// Opaque variable datatype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Var(usize);

/// Simple expressions for numeric parameters
#[derive(Debug, Clone)]
pub enum Expr {
    Const(f64),
    Var(Var),
    BuiltInVar(BuiltInVar),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltInVar {
    GlobalTimeSeconds,
    NoteTimeSeconds
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Sin,
    Cos,
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum EvalError {
    #[snafu(display("Referenced variable {:?} does not exist", var))]
    UnknownVariable { var: Var },
}

pub struct BuiltInValues {
    pub global_time_seconds: f64,
    pub note_time_seconds: f64,
}

impl Default for BuiltInValues {
    fn default() -> Self {
        Self {
            global_time_seconds: 0.0,
            note_time_seconds: 0.0,
        }
    }
}

impl BuiltInValues {
    pub fn get(&self, var: BuiltInVar) -> f64 {
        match var {
            BuiltInVar::GlobalTimeSeconds => self.global_time_seconds,
            BuiltInVar::NoteTimeSeconds => self.note_time_seconds,
        }
    }
}

impl Expr {
    pub fn eval(&self, builtins: &BuiltInValues, env: &[f64]) -> Result<f64, EvalError> {
        match self {
            Expr::Const(x) => Ok(*x),
            Expr::Var(v) => {
                if let Some(val) = env.get(v.0) {
                    Ok(*val)
                } else {
                    Err(EvalError::UnknownVariable { var: *v })
                }
            }
            Expr::BuiltInVar(v) => Ok(builtins.get(*v)),
            Expr::BinOp(op, x, y) => {
                let x = x.eval(builtins, env)?;
                let y = y.eval(builtins, env)?;
                Ok(match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => x / y,
                    BinOp::Rem => x % y,
                    BinOp::Pow => x.powf(y),
                })
            }
            Expr::UnOp(op, x) => {
                let x = x.eval(builtins, env)?;
                Ok(match op {
                    UnOp::Sin => x.sin(),
                    UnOp::Cos => x.cos(),
                })
            }
        }
    }

    /// Parse expressions from a prefix-notation string.
    pub fn parse(input: &str) -> Option<Expr> {
        let mut splitter = input.split_ascii_whitespace();
        Self::parse_any(&mut splitter)
    }

    fn parse_any(input: &mut dyn Iterator<Item = &str>) -> Option<Expr> {
        match input.next() {
            Some("+") => Self::parse_binop(BinOp::Add, input),
            Some("-") => Self::parse_binop(BinOp::Sub, input),
            Some("*") => Self::parse_binop(BinOp::Mul, input),
            Some("/") => Self::parse_binop(BinOp::Div, input),
            Some("%") => Self::parse_binop(BinOp::Rem, input),
            Some("^") => Self::parse_binop(BinOp::Pow, input),
            Some("sin") => Self::parse_unop(UnOp::Sin, input),
            Some("cos") => Self::parse_unop(UnOp::Cos, input),
            // Global constants
            Some("time") => Some(Expr::BuiltInVar(BuiltInVar::GlobalTimeSeconds)),
            Some("note_time") => Some(Expr::BuiltInVar(BuiltInVar::NoteTimeSeconds)),
            Some(other) => {
                if other.starts_with('$') {
                    // $-variables refer to node inputs (not yet implemented in the graph)
                    Some(Expr::Var(Var(other[1..].parse().ok()?)))
                } else {
                    // everything else must be a number
                    Some(Expr::Const(other.parse().ok()?))
                }
            }
            _ => None,
        }
    }

    fn parse_binop(op: BinOp, input: &mut dyn Iterator<Item = &str>) -> Option<Expr> {
        let l = Self::parse_any(input)?;
        let r = Self::parse_any(input)?;
        Some(Expr::BinOp(op, Box::new(l), Box::new(r)))
    }

    fn parse_unop(op: UnOp, input: &mut dyn Iterator<Item = &str>) -> Option<Expr> {
        let l = Self::parse_any(input)?;
        Some(Expr::UnOp(op, Box::new(l)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(Expr::parse("+ 1 2").map(|x| x.eval(&BuiltInValues::default(), &[])), Some(Ok(3.0)));
        assert_eq!(
            Expr::parse("+ 2 * 3 4").map(|x| x.eval(&BuiltInValues::default(), &[])),
            Some(Ok(14.0))
        );
        assert_eq!(
            Expr::parse("/ + 2 * 3 4 5").map(|x| x.eval(&BuiltInValues::default(), &[])),
            Some(Ok(14.0 / 5.0))
        );
        assert_eq!(Expr::parse("% 9 4").map(|x| x.eval(&BuiltInValues::default(), &[])), Some(Ok(1.0)));
        assert_eq!(Expr::parse("^ 3 2").map(|x| x.eval(&BuiltInValues::default(), &[])), Some(Ok(9.0)));
    }

    #[test]
    fn variables() {
        let env = &[2.0, 5.0, 10.0];
        assert_eq!(
            Expr::parse("/ - $1 $2 $0").map(|x| x.eval(&BuiltInValues::default(), env)),
            Some(Ok(-2.5))
        );
    }

    #[test]
    fn undefined_variables() {
        let env = &[2.0];
        assert_eq!(
            Expr::parse("- $1 $0").map(|x| x.eval(&BuiltInValues::default(), env)),
            Some(Err(EvalError::UnknownVariable { var: Var(1) }))
        );
    }

    #[test]
    fn trigonometry() {
        assert_eq!(Expr::parse("sin 0").map(|x| x.eval(&BuiltInValues::default(), &[])), Some(Ok(0.0)));
        assert_eq!(Expr::parse("cos 0").map(|x| x.eval(&BuiltInValues::default(), &[])), Some(Ok(1.0)));
    }
}

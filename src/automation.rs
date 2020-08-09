use snafu::Snafu;

/// Opaque variable datatype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Var(usize);

/// Simple expressions for numeric parameters
#[derive(Debug, Clone)]
pub enum Expr {
    Const(f64),
    Var(Var),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum EvalError {
    #[snafu(display("Referenced variable {:?} does not exist", var))]
    UnknownVariable { var: Var },
}

impl Expr {
    pub fn eval(&self, env: &[f64]) -> Result<f64, EvalError> {
        match self {
            Expr::Const(x) => Ok(*x),
            Expr::Var(v) => {
                if let Some(val) = env.get(v.0) {
                    Ok(*val)
                } else {
                    Err(EvalError::UnknownVariable { var: *v })
                }
            }
            Expr::BinOp(op, x, y) => {
                let x = x.eval(env)?;
                let y = y.eval(env)?;
                Ok(match op {
                    BinOp::Add => x + y,
                    BinOp::Sub => x - y,
                    BinOp::Mul => x * y,
                    BinOp::Div => x / y,
                    BinOp::Rem => x % y,
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
            Some(other) => {
                if other.starts_with('$') {
                    Some(Expr::Var(Var(other[1..].parse().ok()?)))
                } else {
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(Expr::parse("+ 1 2").map(|x| x.eval(&[])), Some(Ok(3.0)));
        assert_eq!(
            Expr::parse("+ 2 * 3 4").map(|x| x.eval(&[])),
            Some(Ok(14.0))
        );
        assert_eq!(
            Expr::parse("/ + 2 * 3 4 5").map(|x| x.eval(&[])),
            Some(Ok(14.0 / 5.0))
        );
        assert_eq!(Expr::parse("% 9 4").map(|x| x.eval(&[])), Some(Ok(1.0)));
    }

    #[test]
    fn variables() {
        let env = &[2.0, 5.0, 10.0];
        assert_eq!(
            Expr::parse("/ - $1 $2 $0").map(|x| x.eval(env)),
            Some(Ok(-2.5))
        );
    }

    #[test]
    fn undefined_variables() {
        let env = &[2.0];
        assert_eq!(
            Expr::parse("- $1 $0").map(|x| x.eval(env)),
            Some(Err(EvalError::UnknownVariable { var: Var(1) }))
        );
    }
}

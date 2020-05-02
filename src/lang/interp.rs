use std::collections::HashMap;

use super::ast;

type InterpreterError = String;
type InterpreterResult<T> = Result<T, InterpreterError>;

pub struct Interpreter {
    scopes: Vec<Scope>,
    context: ContextKind,
}

macro_rules! change_context {
    ($interpreter:ident, $context:expr, $code:expr) => {
        {
            let old = $interpreter.context;
            $interpreter.context = $context;
            let result = $code;
            $interpreter.context = old;
            result
        }
    };
}

impl Interpreter {

    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            context: ContextKind::Statement,
        }
    }

    pub fn lookup_var(&self, var: &ast::Ident) -> Option<&ast::Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.lookup_var(var) {
                return Some(val)
            }
        }
        None
    }

    pub fn eval(&mut self, sym: &ast::SymExp) -> InterpreterResult<Option<ast::Value>> {
        match sym {
            ast::SymExp::Keyword(_) => Err("cannot evaluate keywords".to_owned()),
            ast::SymExp::List(list) => self.eval_list(&list),
            ast::SymExp::Literal(value) => Ok(Some(value.clone())),
            ast::SymExp::Variable(var) =>
                if let Some(value) = self.lookup_var(var) {
                    Ok(Some(value.clone()))
                } else {
                    Err(format!("no such variable {}", var.0))
                },
        }
    }

    fn eval_list(&mut self, list: &[ast::SymExp]) -> InterpreterResult<Option<ast::Value>> {
        let head_exp = list.first().ok_or("cannot evaluate empty list".to_owned())?;
        let args = &list[1..];

        // TODO: allow arbitrary expression as head and evaluate it
        // let head = change_context!(self, ContextKind::Expression, self.eval(head_exp))?.ok_or("expression did not evaluate to value")?;
        match head_exp {
            ast::SymExp::Variable(fun) => match fun.0.as_str() {
                "define" => {
                    self.check_in_statement_context("define")?;
                    match args {
                        [ast::SymExp::Variable(var), value_exp] => {
                            let value = change_context!(self, ContextKind::Expression, self.eval(value_exp))?.ok_or("expression did not evaluate to value")?;
                            self.scopes.last_mut().expect("must have at least one scope").set_var(var.clone(), value);
                            Ok(None)
                        }
                        _ => {
                            Err(format!("invalid arguments in `define`"))
                        }
                    }
                }
                unknown => Err(format!("unknown function {:?}", unknown))
            }
            _ => Err(format!("cannot call {:?}", head_exp)),
        }
    }

    fn check_in_statement_context(&self, what: &str) -> InterpreterResult<()> {
        if self.context == ContextKind::Statement {
            Ok(())
        } else {
            Err(format!("{:?} is only valid in a statement context", what))
        }
    }
}

struct Scope {
    bindings: HashMap<ast::Ident, ast::Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn set_var(&mut self, var: ast::Ident, value: ast::Value) {
        self.bindings.insert(var, value);
    }

    pub fn lookup_var(&self, var: &ast::Ident) -> Option<&ast::Value> {
        self.bindings.get(var)
    }
}

/// The context determines what kind of symbolic expressions are allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextKind {
    /// Only expressions are allowed, not statements.
    Expression,
    /// Both expressions and statements are allowed.
    Statement
}

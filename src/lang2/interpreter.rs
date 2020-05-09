use std::fmt;

use super::value::*;
use super::debug;
use super::primops;
use super::heap::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    location: Option<debug::SourceLocation>,
    info: EvalErrorKind,
}

impl EvalError {
    pub fn new(location: Option<debug::SourceLocation>, info: EvalErrorKind) -> Self {
        Self { location, info }
    }

    pub fn location(&self) -> Option<&debug::SourceLocation> {
        self.location.as_ref()
    }

    pub fn info(&self) -> &EvalErrorKind {
        &self.info
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalErrorKind {
    /// Some expressions, such as keywords or empty lists, cannot be evaluated
    Unevaluatable,
    /// Variable/function was not found
    NoSuchVariable(Symbol),
    /// Tried to call something that cannot be called, such as the int in `(1 a b)`.
    Uncallable,
    /// There was a problem with the arguments in a call
    IncompatibleArguments,
    NotEnoughArguments,
    TooManyArguments,
    /// Keyword was not understood by callee.
    UnknownKeyword(Symbol),
    DivisionByZero,
    /// Type error (e.g. trying to add two incompatible types).
    Type,
    /// Tried to redefine a variable in the scope it was originally defined.
    /// (Shadowing variables in a new scope is fine).
    Redefinition(Symbol),
    /// Miscellaneous errors that shouldn't happen, but might.
    Other(String),
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::Unevaluatable => write!(f, "unevaluatable"),
            EvalErrorKind::NoSuchVariable(var) => write!(f, "no such variable `{}`", var.as_str()),
            EvalErrorKind::Uncallable => write!(f, "uncallable"),
            EvalErrorKind::IncompatibleArguments => write!(f, "incompatible arguments"),
            EvalErrorKind::NotEnoughArguments => write!(f, "not enough arguments in function call"),
            EvalErrorKind::TooManyArguments => write!(f, "too many arguments in function call"),
            EvalErrorKind::UnknownKeyword(var) => write!(f, "unknown keyword `{}`", var.as_str()),
            EvalErrorKind::DivisionByZero => write!(f, "division by zero"),
            EvalErrorKind::Redefinition(var) => write!(f, "redefined variable `{}`", var.as_str()),
            EvalErrorKind::Type => write!(f, "type error"),
            EvalErrorKind::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// Evaluation-specific result type.
pub type Result<T> = std::result::Result<T, EvalError>;

pub struct Interpreter {
    /// Read-only scope (from the perspective of the language)
    /// providing all the built-in primops.
    builtins: Gc<Scope>,
    /// Points to the current innermost scope
    scope_stack: Gc<Scope>,
    /// The interpreter heap.
    heap: Heap,
    /// Debug information about values.
    debug_info: debug::DebugTable,
}

impl Interpreter {
    pub fn new(mut heap: Heap, debug_info: debug::DebugTable) -> Self {
        let builtin_scope = Scope::new();

        let prim = vec![
            // // syntax
            // ("begin", PrimOp(primops::begin)),
            // ("define", PrimOp(primops::define)),
            // ("lambda", PrimOp(primops::lambda)),
            // ("set!", PrimOp(primops::set)),
            // // arithmetic
            // ("+", PrimOp(primops::add)),
            // ("-", PrimOp(primops::sub)),
            // ("*", PrimOp(primops::mul)),
            // ("/", PrimOp(primops::div)),
            // // lists
            // ("list", PrimOp(primops::list)),
            // ("concat", PrimOp(primops::concat)),
            // ("reverse", PrimOp(primops::reverse)),
            // ("for-each", PrimOp(primops::for_each)),
            // ("map", PrimOp(primops::map)),
            // ("range", PrimOp(primops::range)),
            // // dicts
            // ("dict", PrimOp(primops::dict)),
            // ("update", PrimOp(primops::dict_update)),
            // ("get", PrimOp(primops::dict_get)),
            // util
            ("print", PrimOp(primops::print)),
        ];

        for (name, fun) in prim {
            builtin_scope.define(name.into(), heap.alloc(Value::PrimOp(fun)));
        }

        let builtins = heap.alloc(builtin_scope);
        let top_scope = heap.alloc(Scope::nest(builtins.clone()));

        Self {
            builtins,
            scope_stack: top_scope,
            heap,
            debug_info,
        }
    }

    pub fn register_primop(
        &mut self,
        name: &str,
        op: fn(&mut Interpreter, Gc<Value>) -> Result<Gc<Value>>,
    ) -> Result<()> {
        let var = name.into();
        let val = self.heap.alloc(Value::PrimOp(PrimOp(op)));
        if let Some((var, _val)) = self.builtins.pin().define(var, val) {
            // TODO: allow None as location
            Err(EvalError::new(
                None,
                EvalErrorKind::Redefinition(var),
            ))
        } else {
            Ok(())
        }
    }

    pub fn debug_info(&self) -> &debug::DebugTable {
        &self.debug_info
    }

    fn err_by(&self, cause: Id, kind: EvalErrorKind) -> EvalError {
        let location = self.debug_info.get_location(cause);
        EvalError::new(location.cloned(), kind)
    }

    pub fn heap_alloc(&mut self, value: Value) -> Gc<Value> {
        // TODO: share heap allocation for small values
        self.heap.alloc(value)
    }

    pub fn scope_stack(&mut self) -> &Gc<Scope> {
        &self.scope_stack
    }

    /// Create a new topmost scope for bindings.
    /// Any `define`s and `set!`s will target the top-most scope.
    pub fn push_scope(&mut self) {
        let new_scope = Scope::nest(self.scope_stack.clone());
        self.scope_stack = self.heap.alloc(new_scope);
    }

    /// Remove the topmost scope and all its bindings.
    pub fn pop_scope(&mut self) {
        let outer = self.scope_stack.pin().outer();
        if let Some(outer) = outer {
            self.scope_stack = outer;
        } else {
            log::warn!("trying to pop outermost scope")
        }
    }

    pub fn eval(&mut self, value: Gc<Value>) -> Result<Gc<Value>> {
        let pinned = value.pin();
        match &*pinned {
            Value::Symbol(sym) => {
                if let Some(value) = self.scope_stack().pin().lookup(sym) {
                    Ok(value)
                } else {
                    Err(self.err_by(pinned.id(), EvalErrorKind::NoSuchVariable(sym.clone())))
                }
            }
            Value::Cons(head, tail) => self.eval_call(Gc::clone(head), Gc::clone(tail)),
            // The rest is self-evaluating
            _ => Ok(value),
        }
    }

    pub fn eval_call(&mut self, head: Gc<Value>, tail: Gc<Value>) -> Result<Gc<Value>> {
        let head = self.eval(head)?.pin();
        match &*head {
            Value::PrimOp(PrimOp(f)) =>
                f(self, tail),
            Value::Closure(_cl) => {
                unimplemented!()
            }
            _ => Err(self.err_by(head.id(), EvalErrorKind::Uncallable)),
        }
    }
}

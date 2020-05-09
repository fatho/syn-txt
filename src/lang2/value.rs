
use std::{cell::{RefCell}, collections::HashMap, rc::Rc, fmt};
use crate::rational::Rational;
use super::{Gc, Trace};
use super::interpreter;

/// A symbolic value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol(Rc<str>);

impl Symbol {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// TODO: eventually, only allow symbol creation by interning.
impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol(s.into())
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol(s.into())
    }
}

impl From<Rc<str>> for Symbol {
    fn from(s: Rc<str>) -> Self {
        Symbol(s)
    }
}

/// A primitive operation exposed to the interpreted language.
#[derive(Copy, Clone)]
pub struct PrimOp(pub for<'a> fn(&mut interpreter::Interpreter, Gc<Value>) -> interpreter::Result<Gc<Value>>);

impl fmt::Debug for PrimOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.0 as *const ();
        write!(f, "PrimOp({:p})", ptr)
    }
}

impl PartialEq for PrimOp {
    fn eq(&self, other: &Self) -> bool {
        let self_ptr = self.0 as *const ();
        let other_ptr = other.0 as *const ();
        self_ptr == other_ptr
    }
}
impl Eq for PrimOp {}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A symbol
    Symbol(Symbol),
    /// A keyword
    Keyword(Symbol),
    /// A string
    Str(String),
    /// A float
    Float(f64),
    /// A rational number
    Ratio(Rational),
    /// An integer
    Int(i64),
    /// A boolean
    Bool(bool),
    /// Value returned by statements that are not expressions
    Void,
    /// The empty list, nil
    Nil,
    /// A cons cell, used for creating lists of values.
    Cons(Gc<Value>, Gc<Value>),
    /// Closure that can be called
    Closure(Gc<Closure>),
    /// Primitive operation.
    PrimOp(PrimOp),
}

impl Value {
    pub fn is_symbol(&self) -> bool {
        match self {
            Value::Symbol(_) => true,
            _ => false,
        }
    }
    pub fn is_keyword(&self) -> bool {
        match self {
            Value::Keyword(_) => true,
            _ => false,
        }
    }
    pub fn is_str(&self) -> bool {
        match self {
            Value::Str(_) => true,
            _ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            Value::Float(_) => true,
            _ => false,
        }
    }
    pub fn is_ratio(&self) -> bool {
        match self {
            Value::Ratio(_) => true,
            _ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            Value::Int(_) => true,
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Value::Bool(_) => true,
            _ => false,
        }
    }
    pub fn is_void(&self) -> bool {
        match self {
            Value::Void => true,
            _ => false,
        }
    }
    pub fn is_nil(&self) -> bool {
        match self {
            Value::Nil => true,
            _ => false,
        }
    }
    pub fn is_cons(&self) -> bool {
        match self {
            Value::Cons(_, _) => true,
            _ => false,
        }
    }
    pub fn is_closure(&self) -> bool {
        match self {
            Value::Closure(_) => true,
            _ => false,
        }
    }
    pub fn is_primop(&self) -> bool {
        match self {
            Value::PrimOp(_) => true,
            _ => false,
        }
    }
}

impl Trace for Value {
    fn mark(&self) {
        match self {
            Value::Str(_) =>{},
            Value::Float(_) => {}
            Value::Ratio(_) => {}
            Value::Int(_) => {}
            Value::Bool(_) => {}
            Value::Void => {}
            Value::Nil => {}
            Value::Cons(head, tail) => {
                Gc::mark(head);
                Gc::mark(tail);
            },
            Value::Closure(clos) => clos.mark(),
            Value::Symbol(_) => {}
            Value::Keyword(_) => {}
            Value::PrimOp(_) => {}
        }
    }
}

/// A closure is a piece of code that has captured its environment and
/// can be called with arguments.
#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    /// The current scope stack at the point when the closure was made.
    /// Note: mutating variables in these scopes affects the closure as well.
    pub captured_scope: Gc<Scope>,
    /// Name of the parameters that must be passed to the closure when calling it.
    /// The names must be unique.
    pub parameters: Vec<Symbol>,
    /// The code to execute when calling the closure
    /// The value of the last expression becomes the return value.
    pub body: Vec<Value>,
}

impl Trace for Closure {
    fn mark(&self) {
        self.captured_scope.mark();
        self.body.iter().for_each(Value::mark);
    }
}
/// A binding scope for variables.
/// Scopes are lexially nested, and inner scopes have precedence before outer scopes.
#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    // TODO: maybe store small values (Int, Bool, etc.) inline after all?
    bindings: RefCell<HashMap<Symbol, Gc<Value>>>,
    outer: Option<Gc<Scope>>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: RefCell::new(HashMap::new()),
            outer: None,
        }
    }

    /// Create a nested scope inside the given outer scope.
    pub fn nest(outer: Gc<Scope>) -> Self {
        Self {
            bindings: RefCell::new(HashMap::new()),
            outer: Some(outer),
        }
    }

    /// Return a reference to the lexically outer scope, if this scope is not the outermost.
    pub fn outer(&self) -> Option<Gc<Scope>> {
        self.outer.clone()
    }

    /// Define a variable in this scope, if possible.
    /// On success, it returns `None`, otherwise it gives the arguments back to the caller.
    /// NOTE: `define`, unlike set, does not operate recursively on outer scopes.
    pub fn define(&self, var: Symbol, value: Gc<Value>) -> Option<(Symbol, Gc<Value>)> {
        let mut here = self.bindings.borrow_mut();
        if here.get(&var).is_none() {
            here.insert(var, value);
            None
        } else {
            Some((var, value))
        }
    }

    /// Set a variable in the scope where it was defined.
    /// If the variable was not defined, the `value` argument is returned as `Err`,
    /// otherwise, the previous value is returned as `Ok`.
    pub fn set(&self, var: &Symbol, value: Gc<Value>) -> Result<Gc<Value>, Gc<Value>> {
        let mut here = self.bindings.borrow_mut();
        if let Some(slot) = here.get_mut(var) {
            Ok(std::mem::replace(slot, value))
        } else if let Some(outer) = self.outer.as_ref() {
            outer.pin().set(var, value)
        } else {
            Err(value)
        }
    }

    /// Return a copy of the value of the given variable, or `None` if it was not defined.
    pub fn lookup(&self, var: &Symbol) -> Option<Gc<Value>> {
        let here = self.bindings.borrow();
        if let Some(value) = here.get(var) {
            Some(Gc::clone(value))
        } else if let Some(outer) = self.outer.as_ref() {
            outer.pin().lookup(var)
        } else {
            None
        }
    }
}

impl Trace for Scope {
    fn mark(&self) {
        self.outer.iter().for_each(Gc::mark);
        self.bindings.borrow().iter().for_each(|(_, value)| value.mark());
    }
}

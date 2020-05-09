
use std::{cell::{RefCell}, collections::HashMap, rc::Rc};
use crate::rational::Rational;
use super::heap;

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
    Cons(heap::Gc<Value>, heap::Gc<Value>),
    /// Closure that can be called
    Closure(heap::Gc<Closure>),
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
}

impl heap::Trace for Value {
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
                heap::Gc::mark(head);
                heap::Gc::mark(tail);
            },
            Value::Closure(clos) => clos.mark(),
            Value::Symbol(_) => {}
            Value::Keyword(_) => {}
        }
    }
}

/// A closure is a piece of code that has captured its environment and
/// can be called with arguments.
#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    /// The current scope stack at the point when the closure was made.
    /// Note: mutating variables in these scopes affects the closure as well.
    pub captured_scope: ScopeRef,
    /// Name of the parameters that must be passed to the closure when calling it.
    /// The names must be unique.
    pub parameters: Vec<Symbol>,
    /// The code to execute when calling the closure
    /// The value of the last expression becomes the return value.
    pub body: Vec<Value>,
}

impl heap::Trace for Closure {
    fn mark(&self) {
        self.captured_scope.mark();
        self.body.iter().for_each(Value::mark);
    }
}

/// A reference to a shared scope.
pub type ScopeRef = heap::Gc<RefCell<Scope>>;

/// A binding scope for variables.
/// Scopes are lexially nested, and inner scopes have precedence before outer scopes.
#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    bindings: HashMap<Symbol, Value>,
    outer: Option<ScopeRef>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            outer: None,
        }
    }

    /// Wrap this scope into a `ScopeRef`.
    pub fn into_ref(self, heap: &mut heap::Heap) -> ScopeRef {
        heap.alloc(RefCell::new(self))
    }

    /// Create a nested scope inside the given outer scope.
    pub fn nest(outer: ScopeRef) -> Self {
        Self {
            bindings: HashMap::new(),
            outer: Some(outer),
        }
    }

    /// Return a reference to the lexically outer scope, if this scope is not the outermost.
    pub fn outer(&self) -> Option<ScopeRef> {
        self.outer.clone()
    }

    /// Define a variable in this scope, if possible.
    /// On success, it returns `None`, otherwise it gives the arguments back to the caller.
    /// NOTE: `define`, unlike set, does not operate recursively on outer scopes.
    pub fn define(&mut self, var: Symbol, value: Value) -> Option<(Symbol, Value)> {
        if self.bindings.get(&var).is_none() {
            self.bindings.insert(var, value);
            None
        } else {
            Some((var, value))
        }
    }

    /// Set a variable in the scope where it was defined.
    /// If the variable was not defined, the `value` argument is returned as `Err`,
    /// otherwise, the previous value is returned as `Ok`.
    pub fn set(&mut self, var: &Symbol, value: Value) -> Result<Value, Value> {
        if let Some(slot) = self.bindings.get_mut(var) {
            Ok(std::mem::replace(slot, value))
        } else if let Some(outer) = self.outer.as_ref() {
            outer.pin().borrow_mut().set(var, value)
        } else {
            Err(value)
        }
    }

    /// Return a copy of the value of the given variable, or `None` if it was not defined.
    pub fn lookup(&self, var: &Symbol) -> Option<Value> {
        if let Some(value) = self.bindings.get(var) {
            Some(value.clone())
        } else if let Some(outer) = self.outer.as_ref() {
            outer.pin().borrow().lookup(var)
        } else {
            None
        }
    }
}

impl heap::Trace for Scope {
    fn mark(&self) {
        self.outer.iter().for_each(heap::Gc::mark);
        self.bindings.iter().for_each(|(_, value)| value.mark());
    }
}

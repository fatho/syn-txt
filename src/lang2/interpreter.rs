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

pub struct Interpreter<'a> {
    /// Read-only scope (from the perspective of the language)
    /// providing all the built-in primops.
    builtins: Gc<Scope>,
    /// Points to the current innermost scope
    scope_stack: Gc<Scope>,
    /// The interpreter heap.
    heap: &'a mut Heap,
    /// Debug information about values.
    debug_info: &'a mut debug::DebugTable,
}

impl<'a> Interpreter<'a> {
    pub fn new(heap: &'a mut Heap, debug_info: &'a mut debug::DebugTable) -> Self {
        let builtin_scope = Scope::new();

        let prim = vec![
            // syntax
            ("begin", PrimOp(primops::begin)),
            ("define", PrimOp(primops::define)),
            ("lambda", PrimOp(primops::lambda)),
            ("set!", PrimOp(primops::set)),
            ("if", PrimOp(primops::if_)),
            // arithmetic
            ("+", PrimOp(primops::add)),
            ("-", PrimOp(primops::sub)),
            ("*", PrimOp(primops::mul)),
            ("/", PrimOp(primops::div)),
            // relational
            ("=", PrimOp(primops::eq)),
            ("!=", PrimOp(primops::neq)),
            ("<", PrimOp(primops::lt)),
            ("<=", PrimOp(primops::leq)),
            (">", PrimOp(primops::gt)),
            (">=", PrimOp(primops::geq)),
            // lists
            ("list", PrimOp(primops::list)),
            ("cons", PrimOp(primops::cons)),
            ("head", PrimOp(primops::head)),
            ("tail", PrimOp(primops::tail)),
            ("cons?", PrimOp(primops::is_cons)),
            ("nil?", PrimOp(primops::is_nil)),
            ("concat", PrimOp(primops::concat)),
            ("reverse", PrimOp(primops::reverse)),
            ("for-each", PrimOp(primops::for_each)),
            ("map", PrimOp(primops::map)),
            // ("range", PrimOp(primops::range)),
            // // dicts
            // ("dict", PrimOp(primops::dict)),
            // ("update", PrimOp(primops::dict_update)),
            // ("get", PrimOp(primops::dict_get)),
            // util
            ("print", PrimOp(primops::print)),
        ];
        let constants = vec![
            // syntax
            ("nil", Value::Nil),
        ];

        for (name, fun) in prim {
            builtin_scope.define(name.into(), heap.alloc(Value::PrimOp(fun)));
        }
        for (name, c) in constants {
            builtin_scope.define(name.into(), heap.alloc(c));
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

    pub fn make_error(&self, cause: Id, kind: EvalErrorKind) -> EvalError {
        let location = self.debug_info.get_location(cause);
        EvalError::new(location.cloned(), kind)
    }

    pub fn heap_alloc_value(&mut self, value: Value) -> Gc<Value> {
        // TODO: share heap allocation for small values
        self.heap.alloc(value)
    }

    pub fn heap_alloc<T: Trace + std::fmt::Debug + 'static>(&mut self, value: T) -> Gc<T> {
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
                    Err(self.make_error(pinned.id(), EvalErrorKind::NoSuchVariable(sym.clone())))
                }
            }
            Value::Cons(head, tail) => self.eval_call(Gc::clone(head), Gc::clone(tail)),
            // The rest is self-evaluating
            _ => Ok(value),
        }
    }

    pub fn eval_call(&mut self, head: Gc<Value>, mut tail: Gc<Value>) -> Result<Gc<Value>> {
        let head = self.eval(head)?.pin();
        match &*head {
            Value::PrimOp(PrimOp(f)) =>
                f(self, tail),
            Value::Closure(gc_closure) => {
                let clos = gc_closure.pin();
                // Create a new scope inside the captured scope and define the arguments
                let scope_stack = Scope::nest(Gc::clone(&clos.captured_scope));
                for param_var in clos.parameters.iter() {
                    let value = self.pop_argument_eval(&mut tail)?;
                    if let Some((var, _)) = scope_stack.define(param_var.clone(), value)
                    {
                        // the `lambda` prim op ensures that the parameter names are unique,
                        // but the interpreter host might have sneaked in an invalid closure.
                        // TODO: ensure invariants in `Closure`
                        return Err(EvalError::new(None, EvalErrorKind::Other(format!(
                                "invariant violated: closure redefined parameter name {}",
                                var.as_str()
                            )),
                        ));
                    }
                }

                // switch out stack, and switch back in the end
                let closure_scope = self.heap_alloc(scope_stack);
                let previous_stack = std::mem::replace(&mut self.scope_stack, closure_scope);

                let mut return_value = Ok(self.heap_alloc_value(Value::Void));

                let mut current = clos.body.pin();
                while let Value::Cons(head, tail) = &*current {
                    return_value = self.eval(Gc::clone(head));
                    if return_value.is_err() { break }
                    current = tail.pin();
                }

                // ensure that we always switch back the original scope
                std::mem::replace(&mut self.scope_stack, previous_stack);

                return_value
            }
            _ => Err(self.make_error(head.id(), EvalErrorKind::Uncallable)),
        }
    }


    pub fn pop_argument(&mut self, args: &mut Gc<Value>) -> Result<Gc<Value>> {
        if let Value::Cons(head, tail) = &*args.pin() {
            std::mem::replace(args, Gc::clone(tail));
            Ok(Gc::clone(head))
        } else {
            Err(self.make_error(args.id(), EvalErrorKind::NotEnoughArguments))
        }
    }

    pub fn pop_argument_eval(&mut self, args: &mut Gc<Value>) -> Result<Gc<Value>> {
        let arg = self.pop_argument(args)?;
        self.eval(arg)
    }

    pub fn as_symbol(&self, arg: &Gc<Value>) -> Result<Symbol> {
        if let Value::Symbol(sym) = &*arg.pin() {
            Ok(sym.clone())
        } else {
            Err(self.make_error(arg.id(), EvalErrorKind::IncompatibleArguments))
        }
    }

    pub fn expect_no_more_arguments(&mut self, args: &Gc<Value>) -> Result<()> {
        if let Value::Nil = &*args.pin() {
            Ok(())
        } else {
            Err(self.make_error(args.id(), EvalErrorKind::TooManyArguments))
        }
    }
}



#[cfg(test)]
mod test {
    use super::super::{lexer::*, parser::*, span::*, debug::*, compiler};
    use super::*;
    use crate::rational::*;

    fn compile(input: &str) -> (Vec<Gc<Value>>, Heap, DebugTable) {
        let tokens = Lexer::new(input)
            .collect::<std::result::Result<Vec<(Span, Token)>, _>>()
            .unwrap();
        let ast = Parser::new(input, &tokens).parse().unwrap();
        let mut heap = Heap::new();
        let mut debug = debug::DebugTable::new();
        let mut context = compiler::Context {
            debug_table: &mut debug,
            heap: &mut heap,
            filename: "<input>".into(),
        };
        let values: Vec<Gc<Value>> = ast.iter().map(|e| context.compile(e)).collect();
        (values, heap, debug)
    }

    fn expect_values(input: &str, expected: &[Value]) {
        let (vals, mut heap, mut debug) = compile(input);
        let mut interp = Interpreter::new(&mut heap, &mut debug);

        for (e, val) in vals.iter().zip(expected) {
            let result = interp.eval(Gc::clone(e)).unwrap();
            assert_eq!(&*result.pin(), val);
        }
    }

    fn expect_values_or_errors(input: &str, expected: &[std::result::Result<Value, EvalErrorKind>]) {
        let (vals, mut heap, mut debug) = compile(input);
        let mut interp = Interpreter::new(&mut heap, &mut debug);

        for (e, val) in vals.iter().zip(expected) {
            let result = interp.eval(Gc::clone(e)).map(|v| v.pin());
            let result = result.map(|v| (*v).clone()).map_err(|e| e.info);
            assert_eq!(&result, val);
        }
    }

    #[test]
    fn test_arithmetic() {
        expect_values("(+ 1 2)", &[Value::Int(3)]);
        expect_values("(- 8 12)", &[Value::Int(-4)]);

        expect_values("(- -4 -9)", &[Value::Int(5)]);
        expect_values("(- 7)", &[Value::Int(-7)]);

        expect_values("(* 2 (/ 8 12))", &[Value::Ratio(Rational::new(4, 3))]);
        expect_values("(/ 5/4 8/7)", &[Value::Ratio(Rational::new(35, 32))]);
        expect_values("(/ 7)", &[Value::Ratio(Rational::new(1, 7))]);
    }

    #[test]
    fn test_defines() {
        expect_values(
            r#"
            (define pi 3.14)
            (define r (/ 5. 2))
            (define result
                (* r (* 2 pi)))
            result
            (set! result
                (* pi (* r r)))
            result"#,
            &[
                Value::Void,
                Value::Void,
                Value::Void,
                Value::Float(15.700000000000001),
                Value::Void,
                Value::Float(19.625),
            ],
        );
    }

    #[test]
    fn test_scopes() {
        expect_values_or_errors(
            r#"
            (define pi 3.14)
            (define val 5)
            val
            (define area
                (begin
                    (define r 1.0)
                    (set! val (* pi (* r r)))
                    val
                ))
            r
            val
            ; ensure that an error inside a nested scope pops that scope
            (begin
                (define foo 1) ; we expect this definition to be cleaned up
                (set! bar 1) ; the error occurs here
            )
            foo
            "#,
            &[
                Ok(Value::Void),
                Ok(Value::Void),
                Ok(Value::Int(5)),
                Ok(Value::Void),
                Err(EvalErrorKind::NoSuchVariable("r".into())),
                Ok(Value::Float(3.14)),
                Err(EvalErrorKind::NoSuchVariable("bar".into())),
                Err(EvalErrorKind::NoSuchVariable("foo".into())),
            ],
        )
    }

    #[test]
    fn test_closure_stateless() {
        expect_values(
            r#"
            (define plus-one
                (lambda (x) (+ x 1)))
            (plus-one 2)
            (plus-one 3)
            "#,
            &[Value::Void, Value::Int(3), Value::Int(4)],
        )
    }

    /// Test that closures can capture global state and any mutations
    /// from either inside or outside the closure can be seen elsewhere.
    #[test]
    fn test_closure_global_state() {
        expect_values(
            r#"
            (define global-state 0)
            (define (get-global)
                (define ret global-state)
                (set! global-state (+ ret 1))
                ret
            )
            (get-global)
            (get-global)
            (set! global-state 10)
            (get-global)
            global-state
            "#,
            &[
                Value::Void,
                Value::Void,
                Value::Int(0),
                Value::Int(1),
                Value::Void,
                Value::Int(10),
                Value::Int(11),
            ],
        )
    }

    /// Test that closures can capture scopes that are subsequently popped,
    /// never to be seen again.
    #[test]
    fn test_closure_hidden_state() {
        expect_values(
            r#"
            ; A closure that, when called, creates a new fresh closure
            (define make-counter
                (lambda (initial-count)
                    (begin
                        (define counter initial-count)
                        (lambda ()
                            (begin
                                (define count counter)
                                (set! counter (+ 1 count))
                                count
                            ))
                    )))

            (define c1 (make-counter 0))
            (define c2 (make-counter 3))
            (c1)
            (c1)
            (c2)
            (c2)
            (c1)
            "#,
            &[
                Value::Void,
                Value::Void,
                Value::Void,
                Value::Int(0),
                Value::Int(1),
                Value::Int(3),
                Value::Int(4),
                Value::Int(2),
            ],
        )
    }

    #[test]
    fn test_list() {
        expect_values("(list)", &[Value::Nil]);

        expect_values(r#"
            (define (sum l)
                (if (cons? l)
                    (+ (head l) (sum (tail l)))
                    0
                )
            )
            (define foo (list 1 2 3))
            (sum foo)
            (= foo (reverse (list 3 2 1)))
            (=
                (map (lambda (x) (+ 1 x)) (list 1 2 3))
                (list 2 3 4)
            )
            (= (list 1 2) (list 1))
            (> (list 1 2) (list 1))
            ; TODO: move to separate test:
            (< "apocryphal" "auxiliar")
            (=
                (concat (list 1 2) (list 3) (list) (reverse (list 1 2)))
                (list 1 2 3 2 1)
            )
            "#,
            &[
                Value::Void,
                Value::Void,
                Value::Int(6),
                Value::Bool(true),
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
                Value::Bool(true),
                Value::Bool(true),
            ],
        );
        // expect_values(
        //     "(range 1 4)",
        //     &[Value::List(
        //         vec![Value::Int(1), Value::Int(2), Value::Int(3)].into(),
        //     )],
        // );
        // expect_values(
        //     "(range 1 -2 -1)",
        //     &[Value::List(
        //         vec![Value::Int(1), Value::Int(0), Value::Int(-1)].into(),
        //     )],
        // );
        // expect_values(
        //     "(range 3)",
        //     &[Value::List(
        //         vec![Value::Int(0), Value::Int(1), Value::Int(2)].into(),
        //     )],
        // );
        // expect_values("(range 0)", &[Value::List(vec![].into())]);
    }

    // #[test]
    // fn test_dict() {
    //     expect_values("(dict)", &[Value::Dict(Rc::new(HashMap::new()))]);
    //     expect_values(
    //         "
    //         (define d (dict :foo 1 :bar 2))
    //         d
    //         (get d :foo)
    //         (define d2 (update d :foo 4))
    //         (get d :foo)
    //         (get d2 :foo)
    //         (get d2 :bar)
    //         (get d2 :baz)
    //         ",
    //         &[
    //             Value::Void,
    //             Value::Dict({
    //                 let mut d = HashMap::new();
    //                 d.insert(":foo".into(), Value::Int(1));
    //                 d.insert(":bar".into(), Value::Int(2));
    //                 Rc::new(d)
    //             }),
    //             Value::Int(1),
    //             Value::Void,
    //             Value::Int(1),
    //             Value::Int(4),
    //             Value::Int(2),
    //         ],
    //     );
    //     expect_values_or_errors(
    //         "
    //         (define d (dict :foo 1 :bar 2))
    //         (get d :baz)
    //         ",
    //         &[
    //             Ok(Value::Void),
    //             Err(IntpErrInfo::UnknownKeyword(":baz".into())),
    //         ],
    //     );
    // }
}

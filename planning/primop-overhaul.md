# Reworking Primops

Notes about the planned rewrite of primitive operations.

## The Problem

The current primop implementation conflates to concepts: functions and syntax.
Every primop currently gets access to the AST of its calling list.
However, this abstraction falls apart when there is no calling list,
because the call happened implicitly in another primitive function (such as map).

The current workaround is to define a temporary scope with fresh variables to hold the argument values,
and then performing the actual call on these synthesized AST bits.
This is both ugly and slow.

## Idea 1

Split up the `PrimOp` type in a `SyntaxExtension` and `PrimFn` type.
The `SyntaxExtension` interface will work exactly like the `PrimOp` interface works now,
i.e. all implementations get full access to the AST. However, syntax extensions cannot
be used in a higher-order context (which is nonsensical anyway for non-metaprogramming purposes,
and even then there are probably other ways). This means that something like

```scheme
(map define (list ...))
```

will not work (and that's good).

On the other hand, the new `PrimFn` will get a restricted interface that only allows
getting arguments as values and keywords, conceptually something like this

```rust
pub struct PrimFn(
    pub fn(&mut Interpreter, &[ValueOrKeyword]) -> InterpreterResult<Value>
);

pub enum ValueOrKeyword {
    Value(Value),
    Keyword(Ident),
}
```

Calling this in a higher-order function such as map is trivial,
and does not require any hacks like the current implementation.

## Idea 2

Keep the current `ArgParser` interface, but allow it to either process
a list of symbolic expressions, or a list of values, whatever is more
convenient for the caller.

The advantages of this idea vs idea 1 are:
- the changes are localized in one place and don't need a big overhaul
- the `ArgParser` interface could use some streamlining anyway
- there are fewer kinds of values

In contrast to option 1, this would allow expressions like

```scheme
(map define (list ...))
```

but thinking further about this, this might actually make sense
in the presence of quoting and unquoting.
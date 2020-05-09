

// re-export the syntax part of the old interpreter, we're not changing that
pub use super::lang::{ast, lexer, parser, span};
pub mod heap;
pub mod value;
pub mod interpreter;
pub mod compiler;

pub use value::*;
pub use heap::*;

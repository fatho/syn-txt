

use std::{rc::Rc, collections::HashMap};
use super::span::Span;
use super::heap;

/// Location referring to a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: Rc<str>,
    pub span: Span,
}

/// Debug information for the values generated during compilation.
pub struct DebugTable(HashMap<heap::Id, DebugInfo>);

impl DebugTable {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_location(&self, value: heap::Id) -> Option<&SourceLocation> {
        self.0.get(&value).and_then(|entry| entry.location.as_ref())
    }

    pub fn insert(&mut self, value: heap::Id, info: DebugInfo) {
        self.0.insert(value, info);
    }
}

pub struct DebugInfo {
    pub location: Option<SourceLocation>
}

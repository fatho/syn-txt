use super::heap;
use super::span::Span;
use std::collections::HashMap;
use std::sync::Arc;

/// Location referring to a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: Arc<str>,
    pub span: Span,
}

/// Debug information for the values generated during compilation,
/// associated with a specific heap.
pub struct DebugTable {
    value_info: HashMap<heap::Id, DebugInfo>,
    sources: HashMap<Arc<str>, Arc<str>>,
}

impl DebugTable {
    pub fn new() -> Self {
        Self {
            value_info: HashMap::new(),
            sources: HashMap::new(),
        }
    }

    pub fn get_location(&self, value: heap::Id) -> Option<&SourceLocation> {
        self.value_info
            .get(&value)
            .and_then(|entry| entry.location.as_ref())
    }

    pub fn get_source(&self, filename: &str) -> Option<&str> {
        self.sources.get(filename).map(|r| r.as_ref())
    }

    pub fn insert(&mut self, value: heap::Id, info: DebugInfo) {
        self.value_info.insert(value, info);
    }

    pub fn insert_source(&mut self, filename: Arc<str>, source: Arc<str>) {
        self.sources.insert(filename, source);
    }
}

pub struct DebugInfo {
    pub location: Option<SourceLocation>,
}

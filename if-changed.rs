//! Check that the files of a repository change together. A block that follows an
//! `if-changed` comment needs a change in each file that its `then-change` comment names.

mod engine;
mod parser;

/// Helpers for tests.
pub mod testing;

use std::path::PathBuf;

pub use engine::{Engine, GitEngine};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
struct Pattern {
    pub name: Option<String>,
    pub value: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(serde::Serialize))]
struct IfChangedBlock {
    pub name: Option<String>,
    pub range: (usize, usize),
    pub patterns: Vec<Pattern>,
}

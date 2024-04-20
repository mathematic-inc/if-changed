mod engine;
mod parser;

use std::path::PathBuf;

pub use engine::{git, Engine};

#[derive(Debug, Clone)]
struct Pattern {
    pub name: Option<String>,
    pub value: PathBuf,
    pub line: usize,
}

#[derive(Debug, Clone)]
struct IfChangedBlock {
    pub name: Option<String>,
    pub range: (usize, usize),
    pub patterns: Vec<Pattern>,
}

#[cfg(test)]
mod testing;

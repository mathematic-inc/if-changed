mod checker;
mod engine;
mod parser;

pub use engine::{git, Engine};

#[derive(Debug, Clone)]
struct NamedPattern {
    pub name: Option<String>,
    pub pattern: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
struct IfChangedBlock {
    pub name: Option<String>,
    pub range: (usize, usize),
    pub patterns: Vec<NamedPattern>,
}

#[cfg(test)]
mod testing;

mod git;

use std::path::{Path, PathBuf};

pub use git::git;

use super::checker::Checker;

pub trait Engine {
    /// Get the paths that have been changed.
    fn changed_paths(&self) -> impl Iterator<Item = PathBuf>;

    /// Get the paths that match a given pattern.
    fn paths(&self, pattern: &str) -> impl Iterator<Item = PathBuf>;

    /// Resolve a path relative to an absolute path.
    fn resolve(&self, path: impl AsRef<Path>) -> PathBuf;

    /// Check if a file has been ignored.
    fn is_ignored(&self, path: impl AsRef<Path>) -> bool;

    /// Check if a range of lines in a file has been modified.
    fn is_range_modified(&self, path: impl AsRef<Path>, range: (usize, usize)) -> bool;

    /// Check a file for dependent changes.
    fn check(&self, path: impl AsRef<Path>) -> Result<(), Vec<String>> {
        Checker::new(self, path.as_ref())?.check()
    }
}

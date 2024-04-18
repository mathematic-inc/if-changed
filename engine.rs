mod git;

use std::path::{Path, PathBuf};

pub use git::git;

use super::checker::Checker;

pub trait Engine {
    /// Get the root path of the engine.
    fn root(&self) -> &Path;

    /// Get the paths that have been changed.
    fn changed_paths(&self) -> impl Iterator<Item = &PathBuf>;

    /// Get the paths that match a given pattern.
    fn matched_paths(&self, pattern: &str) -> impl Iterator<Item = PathBuf>;

    /// Check if a file has been modified.
    fn is_modified(&self, path: &Path) -> bool;

    /// Check if a file has been ignored.
    fn is_ignored(&self, path: &Path) -> bool;

    /// Check if a range of lines in a file has been modified.
    fn is_range_modified(&self, path: &Path, range: (usize, usize)) -> bool;

    /// Check a file for dependent changes.
    fn check(&self, path: impl AsRef<Path>) -> Result<(), Vec<String>> {
        Checker::new(self, path.as_ref())?.check()
    }
}

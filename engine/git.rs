use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use bstr::ByteSlice;

use super::Engine;

const IF_CHANGED_IGNORE_TRAILER: &[u8] = b"ignore-if-changed";

pub fn git<'repo>(
    repository: &'repo git2::Repository,
    from_ref: Option<&str>,
    to_ref: Option<&str>,
) -> impl Engine + 'repo {
    GitEngine::new(repository, from_ref, to_ref)
}

struct GitEngine<'a> {
    root: PathBuf,
    changed_paths: HashSet<PathBuf>,
    ignore_pathspec: Option<git2::Pathspec>,
    repository: &'a git2::Repository,
    from_tree: git2::Tree<'a>,
    to_tree: Option<git2::Tree<'a>>,
}

impl<'a> GitEngine<'a> {
    fn new(repository: &'a git2::Repository, from_ref: Option<&str>, to_ref: Option<&str>) -> Self {
        let ignore_pathspec = ignore_pathspec(to_ref, repository);

        let from_tree = repository
            .revparse_single(from_ref.unwrap_or("HEAD"))
            .expect("from_ref is not a valid revision")
            .peel_to_tree()
            .expect("from_ref does not point to a tree");

        let to_tree = to_ref.map(|to_ref| {
            repository
                .revparse_single(to_ref)
                .unwrap()
                .peel_to_tree()
                .unwrap()
        });

        let changed_paths = {
            let mut options = git2::DiffOptions::new();
            match &to_tree {
                Some(to_tree) => repository.diff_tree_to_tree(
                    Some(&from_tree),
                    Some(to_tree),
                    Some(&mut options),
                ),
                None => {
                    repository.diff_tree_to_workdir_with_index(Some(&from_tree), Some(&mut options))
                }
            }
            .unwrap()
            .deltas()
            .map(|delta| delta.new_file().path().unwrap().to_owned())
            .collect::<HashSet<_>>()
        };

        let root = repository
            .workdir()
            .expect("bare repos are not supported")
            .canonicalize()
            .unwrap();

        Self {
            root,
            ignore_pathspec,
            changed_paths,
            repository,
            from_tree,
            to_tree,
        }
    }

    /// Get the diff of a file, if any.
    fn get_diff(&self, path: &Path) -> git2::Diff {
        let mut options = git2::DiffOptions::new();
        options.pathspec(path).disable_pathspec_match(true);
        match &self.to_tree {
            Some(to_tree) => self.repository.diff_tree_to_tree(
                Some(&self.from_tree),
                Some(to_tree),
                Some(&mut options),
            ),
            None => self
                .repository
                .diff_tree_to_workdir_with_index(Some(&self.from_tree), Some(&mut options)),
        }
        .unwrap_or_else(|_| panic!("failed to diff {}", path.display()))
    }

    /// Get the patch of a file, if any.
    fn get_patch(&self, path: &Path) -> Option<git2::Patch> {
        git2::Patch::from_diff(&self.get_diff(path), 0)
            .ok()
            .flatten()
    }
}

impl Engine for GitEngine<'_> {
    fn root(&self) -> &Path {
        &self.root
    }

    fn changed_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.changed_paths.iter()
    }

    fn matched_paths(&self, mut spec: &str) -> impl Iterator<Item = PathBuf> {
        if spec.starts_with('/') {
            spec = &spec[1..];
        }
        let pathspec = git2::Pathspec::new([spec]).unwrap();
        let matches = pathspec
            .match_workdir(self.repository, git2::PathspecFlags::DEFAULT)
            .expect("bare repos are not supported");
        matches
            .entries()
            .map(|entry| Path::new(&entry.to_os_str_lossy()).to_owned())
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn is_modified(&self, path: &Path) -> bool {
        self.changed_paths.contains(path)
    }

    fn is_ignored(&self, path: &Path) -> bool {
        if let Some(path_spec) = &self.ignore_pathspec {
            if path_spec.matches_path(path, git2::PathspecFlags::DEFAULT) {
                return true;
            }
        }
        false
    }

    fn is_range_modified(&self, path: &Path, range: (usize, usize)) -> bool {
        let Some(patch) = self.get_patch(path) else {
            return false;
        };
        for (hunk_index, hunk) in (0..patch.num_hunks()).map(|i| (i, patch.hunk(i).unwrap().0)) {
            if usize::try_from(hunk.new_start()).unwrap() > range.1 {
                break;
            }
            if usize::try_from(hunk.new_start() + hunk.new_lines()).unwrap() < range.0 {
                continue;
            }
            for line in (0..patch.num_lines_in_hunk(hunk_index).unwrap())
                .map(|i| patch.line_in_hunk(hunk_index, i).unwrap())
            {
                match line.origin() {
                    '+' if {
                        let line_no = usize::try_from(line.new_lineno().unwrap()).unwrap();
                        line_no >= range.0 && line_no <= range.1
                    } =>
                    {
                        return true;
                    }
                    '-' if {
                        let line_no = usize::try_from(line.old_lineno().unwrap()).unwrap();
                        line_no >= range.0 && line_no <= range.1
                    } =>
                    {
                        return true;
                    }
                    _ => {
                        continue;
                    }
                }
            }
        }
        false
    }
}

fn ignore_pathspec(to_ref: Option<&str>, repository: &git2::Repository) -> Option<git2::Pathspec> {
    let to_ref = to_ref?;

    let commit = repository
        .revparse_single(to_ref)
        .ok()?
        .peel_to_commit()
        .ok()?;
    let trailers = git2::message_trailers_bytes(commit.message_bytes()).ok()?;
    let pathspecs = trailers
        .iter()
        .filter(|(name, _)| name.to_ascii_lowercase() == IF_CHANGED_IGNORE_TRAILER)
        .flat_map(|(_, value)| split_pathspecs(value))
        .collect::<Vec<_>>();
    if pathspecs.is_empty() {
        None
    } else {
        Some(git2::Pathspec::new(pathspecs).expect("Ignore-if-changed is invalid."))
    }
}

fn split_pathspecs(value: &[u8]) -> impl Iterator<Item = &[u8]> {
    value
        .split_once_str(b"--")
        .unwrap_or((value, b""))
        .0
        .split_str(b",")
        .map(|s| s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::git_test;

    macro_rules! extract_pathspecs_test {
        ($name:ident, $val:expr) => {
            #[test]
            fn $name() {
                insta::assert_debug_snapshot!(split_pathspecs($val)
                    .map(|value| value.to_str().unwrap())
                    .collect::<Vec<_>>());
            }
        };
    }

    extract_pathspecs_test!(test_basic_pathspec, b"a");
    extract_pathspecs_test!(test_multiple_pathspec, b"a/b, b/c");
    extract_pathspecs_test!(
        test_multiple_pathspec_with_comment,
        b"a/b, b/c -- Hello world!"
    );
    extract_pathspecs_test!(test_multiple_pathspec_with_empty_comment, b"a/b, b/c --");

    #[test]
    fn test_git() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "b" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        insta::assert_debug_snapshot!(engine.changed_paths().collect::<Vec<_>>(), @"[]");
        insta::assert_debug_snapshot!(engine.matched_paths("a").collect::<Vec<_>>(), @r###"
        [
            "a",
        ]
        "###);
        assert!(!engine.is_ignored(Path::new("a")));
    }

    #[test]
    #[should_panic]
    fn test_git_without_head() {
        let (_tempdir, repo) = git_test! {
           staged: ["a" => "a", "b" => "b"]
        };

        git(&repo, None, None);
    }

    #[test]
    fn test_matched_paths() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        insta::assert_debug_snapshot!(engine.matched_paths("b").collect::<Vec<_>>(), @"[]");
        insta::assert_debug_snapshot!(engine.matched_paths("a").collect::<Vec<_>>(), @r###"
        [
            "a",
        ]
        "###);
        insta::assert_debug_snapshot!(engine.matched_paths("/a").collect::<Vec<_>>(), @r###"
        [
            "a",
        ]
        "###);
        insta::assert_debug_snapshot!(engine.matched_paths("*/a").collect::<Vec<_>>(), @r###"
        [
            "c/a",
        ]
        "###);
        insta::assert_debug_snapshot!(engine.matched_paths("*a").collect::<Vec<_>>(), @r###"
        [
            "a",
            "c/a",
        ]
        "###);
        insta::assert_debug_snapshot!(engine.matched_paths("*/b").collect::<Vec<_>>(), @r###"
        [
            "c/b",
            "d/b",
        ]
        "###);
    }

    #[test]
    fn test_changed_paths() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            staged: ["a" => "b"]
            working: ["c/a" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        let mut changed_paths = engine.changed_paths().collect::<Vec<_>>();
        changed_paths.sort();
        insta::assert_debug_snapshot!(changed_paths, @r###"
        [
            "a",
            "c/a",
        ]
        "###);
    }

    #[test]
    fn test_changed_paths_staged_only() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            staged: ["a" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        insta::assert_debug_snapshot!(engine.changed_paths().collect::<Vec<_>>(), @r###"
        [
            "a",
        ]
        "###);
    }

    #[test]
    fn test_changed_paths_working_only() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            working: ["a" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        insta::assert_debug_snapshot!(engine.changed_paths().collect::<Vec<_>>(), @r###"
        [
            "a",
        ]
        "###);
    }

    #[test]
    fn test_without_if_changed_ignore_trailer() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            "second commit": ["a" => "b"]
        };

        let engine = git(&repo, Some("HEAD~1"), Some("HEAD"));
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        assert!(!engine.is_ignored(Path::new("a")));
        assert!(!engine.is_ignored(Path::new("c/a")));
    }

    #[test]
    fn test_with_if_changed_ignore_trailer() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            "second commit\n\nignore-if-changed: c/a": ["a" => "b"]
        };

        let engine = git(&repo, Some("HEAD~1"), Some("HEAD"));
        assert_eq!(engine.root(), tempdir.path().canonicalize().unwrap());

        assert!(!engine.is_ignored(Path::new("a")));
        assert!(engine.is_ignored(Path::new("c/a")));
    }
}

use std::{
    borrow::{BorrowMut, Cow},
    path::{Path, PathBuf, MAIN_SEPARATOR_STR},
    str::FromStr as _,
};

use bstr::ByteSlice;
use genawaiter::{rc::gen, yield_};

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
    ignore_pathspec: Option<git2::Pathspec>,
    repository: &'a git2::Repository,
    from_tree: Option<git2::Tree<'a>>,
    to_tree: Option<git2::Tree<'a>>,
}

impl<'a> GitEngine<'a> {
    fn new(repository: &'a git2::Repository, from_ref: Option<&str>, to_ref: Option<&str>) -> Self {
        let ignore_pathspec = ignore_pathspec(to_ref, repository);

        let from_tree = match from_ref {
            Some(from_ref) => Some(
                repository
                    .revparse_single(from_ref)
                    .expect("from_ref is not a valid revision")
                    .peel_to_tree()
                    .expect("from_ref does not point to a tree"),
            ),
            None => repository
                .head()
                .map(|head| head.peel_to_tree().unwrap())
                .ok(),
        };

        let to_tree = to_ref.map(|to_ref| {
            repository
                .revparse_single(to_ref)
                .unwrap()
                .peel_to_tree()
                .unwrap()
        });

        Self {
            ignore_pathspec,
            repository,
            from_tree,
            to_tree,
        }
    }

    /// Get the diff of a file, if any.
    fn diff(&self, mut options: impl BorrowMut<git2::DiffOptions>) -> git2::Diff {
        match &self.to_tree {
            Some(to_tree) => self.repository.diff_tree_to_tree(
                self.from_tree.as_ref(),
                Some(to_tree),
                Some(options.borrow_mut()),
            ),
            None => self.repository.diff_tree_to_workdir_with_index(
                self.from_tree.as_ref(),
                Some(options.borrow_mut().include_untracked(true)),
            ),
        }
        .unwrap()
    }

    /// Get the patch of a file, if any.
    fn patch(&self, path: &Path) -> Option<git2::Patch> {
        git2::Patch::from_diff(
            &self.diff(
                git2::DiffOptions::new()
                    .pathspec(path)
                    .disable_pathspec_match(true),
            ),
            0,
        )
        .ok()
        .flatten()
    }
}

impl Engine for GitEngine<'_> {
    fn matches(
        &self,
        patterns: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> impl Iterator<Item = Result<PathBuf, PathBuf>> {
        let mut patterns = patterns
            .into_iter()
            .map(|pattern| {
                let pattern = pattern.as_ref();
                pattern
                    .strip_prefix(MAIN_SEPARATOR_STR)
                    .unwrap_or(pattern)
                    .to_owned()
            })
            .collect::<Vec<_>>();

        // Need to reverse the pathspecs to match in `.gitignore` order.
        patterns.reverse();

        let diff = self.diff(git2::DiffOptions::new());
        gen!({
            if patterns.is_empty() {
                for delta in diff.deltas() {
                    yield_!(Ok(delta.new_file().path().unwrap().to_owned()))
                }
                return;
            }

            let pathspec = git2::Pathspec::new(patterns).unwrap();
            let matches = pathspec
                .match_diff(&diff, git2::PathspecFlags::FIND_FAILURES)
                .expect("bare repos are not supported");
            for delta in matches.diff_entries() {
                yield_!(Ok(delta.new_file().path().unwrap().to_owned()))
            }
            for entry in matches.failed_entries() {
                yield_!(Err(PathBuf::from_str(&entry.to_str_lossy()).unwrap()))
            }
        })
        .into_iter()
    }

    fn resolve(&self, path: impl AsRef<Path>) -> PathBuf {
        self.repository
            .workdir()
            .expect("bare repos are not supported")
            .canonicalize()
            .unwrap()
            .join(path.as_ref())
    }

    fn is_ignored(&self, path: impl AsRef<Path>) -> bool {
        let Some(pathspec) = &self.ignore_pathspec else {
            return false;
        };
        pathspec.matches_path(path.as_ref(), git2::PathspecFlags::DEFAULT)
    }

    fn is_range_modified(&self, path: impl AsRef<Path>, range: (usize, usize)) -> bool {
        let Some(patch) = self.patch(path.as_ref()) else {
            return false;
        };
        // Special case for untracked files. They are always considered modified.
        if patch.delta().status() == git2::Delta::Untracked {
            return true;
        }
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
    let patterns = trailers
        .iter()
        .filter(|(name, _)| name.to_ascii_lowercase() == IF_CHANGED_IGNORE_TRAILER)
        .flat_map(|(_, value)| split_patterns(value))
        .map(|pattern| PathBuf::from_str(&pattern).unwrap())
        .collect::<Vec<_>>();
    if patterns.is_empty() {
        None
    } else {
        Some(git2::Pathspec::new(patterns.iter().rev()).expect("Ignore-if-changed is invalid."))
    }
}

fn split_patterns(value: &[u8]) -> impl Iterator<Item = Cow<str>> {
    value
        .split_once_str(b"--")
        .unwrap_or((value, b""))
        .0
        .split_str(b",")
        .map(|s| s.trim().to_str_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::git_test;

    macro_rules! extract_pathspec_test {
        ($name:ident, $val:expr, @$exp:literal) => {
            #[test]
            fn $name() {
                insta::assert_compact_json_snapshot!(split_patterns($val)
                    .collect::<Vec<_>>(), @$exp);
            }
        };
    }

    extract_pathspec_test!(test_basic_pathspec, b"a", @r###"["a"]"###);
    extract_pathspec_test!(test_multiple_pathspec, b"a/b, b/c", @r###"["a/b", "b/c"]"###);
    extract_pathspec_test!(
        test_multiple_pathspec_with_comment,
        b"a/b, b/c -- Hello world!", @r###"["a/b", "b/c"]"###
    );
    extract_pathspec_test!(test_multiple_pathspec_with_empty_comment, b"a/b, b/c --", @r###"["a/b", "b/c"]"###);

    #[test]
    fn test_git() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "b" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_compact_json_snapshot!(engine.matches(["";0]).collect::<Vec<_>>(), @"[]");
        insta::assert_compact_json_snapshot!(engine.matches(&["a"]).collect::<Vec<_>>(), @r###"[{"Err": "a"}]"###);
        assert!(!engine.is_ignored(Path::new("a")));
    }

    #[test]
    fn test_git_without_head() {
        let (tempdir, repo) = git_test! {
            staged: ["a" => "a", "b" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_compact_json_snapshot!(engine.matches(["";0]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}, {"Ok": "b"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["a"]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}]"###);
        assert!(!engine.is_ignored(Path::new("a")));
    }

    #[test]
    fn test_matches() {
        let (tempdir, repo) = git_test! {
            staged: ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_compact_json_snapshot!(engine.matches(&["b"]).collect::<Vec<_>>(), @r###"[{"Err": "b"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["a"]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["/a"]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["*/a"]).collect::<Vec<_>>(), @r###"[{"Ok": "c/a"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["*a"]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}, {"Ok": "c/a"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["*/b"]).collect::<Vec<_>>(), @r###"[{"Ok": "c/b"}, {"Ok": "d/b"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["c/*"]).collect::<Vec<_>>(), @r###"[{"Ok": "c/a"}, {"Ok": "c/b"}]"###);
        insta::assert_compact_json_snapshot!(engine.matches(&["c/*", "!c/b", "!c/c"]).collect::<Vec<_>>(), @r###"[{"Ok": "c/a"}, {"Err": "c/c"}]"###);
    }

    #[test]
    fn test_changes() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            staged: ["a" => "b"]
            working: ["c/a" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_compact_json_snapshot!(engine.matches([""; 0]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}, {"Ok": "c/a"}]"###);
    }

    #[test]
    fn test_changes_staged_only() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            staged: ["a" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_compact_json_snapshot!(engine.matches(["";0]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}]"###);
    }

    #[test]
    fn test_changes_working_only() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            working: ["a" => "b"]
        };

        let engine = git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_compact_json_snapshot!(engine.matches(["";0]).collect::<Vec<_>>(), @r###"[{"Ok": "a"}]"###);
    }

    #[test]
    fn test_without_if_changed_ignore_trailer() {
        let (tempdir, repo) = git_test! {
            "initial commit": ["a" => "a", "c/a" => "a", "c/b" => "b", "d/b" => "b"]
            "second commit": ["a" => "b"]
        };

        let engine = git(&repo, Some("HEAD~1"), Some("HEAD"));
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

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
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        assert!(!engine.is_ignored(Path::new("a")));
        assert!(engine.is_ignored(Path::new("c/a")));
    }
}

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use std::process::ExitCode;

use clap::Parser as ClapParser;
use genawaiter::{rc::gen, yield_};
use if_changed::{Engine as _, GitEngine};

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The revision to compare against. By default, HEAD is used.
    #[arg(long, env = "PRE_COMMIT_FROM_REF")]
    pub from_ref: Option<String>,

    /// The revision to compare with. By default, the current working tree is used.
    #[arg(long, env = "PRE_COMMIT_TO_REF")]
    pub to_ref: Option<String>,

    /// Git patterns defining the set of files to check. By default, this will
    /// be all changed files between revisions.
    ///
    /// This list follows the same rules as
    /// [`.gitignore`](https://git-scm.com/docs/gitignore) except relative
    /// paths/patterns are always matched against the repository root, even if the
    /// paths/patterns don't contain `/`. In particular, a leading `!` before a
    /// pattern will reinclude the pattern if it was excluded by a previous
    /// pattern.
    #[arg()]
    pub patterns: Vec<String>,
}

fn run(cli: Cli, repository: git2::Repository) -> impl Iterator<Item = String> {
    gen!({
        let engine = GitEngine::new(&repository, cli.from_ref.as_deref(), cli.to_ref.as_deref());
        for result in engine.matches(cli.patterns) {
            let Ok(path) = result else {
                continue;
            };
            if engine.is_ignored(&path) {
                continue;
            }
            if let Err(errors) = engine.check(path) {
                for error in errors {
                    yield_!(error);
                }
            }
        }
    })
    .into_iter()
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn main() -> ExitCode {
    let mut has_error = false;
    let repository = match git2::Repository::open_from_env() {
        Ok(repository) => repository,
        Err(error) => {
            eprintln!("Could not open the repository: {error}");
            return ExitCode::FAILURE;
        }
    };
    for error in run(Cli::parse(), repository) {
        has_error = true;
        eprintln!("{error}");
    }
    if has_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

#[cfg(test)]
mod tests {
    use if_changed::testing::git_test;
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_run() {
        let (tempdir, _repo) = git_test! {
            "initial commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(a.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @"[]");
    }

    #[test]
    fn test_run_fail() {
        let (tempdir, _repo) = git_test! {
            "initial commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @r###"["Expected \"b.ts\" to be modified because of \"then-change\" in \"a.ts\" at line 4."]"###);
    }

    #[test]
    fn test_run_commit_footer() {
        let (tempdir, _repo) = git_test! {
            "initial commit\n\nignore-if-changed: a.ts": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @"[]");
    }

    #[test]
    fn test_run_commit_footer_with_reason() {
        let (tempdir, _repo) = git_test! {
            "initial commit\n\nignore-if-changed: a.ts -- idky": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @"[]");
    }

    #[test]
    fn test_run_no_matching() {
        let (tempdir, _repo) = git_test! {
            "initial commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: Some("HEAD".into()),
            patterns: vec!["c.js".to_string()],
        }, repository).collect::<Vec<_>>(), @"[]");
    }

    #[test]
    fn test_run_working_dir() {
        let (tempdir, _repo) = git_test! {
            working: [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(a.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: None,
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @"[]");
    }

    #[test]
    fn test_run_working_dir_fail() {
        let (tempdir, _repo) = git_test! {
            working: [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: None,
            to_ref: None,
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @r###"["Expected \"b.ts\" to be modified because of \"then-change\" in \"a.ts\" at line 4."]"###);
    }

    #[test]
    fn test_run_two_commits() {
        let (tempdir, _repo) = git_test! {
            "initial commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(a.ts)
                    }
                "}
            ]
            "second commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        B,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        B,
                        // then-change(a.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: Some("HEAD^".into()),
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @"[]");
    }

    #[test]
    fn test_run_two_commits_fail() {
        let (tempdir, _repo) = git_test! {
            "initial commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(a.ts)
                    }
                "}
            ]
            "second commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        B,
                        // then-change(b.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: Some("HEAD^".into()),
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @r###"["Expected \"b.ts\" to be modified because of \"then-change\" in \"a.ts\" at line 5."]"###);
    }

    #[test]
    fn test_run_two_commits_fail_no_change() {
        let (tempdir, _repo) = git_test! {
            "initial commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(a.ts)
                    }
                "}
            ]
            "second commit": [
                "a.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        B,
                        // then-change(b.ts)
                    }
                "},
                "b.ts" => indoc! {"
                    const enum G {
                        // if-changed
                        A,
                        // then-change(a.ts)
                    }
                "}
            ]
        };

        let repository = git2::Repository::open(tempdir.path()).unwrap();
        insta::assert_compact_json_snapshot!(run(Cli {
            from_ref: Some("HEAD^".into()),
            to_ref: Some("HEAD".into()),
            patterns: vec![],
        }, repository).collect::<Vec<_>>(), @r###"["Expected \"b.ts\" to be modified because of \"then-change\" in \"a.ts\" at line 5."]"###);
    }
}

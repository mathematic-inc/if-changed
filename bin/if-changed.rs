use std::{path::PathBuf, process::ExitCode};

use clap::Parser as ClapParser;
use if_changed::{git, Engine as _};

#[derive(Clone, Debug)]
pub struct IfChangedBlock {
    pub name: Option<String>,
    pub range: (usize, usize),
    pub paths: Vec<(usize, PathBuf, Option<String>)>,
}

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
    /// paths/patterns are always matched against the repository root, if the
    /// paths/patterns don't contain `/`. In particular, a leading `!` before a
    /// pattern will reinclude the pattern if it was excluded by a previous
    /// pattern.
    #[arg()]
    pub patterns: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let repository = git2::Repository::open_from_env().unwrap();
    let engine = git(&repository, cli.from_ref.as_deref(), cli.to_ref.as_deref());
    let mut has_error = false;
    for result in engine.matches(cli.patterns) {
        let Ok(path) = result else {
            continue;
        };
        if engine.is_ignored(&path) {
            continue;
        }
        if let Err(errors) = engine.check(path) {
            for error in errors {
                eprintln!("{}", error);
            }
            has_error = true;
        }
    }
    if has_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

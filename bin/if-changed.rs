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
    #[arg(long)]
    pub from_ref: Option<String>,

    /// The revision to compare with. By default, the current working tree is used.
    #[arg(long)]
    pub to_ref: Option<String>,

    /// Git pathspec defining the set of files to check. By default, this will be all changed files between revisions.
    #[arg()]
    pub pathspec: Vec<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let repository = git2::Repository::open_from_env().unwrap();
    let engine = git(&repository, cli.from_ref.as_deref(), cli.to_ref.as_deref());
    let pathspec = git2::Pathspec::new(&cli.pathspec).unwrap();
    let mut has_error = false;
    for path in engine.changed_paths() {
        if !pathspec.matches_path(&path, Default::default()) || engine.is_ignored(&path) {
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

#![warn(rust_2018_idioms)]

use std::process::Command;

// https://github.com/mathematic-inc/if-changed/pull/450
// Installation checks must work before the user opens a Git repository.
#[test]
fn cli_help_and_version_work_outside_a_repository() {
    let directory = tempfile::tempdir().unwrap();

    for (argument, expected) in [("--help", "Usage:"), ("--version", "if-changed")] {
        let output = Command::new(env!("CARGO_BIN_EXE_if-changed"))
            .arg(argument)
            .current_dir(directory.path())
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{argument} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    }
}

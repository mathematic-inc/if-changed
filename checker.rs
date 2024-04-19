use std::path::Path;

use super::{engine::Engine, parser::Parser};
use crate::NamedPattern;

pub struct Checker<'a, E: ?Sized> {
    engine: &'a E,
    path: &'a Path,
    parser: Parser,
}

impl<'a, E: Engine + ?Sized> Checker<'a, E> {
    pub(super) fn new(engine: &'a E, path: &'a Path) -> Result<Self, Vec<String>> {
        Ok(Self {
            engine,
            path,
            parser: Parser::new(engine.resolve(path)).map_err(|error| vec![error.to_string()])?,
        })
    }

    pub(super) fn check(self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for block in self.parser {
            let block = match block {
                Ok(block) => block,
                Err(error) => return Err(error),
            };

            if !self.engine.is_range_modified(self.path, block.range) {
                continue;
            }

            for NamedPattern {
                name,
                mut pattern,
                line,
            } in block.patterns
            {
                // Empty pattern means current file.
                if pattern.is_empty() {
                    pattern = self
                        .path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                }
                pattern = self
                    .path
                    .parent()
                    .unwrap()
                    .join(&pattern)
                    .to_string_lossy()
                    .into_owned();

                // Every pattern must match at least one file.
                let mut path_found = false;
                for path in self.engine.paths(&pattern) {
                    path_found = true;

                    // If the file isn't modified, then we immediately fail.
                    if self
                        .engine
                        .changed_paths()
                        .all(|changed_path| changed_path != path)
                    {
                        errors.push(format!(
                            "Expected {path:?} to be modified because of \"then-change\" in {:?} at line {line}.",
                            self.path,
                        ));
                        continue;
                    }

                    // If this is not a named block, then we're done.
                    let Some(ref name) = name else {
                        continue;
                    };

                    // Try to open the file in search of the named block.
                    let mut parser = match Parser::new(self.engine.resolve(&path)) {
                        Ok(parser) => parser,
                        Err(error) => {
                            errors.push(format!(
                            "Could not open {path:?} for \"then-change\" in {:?} at line {line}: {error:?}",
                            self.path.display(),
                        ));
                            continue;
                        }
                    };

                    // Search for the named block, accumulating errors along the way.
                    let Some(block) = parser.find_map(|block| match block {
                        Ok(block) if block.name.as_deref() == Some(name) => Some(Ok(block)),
                        Err(error) => Some(Err(error)),
                        _ => None,
                    }) else {
                        errors.push(format!(
                            "Could not find \"if-changed\" with name \"{name}\" in {path:?} for \"then-change\" in {:?} at line {line}.",
                            self.path.display(),
                        ));
                        continue;
                    };

                    match block {
                        Ok(block) => {
                            if !self.engine.is_range_modified(&path, block.range) {
                                errors.push(format!(
                                    "Expected {path:?} to be modified because of \"then-change\" in {:?} at line {line}.",
                                    self.path,
                                ));
                            }
                        }
                        Err(error) => errors.extend(error),
                    }
                }

                if !path_found {
                    errors.push(format!(
                        "Could not find any file matching {pattern:?} for \"then-change\" in {:?} at line {line}.",
                        self.path.display(),
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use indoc::indoc;

    use super::Checker;
    use crate::{testing::git_test, Engine as _};

    #[test]
    fn test_check() {
        let (tempdir, repo) = git_test! {
            "initial commit": [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js)
                "},
                "src/b.js" => ""
            ]
            working: [
                "src/a.js" => indoc!{"
                    // if-changed
                    foobar
                    // then-change(b.js)
                "},
                "src/b.js" => "bar"
            ]
        };

        let engine = crate::git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        let mut changed_paths = engine.changed_paths().collect::<Vec<_>>();
        changed_paths.sort();
        insta::assert_debug_snapshot!(changed_paths, @r###"
        [
            "src/a.js",
            "src/b.js",
        ]
        "###);

        insta::assert_debug_snapshot!(Checker::new(&engine, &Path::new("src/a.js")).unwrap().check(), @r###"
        Ok(
            (),
        )
        "###);
    }

    #[test]
    fn test_check_fail() {
        let (tempdir, repo) = git_test! {
            "initial commit": [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js)
                "},
                "src/b.js" => ""
            ]
            working: [
                "src/a.js" => indoc!{"
                    // if-changed
                    foobar
                    // then-change(b.js)
                "}
            ]
        };

        let engine = crate::git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_debug_snapshot!(engine.changed_paths().collect::<Vec<_>>(), @r###"
        [
            "src/a.js",
        ]
        "###);
        insta::assert_debug_snapshot!(Checker::new(&engine, &Path::new("src/a.js")).unwrap().check(), @r###"
        Err(
            [
                "Expected \"src/b.js\" to be modified because of \"then-change\" in \"src/a.js\" at line 3.",
            ],
        )
        "###);
    }

    #[test]
    fn test_check_unrelated() {
        let (tempdir, repo) = git_test! {
            "initial commit": [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js)
                "},
                "src/b.js" => ""
            ]
            working: [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js)
                    this
                "}
            ]
        };

        let engine = crate::git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        insta::assert_debug_snapshot!(engine.changed_paths().collect::<Vec<_>>(), @r###"
        [
            "src/a.js",
        ]
        "###);
        insta::assert_debug_snapshot!(Checker::new(&engine, &Path::new("src/a.js")).unwrap().check(), @r###"
        Ok(
            (),
        )
        "###);
    }

    #[test]
    fn test_check_named() {
        let (tempdir, repo) = git_test! {
            "initial commit": [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js:bar)
                "},
                "src/b.js" => indoc!{"
                    // if-changed(bar)
                    foo
                    // then-change(a.js)
                "}
            ]
            working: [
                "src/a.js" => indoc!{"
                    // if-changed
                    foobar
                    // then-change(b.js:bar)
                "},
                "src/b.js" => indoc!{"
                    // if-changed(bar)
                    foobar
                    // then-change(a.js)
                "}
            ]
        };

        let engine = crate::git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        let mut changed_paths = engine.changed_paths().collect::<Vec<_>>();
        changed_paths.sort();
        insta::assert_debug_snapshot!(changed_paths, @r###"
        [
            "src/a.js",
            "src/b.js",
        ]
        "###);

        insta::assert_debug_snapshot!(Checker::new(&engine, &Path::new("src/a.js")).unwrap().check(), @r###"
        Ok(
            (),
        )
        "###);
    }

    #[test]
    fn test_check_named_fail() {
        let (tempdir, repo) = git_test! {
            "initial commit": [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js:bar)
                "},
                "src/b.js" => indoc!{"
                    // if-changed(bar)
                    foo
                    // then-change(a.js)
                "}
            ]
            working: [
                "src/a.js" => indoc!{"
                    // if-changed
                    foobar
                    // then-change(b.js:bar)
                "},
                "src/b.js" => indoc!{"
                    // if-changed(bar)
                    foo
                    // then-change(a.js)
                    bar
                "}
            ]
        };

        let engine = crate::git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        let mut changed_paths = engine.changed_paths().collect::<Vec<_>>();
        changed_paths.sort();
        insta::assert_debug_snapshot!(changed_paths, @r###"
        [
            "src/a.js",
            "src/b.js",
        ]
        "###);

        insta::assert_debug_snapshot!(Checker::new(&engine, &Path::new("src/a.js")).unwrap().check(), @r###"
        Err(
            [
                "Expected \"src/b.js\" to be modified because of \"then-change\" in \"src/a.js\" at line 3.",
            ],
        )
        "###);
    }

    #[test]
    fn test_check_named_missing() {
        let (tempdir, repo) = git_test! {
            "initial commit": [
                "src/a.js" => indoc!{"
                    // if-changed
                    foo
                    // then-change(b.js:bar)
                "},
                "src/b.js" => ""
            ]
            working: [
                "src/a.js" => indoc!{"
                    // if-changed
                    foobar
                    // then-change(b.js:bar)
                "},
                "src/b.js" => "foo"
            ]
        };

        let engine = crate::git(&repo, None, None);
        assert_eq!(engine.resolve(""), tempdir.path().canonicalize().unwrap());

        let mut changed_paths = engine.changed_paths().collect::<Vec<_>>();
        changed_paths.sort();
        insta::assert_debug_snapshot!(changed_paths, @r###"
        [
            "src/a.js",
            "src/b.js",
        ]
        "###);

        insta::assert_debug_snapshot!(Checker::new(&engine, &Path::new("src/a.js")).unwrap().check(), @r###"
        Err(
            [
                "Could not find \"if-changed\" with name \"bar\" in \"src/b.js\" for \"then-change\" in \"src/a.js\" at line 3.",
            ],
        )
        "###);
    }
}

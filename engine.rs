mod git;

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub use git::git;

use super::parser::Parser;

pub trait Engine {
    /// Iterate over changed files that match the given patterns and patterns that don't match any file.
    ///
    /// If patterns is empty, all changed files are returned.
    fn matches(
        &self,
        patterns: impl IntoIterator<Item = impl AsRef<Path>>,
    ) -> impl Iterator<Item = Result<PathBuf, PathBuf>>;

    /// Resolve a path to an absolute path.
    fn resolve(&self, path: impl AsRef<Path>) -> PathBuf;

    /// Check if a file has been ignored.
    fn is_ignored(&self, path: impl AsRef<Path>) -> bool;

    /// Check if a range of lines in a file has been modified.
    fn is_range_modified(&self, path: impl AsRef<Path>, range: (usize, usize)) -> bool;

    /// Check a file for dependent changes.
    fn check(&self, path: impl AsRef<Path>) -> Result<(), Vec<String>> {
        let path = path.as_ref();
        let parser = Parser::new(self.resolve(path)).map_err(|error| vec![error.to_string()])?;

        let mut errors = Vec::new();
        for block in parser {
            let block = match block {
                Ok(block) => block,
                Err(error) => return Err(error),
            };

            if !self.is_range_modified(path, block.range) {
                continue;
            }

            // Resolve patterns based on the current file.
            let resolved_patterns = block
                .patterns
                .into_iter()
                .map(|mut pattern| {
                    // Empty pattern means current file.
                    pattern.value = if pattern.value == Path::new("") {
                        path.to_owned()
                    } else {
                        path.parent().unwrap().join(&pattern.value)
                    };
                    pattern
                })
                .collect::<Vec<_>>();

            let mut named_patterns = BTreeMap::new();
            let mut unnamed_patterns = BTreeMap::new();
            for pattern in &resolved_patterns {
                let Some(name) = &pattern.name else {
                    unnamed_patterns.insert(&*pattern.value, pattern.line);
                    continue;
                };
                named_patterns.insert(&*pattern.value, (&**name, pattern.line));
            }

            for pattern in self.matches(unnamed_patterns.keys()).flat_map(Result::err) {
                let line = unnamed_patterns.get(&*pattern).unwrap();
                errors.push(format!(
                    "Expected {pattern:?} to be modified because of \"then-change\" in {path:?} at line {line}."
                ));
            }

            for (pattern, (name, line)) in named_patterns {
                for result in self.matches([pattern]) {
                    let dependent = match result {
                        Ok(path) => path,
                        Err(pattern) => {
                            errors.push(format!(
                                "Expected {pattern:?} to be modified because of \"then-change\" in {path:?} at line {line}."
                            ));
                            continue;
                        }
                    };

                    // Try to open the file in search of the named block.
                    let mut parser = match Parser::new(self.resolve(&dependent)) {
                        Ok(parser) => parser,
                        Err(error) => {
                            errors.push(format!(
                                "Could not open {dependent:?} for \"then-change\" in {path:?} at line {line}: {error:?}"
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
                            "Could not find \"if-changed\" with name \"{name}\" in {dependent:?} for \"then-change\" in {path:?} at line {line}."
                        ));
                        continue;
                    };

                    match block {
                        Ok(block) => {
                            if !self.is_range_modified(&dependent, block.range) {
                                errors.push(format!(
                                    "Expected {dependent:?} to be modified because of \"then-change\" in {path:?} at line {line}."
                                ));
                            }
                        }
                        Err(error) => errors.extend(error),
                    }
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

        insta::assert_compact_json_snapshot!(engine.matches([""; 0]).collect::<Vec<_>>(), @r###"[{"Ok": "src/a.js"}, {"Ok": "src/b.js"}]"###);
        insta::assert_compact_json_snapshot!(engine.check(&Path::new("src/a.js")), @r###"{"Ok": null}"###);
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

        insta::assert_compact_json_snapshot!(engine.matches(["";0]).collect::<Vec<_>>(), @r###"[{"Ok": "src/a.js"}]"###);
        insta::assert_compact_json_snapshot!(engine.check(&Path::new("src/a.js")), @r###"{"Err": ["Expected \"src/b.js\" to be modified because of \"then-change\" in \"src/a.js\" at line 3."]}"###);
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

        insta::assert_compact_json_snapshot!(engine.matches(["";0]).collect::<Vec<_>>(), @r###"[{"Ok": "src/a.js"}]"###);
        insta::assert_compact_json_snapshot!(engine.check(&Path::new("src/a.js")), @r###"{"Ok": null}"###);
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

        insta::assert_compact_json_snapshot!(engine.matches([""; 0]).collect::<Vec<_>>(), @r###"[{"Ok": "src/a.js"}, {"Ok": "src/b.js"}]"###);
        insta::assert_compact_json_snapshot!(engine.check( &Path::new("src/a.js")), @r###"{"Ok": null}"###);
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

        insta::assert_compact_json_snapshot!(engine.matches([""; 0]).collect::<Vec<_>>(), @r###"[{"Ok": "src/a.js"}, {"Ok": "src/b.js"}]"###);
        insta::assert_compact_json_snapshot!(engine.check(&Path::new("src/a.js")), @r###"{"Err": ["Expected \"src/b.js\" to be modified because of \"then-change\" in \"src/a.js\" at line 3."]}"###);
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

        insta::assert_compact_json_snapshot!(engine.matches([""; 0]).collect::<Vec<_>>(), @r###"[{"Ok": "src/a.js"}, {"Ok": "src/b.js"}]"###);
        insta::assert_compact_json_snapshot!(engine.check(&Path::new("src/a.js")), @r###"
        {
          "Err": [
            "Could not find \"if-changed\" with name \"bar\" in \"src/b.js\" for \"then-change\" in \"src/a.js\" at line 3."
          ]
        }
        "###);
    }
}

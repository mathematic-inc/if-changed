use std::{
    fs,
    io::{self, BufRead},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use super::IfChangedBlock;
use crate::NamedPattern;

const COMMENT_START_TOKENS: [char; 12] =
    ['/', '#', '-', '\'', ';', 'R', 'E', 'M', '!', '*', '<', '!'];

struct StringRef {
    #[allow(dead_code)]
    owner: String,
    reference: *const str,
}

impl StringRef {
    fn new(owner: String) -> StringRef {
        StringRef {
            reference: owner.as_str(),
            owner,
        }
    }

    fn modify_with(&mut self, f: impl FnOnce(&str) -> &str) -> &mut Self {
        self.reference = f(&*self);
        self
    }

    fn try_modify_with(&mut self, f: impl FnOnce(&str) -> Option<&str>) -> Option<&mut Self> {
        if let Some(reference) = f(&*self) {
            self.reference = reference;
            Some(self)
        } else {
            None
        }
    }
}

impl Deref for StringRef {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        return unsafe { self.reference.as_ref().unwrap_unchecked() };
    }
}

struct NumberedLine {
    number: usize,
    value: StringRef,
}

impl NumberedLine {
    fn new(number: usize, line: String) -> NumberedLine {
        NumberedLine {
            number,
            value: StringRef::new(line),
        }
    }
}

impl Deref for NumberedLine {
    type Target = StringRef;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for NumberedLine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub(super) struct Parser {
    path: PathBuf,

    lines: io::Lines<io::BufReader<std::fs::File>>,
    line: NumberedLine,

    blocks: Vec<IfChangedBlock>,
}

impl Parser {
    pub(super) fn new(path: impl AsRef<Path>) -> Result<Parser, io::Error> {
        Ok(Parser {
            path: path.as_ref().to_path_buf(),
            lines: io::BufReader::new(match fs::File::open(&path) {
                Ok(file) => file,
                Err(error) => return Err(error),
            })
            .lines(),
            line: NumberedLine::new(0, String::default()),
            blocks: Vec::new(),
        })
    }

    fn next_line(&mut self) -> Result<bool, Vec<String>> {
        match self.lines.next() {
            Some(result) => match result {
                Ok(line) => {
                    self.line = NumberedLine::new(self.line.number + 1, line);
                    Ok(true)
                }
                Err(value) => Err(vec![format!(
                    "Failed to read {}: {}",
                    self.path.display(),
                    value
                )]),
            },
            None => Ok(false),
        }
    }

    fn skip_comments(&mut self) {
        self.skip_whitespaces();
        self.line
            .modify_with(|line| line.trim_start_matches(COMMENT_START_TOKENS.as_ref()));
    }

    fn skip_whitespaces(&mut self) {
        self.line.modify_with(str::trim_start);
    }

    fn skip_whitespaces_and_eat(&mut self, value: &str) -> bool {
        self.skip_whitespaces();
        self.line
            .try_modify_with(|line| line.strip_prefix(value))
            .is_some()
    }

    fn find_and_eat(&mut self, value: &str) -> bool {
        self.line
            .try_modify_with(|line| match line.find(value) {
                Some(index) => Some(&line[index + value.len()..]),
                None => None,
            })
            .is_some()
    }

    fn parse_if_changed(&mut self) -> Result<Option<Option<String>>, Vec<String>> {
        self.skip_comments();
        Ok(if self.skip_whitespaces_and_eat("if-changed") {
            Some(self.parse_if_changed_name()?)
        } else {
            None
        })
    }

    fn parse_if_changed_name(&mut self) -> Result<Option<String>, Vec<String>> {
        if !self.skip_whitespaces_and_eat("(") {
            return Ok(None);
        }
        let end = match self.line.find(')') {
            Some(end) => end,
            None => {
                return Err(vec![format!(
                    "Could not find ')' for \"if-changed\" at line {} for \"{}\".",
                    self.line.number,
                    self.path.display()
                )])
            }
        };
        let id = self.line[..end].trim().to_string();
        self.line.modify_with(|line| &line[end + 1..]);
        Ok(Some(id))
    }

    fn parse_then_change(&mut self) -> Result<Option<(Vec<NamedPattern>, usize)>, Vec<String>> {
        Ok(if self.find_and_eat("then-change") {
            // Note we grab the line number before parsing the paths. This is
            // important as changes in file references shouldn't require
            // changing existing file references. This only matters if the
            // file references are multiline.
            let line = self.line.number;
            let specs = self.parse_then_change_paths()?;
            Some((specs, line))
        } else {
            None
        })
    }

    fn parse_then_change_paths(&mut self) -> Result<Vec<NamedPattern>, Vec<String>> {
        let then_change_line = self.line.number;
        if !self.skip_whitespaces_and_eat("(") {
            return Err(vec![format!(
                "Could not find '(' for \"then-change\" at line {then_change_line} for \"{}\".",
                self.path.display()
            )]);
        }

        let mut related_paths = Vec::new();

        let mut pathspec_buffer = String::new();
        let mut pathspec_line = 0;
        let mut right_paren_found = false;
        loop {
            // Skip over whitespaces and empty line comments.
            while {
                self.skip_whitespaces();
                self.line.is_empty()
            } {
                if !self.next_line()? {
                    return Err(vec![format!(
                        "Could not find ')' for \"then-change\" at line {then_change_line} for \"{}\".",
                        self.path.display()
                    )]);
                }
                self.skip_comments();
            }

            // At this point, the line is guaranteed to not be empty and within a comment.
            if pathspec_line == 0 {
                pathspec_line = self.line.number;
            }
            match self.line.find('\\') {
                Some(index) => {
                    pathspec_buffer.push_str(self.line[..index].trim());
                    self.line.modify_with(|line| &line[index + 1..]);
                    continue;
                }
                None => {
                    // If a continuation is not found, then detect either an
                    // comma, ending parenthesis, or EOL.
                    let (index, len) = match self.line.find(',') {
                        Some(index) => (index, 1),
                        None => match self.line.find(')') {
                            Some(index) => {
                                right_paren_found = true;
                                (index, 1)
                            }
                            None => (self.line.len(), 0),
                        },
                    };
                    pathspec_buffer.push_str(self.line[..index].trim());
                    self.line.modify_with(|line| &line[index + len..]);
                }
            }

            let (pathspec, name) = match pathspec_buffer.split_once(':') {
                // If the related path has the form "foo:bar", then
                // `pathspec` will be "foo" and `name` will be "bar".
                Some((pathspec, name)) => {
                    (pathspec.trim().to_owned(), Some(name.trim().to_owned()))
                }
                // Otherwise, `name` is none and the related path is
                // `pathspec_buffer` itself.
                None => {
                    if pathspec_buffer.is_empty() {
                        if right_paren_found {
                            break;
                        }
                        return Err(vec![format!(
                            "Unexpected empty path at line {pathspec_line} for \"then-change\" at line {then_change_line} for \"{}\".",
                            self.path.display()
                        )]);
                    }
                    (pathspec_buffer.clone(), None)
                }
            };

            related_paths.push(NamedPattern {
                name,
                pattern: pathspec,
                line: pathspec_line,
            });
            if right_paren_found {
                break;
            }

            pathspec_line = 0;
            pathspec_buffer.clear();
        }
        Ok(related_paths)
    }
}

impl Iterator for Parser {
    type Item = Result<IfChangedBlock, Vec<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        while match self.next_line() {
            Ok(value) => value,
            Err(error) => return Some(Err(error)),
        } {
            if let Some(name) = match self.parse_if_changed() {
                Ok(name) => name,
                Err(error) => return Some(Err(error)),
            } {
                self.blocks.push(IfChangedBlock {
                    name,
                    range: (self.line.number, 0),
                    patterns: Vec::new(),
                });
            }

            if let Some((paths, end)) = match self.parse_then_change() {
                Ok(info) => info,
                Err(error) => return Some(Err(error)),
            } {
                let mut block = match self.blocks.pop() {
                    Some(block) => block,
                    None => {
                        return Some(Err(vec![format!(
                            "Missing \"if-changed\" for \"then-change\" at line {}.",
                            end
                        )]))
                    }
                };

                block.range.1 = end;
                block.patterns = paths;

                return Some(Ok(block));
            }
        }
        if self.blocks.is_empty() {
            return None;
        }

        Some(Err(self
            .blocks
            .iter()
            .filter(|block| block.range.1 == 0)
            .map(|block| {
                format!(
                    "Could not find \"then-changed\" for \"if-changed\" at line {} for \"{}\".",
                    block.range.0,
                    self.path.display()
                )
            })
            .collect()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::Parser;

    macro_rules! parser_test {
        ($name:ident, $value:expr) => {
            #[test]
            fn $name() {
                let mut file = NamedTempFile::new().unwrap();
                writeln!(file, $value).unwrap();
                insta::assert_debug_snapshot!(Parser::new(file.path())
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>());
            }
        };
    }

    parser_test!(it_parses_empty_files, "");

    parser_test!(
        it_parses,
        "
            // if-changed
            const FOO: u32 = 0;
            // then-change(foo.rs)

            // if-changed(some-name)
            const FOO: u32 = 0;
            // then-change(foo.rs)
        "
    );

    parser_test!(
        it_parses_empty_path_with_name,
        "
            // if-changed(a)
            const FOO: u32 = 0;
            // then-change(:b)

            // if-changed(b)
            const FOO: u32 = 0;
            // then-change(:a)
        "
    );

    parser_test!(
        it_parses_inline_blocks,
        "// if-changed this is a test then-change(foo.rs)"
    );

    parser_test!(
        it_parses_multiple_paths_inline,
        "
            // if-changed
            const FOO: u32 = 0;
            // then-change(foo.rs, bar.rs)

            // if-changed
            const FOO: u32 = 0;
            // then-change(foo.rs, bar.rs, baz.rs)
        "
    );

    parser_test!(
        it_parses_multiple_paths_multiline,
        "
            // if-changed
            const FOO: u32 = 0;
            // then-change(
            //   foo.rs,
            //   bar.rs,
            // )

            // if-changed
            const FOO: u32 = 0;
            // then-change(foo.rs,
            //   bar.rs,
            // )

            // if-changed
            const FOO: u32 = 0;
            // then-change(foo.rs,
            //   bar.rs)

            // if-changed
            const FOO: u32 = 0;
            // then-change(foo.rs,
            //   bar.rs,
            //)

            // if-changed
            const FOO: u32 = 0;
            // then-change(
            //   foo.rs
            //   bar.rs
            // )
        "
    );

    parser_test!(
        it_parses_multiline_comments,
        "
            <!-- if-changed -->
            <div></div>
            <!--
                then-change(
                    foo.rs,
                    bar.rs,
                )
            -->
        "
    );
}

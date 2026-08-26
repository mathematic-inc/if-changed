# `if-changed`

[![crates.io](https://img.shields.io/crates/v/if-changed?style=flat-square)](https://crates.io/crates/if-changed)
[![license](https://img.shields.io/crates/l/if-changed?style=flat-square)](https://github.com/mathematic-inc/if-changed)
[![ci](https://img.shields.io/github/actions/workflow/status/mathematic-inc/if-changed/ci.yml?label=ci&style=flat-square)](https://github.com/mathematic-inc/if-changed/actions/workflows/ci.yml)
[![docs](https://img.shields.io/docsrs/if-changed?style=flat-square)](https://docs.rs/if-changed/latest/if_changed/index.html)

`if-changed` is a command-line utility that checks for `"if-changed"` and `"then-change"` comments in a repository diff and errors if dependent files need changes.

## Installation

```bash
cargo install if-changed
```

## Usage

```bash
Usage: if-changed [OPTIONS] [PATTERNS]...

Arguments:
  [PATTERNS]...
          Git patterns defining the set of files to check. By default, this will be all changed files between revisions.

          This list follows the same rules as [`.gitignore`](https://git-scm.com/docs/gitignore) except relative paths/patterns are always matched against the repository root, even if the paths/patterns don't contain `/`. In particular, a leading `!` before a pattern will reinclude the pattern if it was excluded by a previous pattern.

Options:
      --from-ref <FROM_REF>
          The revision to compare against. By default, HEAD is used

          [env: PRE_COMMIT_FROM_REF=]

      --to-ref <TO_REF>
          The revision to compare with. By default, the current working tree is used

          [env: PRE_COMMIT_TO_REF=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

To use with [`pre-commit`](https://pre-commit.com), add the following to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/mathematic-inc/if-changed
    rev: v0.3.2
    hooks:
      - id: if-changed
```

### Motivating example

Suppose you have the following:

```rs
// lib.rs
enum ErrorCode {
    A,
    B,
    C,
}
```

```ts
// lib.ts
const enum ErrorCode {
  A,
  B,
  C,
}
```

Typically, to synchronize these enums, a common approach is to extract the enum values into a "source-of-truth" file. This often requires significant effort to generate the enums using the build system or a script, and to ensure everything works correctly. If the job is a one-off, the costs heavily outweigh the benefits.

This is where `if-changed` comes in. Instead of the above, suppose we have:

```diff
 // lib.rs
+// if-changed(ecrs)
 enum ErrorCode {
     A,
     B,
     C,
 }
+// then-change(lib.ts:ects)
```

```diff
 // lib.ts
+// if-changed(ects)
 const enum ErrorCode {
   A,
   B,
   C,
 }
+// then-change(lib.rs:ecrs)
```

Once this is committed, the next time `lib.rs` (or `lib.ts`) is changed in the lines surrounded by `"if-changed"` and `"then-change"`, `if-changed` will error if the other file (referenced in the `"then-change"` comment) does not have any changes in the corresponding named block.

> [!TIP]
>
> If you just want to assert that any change in a file is okay, then just reference the file without the name. For example,
>
> ```diff
>  // lib.ts
>  // if-changed(ects)
>  const enum ErrorCode {
>    A,
>    B,
>    C,
>  }
> -// then-change(lib.rs:ecrs)
> +// then-change(lib.rs)
> ```

### File lists

If a block needs to specify several files, you can use commas and/or newlines to separate paths/patterns. For example,

```c
// then-change(foo/bar, baz)

/// OR

// then-change(
//   foo/bar
//   bar
// )
```

These lists follow the same rules as [`.gitignore`](https://git-scm.com/docs/gitignore), with the exception that relative paths/patterns are always matched against the file in which they are written, even if the paths/patterns don't contain `/`. Use a starting `/` to match the pattern against the repository root, e.g. `/*/bar`.

### Long paths

If a path is too long, you can use a shell continuation `\` to split it across multiple lines. For example, for the path `this/is/a/really/long/path/to/some/very/far/away/file`, you can do

```c
// then-change(
//   this/is/a/really/long/path/to/some/very/far/ \
//   away/file
// )
```

### Disabling `if-changed`

To disable `if-changed` for a specific file during a commit, add `Ignore-if-changed: <path>, ... -- [REASON]` to the commit footer. Here, `<path>` names the file. In general, `<path>` can be any pattern allowed by [fnmatch](https://man7.org/linux/man-pages/man3/fnmatch.3.html).

> [!NOTE]
>
> If you want to disable `if-changed` when diffing the working tree, you can execute `if-changed` with the following:
>
> ```bash
> if-changed '*' !<path-or-pattern>
> ```
>
> where `<path-or-pattern>` is the path/pattern you want to ignore.

## Contributing

AI agents handle most maintenance and implementation for this repository. For us, reviewing an unsolicited pull request takes longer than implementing a proposal after we have agreed on it.

[Start a Discussion](https://github.com/mathematic-inc/if-changed/discussions/new) and wait for a maintainer to review the proposal before doing implementation work. If we accept it, a Mathematic maintainer or agent will open the pull request. GitHub restricts pull request creation to Mathematic maintainers and repository collaborators who have write, maintain, or admin access, plus authorized maintenance agents.

When Mathematic implements a proposal, we will link the implementation pull request to the Discussion and credit the original author.

Read [CONTRIBUTING.md](CONTRIBUTING.md) for the full policy.

> This project is free and open-source work by a 501(c)(3) non-profit. If you find it useful, please consider [donating](https://github.com/sponsors/mathematic-inc).

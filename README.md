# `if-changed`

`if-changed` is a command-line utility that checks for `"if-changed"` and `"then-change"` comments in a repository diff and errors if dependent files need changes.

## Installation

```bash
cargo install if-changed
```

## Usage

```bash
Usage: if-changed [OPTIONS] [PATHSPEC]...

Arguments:
  [PATHSPEC]...  Git pathspec defining the set of files to check. By default, this will be all changed files between revisions

Options:
      --from-ref <FROM_REF>  The revision to compare against. By default, HEAD is used
      --to-ref <TO_REF>      The revision to compare with. By default, the current working tree is used
  -h, --help                 Print help
  -V, --version              Print version
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

Typically, to synchronize these enums, a common approach is to extract the enum values into a "source-of-truth" file. This then often requires significant effort to generate the enums through the build system or a script and to get everything working correctly. If the job is a one-off, the costs heavily outweigh the benefits.

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

Once this is commited, the next time `lib.rs` (or `lib.ts`) is changed in the lines surrounded by `"if-changed"` and `"then-change"`, `if-changed` will error if the other file (referenced in the `"then-change"` comment) does not have any changes in the corresponding named block.

> [!TIP]
>
> If you just want to assert that any change in a file is ok, then just reference the file without the name. For example,
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

### Pathspec list

If a block needs to specify several files, you can use commas and/or newlines to separate multiple paths/pathspecs. For example,

```c
// then-change(foo/bar, baz)

/// OR

// then-change(
//   foo/bar
//   bar
// )
```

These pathspecs follow the same rules as [`.gitignore` pathspecs](https://git-scm.com/docs/gitignore#_pattern_format) except relative pathspecs are always matched against the file it's in, even if the pathspec doesn't contain `/`. Use a beginning `/` to match the pathspec against the repository root, e.g. `/foo/bar`.

### Long paths

If a path is too long, you can use a shell continuation `\` to split it across multiple lines. For example, for the path `this/is/a/really/long/path/to/some/very/far/away/file`, you can do

```c
// then-change(
//   this/is/a/really/long/path/to/some/very/far/ \
//   away/file
// )
```

### Disabling `if-changed`

To disable `if-changed` on a file for a commit, add `Ignore-if-changed: <pathspec>, ... -- [REASON]` to the commit footer where `<pathspec>` is the file path. In general, `<pathspec>` can be any pattern allowed by [fnmatch](https://man7.org/linux/man-pages/man3/fnmatch.3.html).

> [!NOTE]
>
> If you want to disable `if-changed` when diffing the working tree, you can execute `if-changed` with the following:
>
> ```bash
> if-changed !<pathspec> '*'
> ```
>
> where `<pathspec>` is the file path you want to ignore. **It's important that `!<pathspec>` is first** (follows from `.gitignore` rules). Again, `<pathspec>` can be any pattern.

## Contributing

Contributions to `if-changed` are welcome! Please submit a pull request or create an issue on the GitHub repository.

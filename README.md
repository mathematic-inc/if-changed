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

### Motivating Example

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

### Disabling `if-changed`

To disable `if-changed` on a file for a commit, add `Ignore-if-changed: <path-spec>, ... -- [REASON]` to the commit footer where `<path-spec>` is the file path. In general, `<path-spec>` can be any pattern allowed by [fnmatch](https://man7.org/linux/man-pages/man3/fnmatch.3.html).

> [!NOTE]
>
> If you want to disable `if-changed` when diffing the working tree, you can execute `if-changed` with the following:
>
> ```bash
> if-changed !<path-spec> '*'
> ```
>
> where `<path-spec>` is the file path you want to ignore. **It's important that `!<path-spec>` is first** (follows from `.gitignore` rules). Again, `<path-spec>` can be any pattern.

## Contributing

Contributions to `if-changed` are welcome! Please submit a pull request or create an issue on the GitHub repository.

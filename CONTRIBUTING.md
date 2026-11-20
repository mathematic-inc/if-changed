# Contributing

AI agents handle most maintenance and implementation for this repository. For
us, reviewing an unsolicited pull request takes longer than implementing a
proposal after we have agreed on it.

## Propose a change

1. [Start a GitHub Discussion](https://github.com/mathematic-inc/if-changed/discussions/new)
   describing the problem or idea.
2. Wait for a Mathematic maintainer to review the proposal before doing
   implementation work.
3. If we accept the proposal, a Mathematic maintainer or agent will open the
   pull request.

When Mathematic implements a proposal, we will link the implementation pull
request to the Discussion and credit the original author.

GitHub restricts pull request creation to Mathematic maintainers and repository
collaborators who have write, maintain, or admin access, plus authorized
maintenance agents.

## Development

Install the pinned tools with `mise install`. Run the checks that CI runs with:

```sh
mise run check
```

Fix what the tools can fix with `mise run fix`. ToolConfig writes the settings
files of the tools. Change `.toolconfig.pkl`, not those files, and write them
again with `mise run generate`.

The minimum supported Rust version is 1.87. Check it with:

```sh
rustup run 1.87 cargo check --locked --workspace --all-features --all-targets
```

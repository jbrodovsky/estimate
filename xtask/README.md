# xtask

Repository tooling for `estimate`, written in Rust and run through Cargo, following the
[cargo-xtask](https://github.com/matklad/cargo-xtask) pattern. It is a workspace member with
`publish = false`, so it never ships with the library.

`.cargo/config.toml` defines the alias:

```toml
[alias]
xtask = "run --quiet --package xtask --"
```

so `cargo xtask <task>` builds and runs this crate with `<task>` as its argument.

## Tasks

| Task       | Command                | Also run by                        |
|------------|------------------------|------------------------------------|
| `doc-lint` | `cargo xtask doc-lint` | `just doc-lint`, CI job `doc-lint` |

### `doc-lint`

Checks that every public item in `estimate` and `estimate-derive` has a doc comment in the shape
set out in "Doc comments" in [AGENTS.md](../AGENTS.md). rustdoc and clippy can't check that
structure on stable Rust, so this task does.

It parses each crate with [`syn`](https://docs.rs/syn), starting at its `src/lib.rs` and following
`pub mod` declarations (inline, `name.rs` or `name/mod.rs`). For each public item it checks:

- **Presence.** The item has a doc comment.
- **Summary.** The first paragraph is a single line ending in `.`, `?` or `!`, and the comment
  doesn't open with a heading or code block.
- **Headings.** Headings are level 1 and spelled as the standard sections (`# Example` and `# Args`
  are flagged with the right spelling). They appear in the order `Arguments`, `Returns`, `Errors`,
  `Examples`, with no duplicates and `# Examples` last. Custom headings like `# Invariant` are
  allowed before `# Examples`.
- **Signature agreement** (functions and methods only):
  - `# Arguments` is present exactly when there are non-`self` parameters. It has one
    `` * `name` - description `` line per parameter, in signature order, and no stale names.
  - `# Returns` is present exactly when the return type isn't `()`.
  - `# Errors` is present exactly when the return type is a `Result`.
- **Examples.** Functions, types, traits and proc macros, plus any module that declares something
  public, have an `# Examples` section with a runnable block (` ``` `, ` ```rust ` or
  ` ```no_run `).
- **Code blocks.** No ` ```ignore ` (it never compiles) and no ` ```text ` (equations go in
  ` ```math `). Every block is closed.
- **Math.** Inline `$...$` is balanced, `$$...$$` isn't used, and inline math contains no
  backslash-before-punctuation escape (`\,`, `\{`, `\|`, `\\`) that rustdoc's Markdown would
  remove before KaTeX renders it.

Items marked `#[cfg(test)]` or `#[doc(hidden)]` are skipped, along with trait `impl` blocks and
anything that isn't plain `pub`. The full rules for each item kind are in the table in AGENTS.md.

The check is syntactic: it reads source files and never asks the compiler what is public. That
works because the workspace denies `unreachable_pub`, so every `pub` item in a `pub` module really
is public API.

Each problem prints as one line, and any problem makes the task exit non-zero:

```text
estimate/src/manifold.rs:42: estimate::manifold::Manifold::boxplus: returns `Result` but has no `# Errors`
```

`doc-lint` checks the layout of a doc comment, not whether its examples pass. Doctests run under
`cargo test` (the CI `test` job).

## Layout

The whole crate is [`src/main.rs`](src/main.rs):

- `main` dispatches on the first argument.
- `Checker` walks a crate's module tree and records a `Diagnostic` for each problem.
- `Doc` and `Section` split a doc comment into headed sections and track code blocks.
- `lint` and the functions it calls (`summary`, `headings`, `code_blocks`, `inline_math`,
  `function`, `arguments`, `examples`) implement the rules above.
- The `tests` module at the bottom holds unit tests for each rule.

Run the tests with:

```sh
cargo test -p xtask
```

## Changing it

- **Changing a doc rule:** update AGENTS.md and `doc-lint` together, and add a test for the new
  rule in `tests`.
- **Linting another crate:** add it to `CRATES` at the top of `main.rs`.
- **Adding a task:** add a match arm in `main` and update the usage message and the table above.
  If the task becomes part of CI, add a `just` recipe and a CI job for it too.

The workspace lints apply here as well (`[lints] workspace = true`). Anything a lint needs relaxed
goes in `[workspace.lints]` in the root `Cargo.toml`, never in an `#[allow]` at the call site.

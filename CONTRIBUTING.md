# Contributing to estimate

Thanks for your interest in **estimate**. This guide covers how to set up, what the code has to
look like, and what a pull request needs before it can merge.

Before starting on anything large, read the [README](README.md), especially the thesis, the
[scope and non-goals](README.md#scope-and-non-goals), and the design principles. Some work is
outside v1 scope on purpose, such as smoothers, factor graphs, parameter learning and
domain-specific models. If you want to propose something like that, or change a public trait,
open an issue first so we can agree on the shape before you write the code.

## Ways to contribute

- **Report a bug**, using the bug report issue form. A reachable panic in library code is always
  a bug.
- **Propose a feature or filter**, using the feature request form. Equations are welcome.
- **Improve the docs**: the rustdoc, or the user guide in [`docs/`](docs/).
- **Add a worked example** from a new domain. Examples that reuse the same traits are the best
  demonstration of the thesis.
- **Send a pull request** for an open issue.

For security vulnerabilities, follow [SECURITY.md](SECURITY.md) and don't open a public issue.

## Setup

- **Rust:** the toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml). With
  [rustup](https://rustup.rs) installed, the first `cargo` command in the repo installs the
  pinned version, its components (clippy, rustfmt, rust-src) and the `thumbv7em-none-eabihf`
  target. Use the pinned toolchain: pedantic and nursery lints change between releases, so any
  other version will disagree with CI.
- **[just](https://github.com/casey/just)** runs the task recipes in the [`justfile`](justfile).
- **[uv](https://docs.astral.sh/uv/)** builds the MkDocs user guide (`just user-guide`). Preview
  it with `uvx --with-requirements docs/requirements.txt mkdocs serve`.

## Before you open a pull request

Run the full CI suite locally:

```bash
just ci
```

`just ci` mirrors [`.github/workflows/ci.yml`](.github/workflows/ci.yml) job for job:

| Recipe       | Checks                                                                 |
| ------------ | ---------------------------------------------------------------------- |
| `fmt`        | `rustfmt` formatting (fix with `just fmt-fix`)                         |
| `clippy`     | clippy across the feature matrix: default, `no_std`, `+alloc`, `+libm` |
| `test`       | unit tests, property tests and doctests                                |
| `rustdoc`    | `cargo doc` with every rustdoc lint denied                             |
| `doc-lint`   | `cargo xtask doc-lint`: every public item follows the doc template     |
| `user-guide` | strict MkDocs build of the user guide                                  |
| `bare-metal` | the core builds for `thumbv7em-none-eabihf`, so any `std` use fails    |

Every warning is an error, both locally and in CI. Run `just` with no arguments to list all
recipes, so you can run one job at a time.

## Code rules

[AGENTS.md](AGENTS.md) holds the full rules, and it is the source of truth. In short:

- **Code reads like the equations.** Put the equation in a doc comment above the code that
  implements it. Name variables after the quantity they represent, not the symbol: `state`, not
  `x`; `covariance`, not `p`.
- **No panics in library code.** Don't use `unwrap`, `expect`, `panic!`, `todo!`,
  `unimplemented!` or `[]` indexing. Return a `FilterError` instead. Tests are exempt.
- **No `unsafe`.** It is forbidden workspace-wide.
- **The core is `no_std`.** Anything that needs allocation goes behind the `alloc` feature.
- **Lints are relaxed only in `[workspace.lints]`** in [`Cargo.toml`](Cargo.toml), with a
  comment giving the reason. Never use `#[allow]` at a call site.
- **Prefer larger single-file modules** (`foo.rs`). Move to a `foo/` directory only when several
  submodules are clearly coming.
- **Keep the MSRV in sync.** If the toolchain changes, `rust-toolchain.toml`, `rust-version` in
  `Cargo.toml` and `msrv` in `clippy.toml` all change together. Raising the MSRV is a breaking
  change.

### Documentation

Every public item follows the doc-comment template in
[AGENTS.md § Doc comments](AGENTS.md#doc-comments): a summary line, `# Arguments`, `# Returns`,
`# Errors`, then a runnable `# Examples` block that asserts something. Equations are KaTeX
(`$...$` inline, ` ```math ` blocks for display). `cargo xtask doc-lint` enforces the structure.
Never use ` ```ignore ` doctests.

### Public traits and design decisions

Read [docs/design.md](docs/design.md) before changing a public trait, and record any new design
decision there in the same pull request.

### Verification

Correctness is verified, not claimed. A new filter merges only with statistical evidence that it
is consistent, following the README's
[verification strategy](README.md#verification-strategy):

- **Analytic ground truth:** on linear-Gaussian problems, the filter matches the KF to numerical
  tolerance.
- **Monte Carlo consistency:** NEES and NIS tests against $\chi^2$ bounds.
- **Manifold properties:** `proptest` checks of the $\boxplus$/$\boxminus$ axioms, and of
  analytic against finite-difference Jacobians, for every new manifold.

## Pull requests

- Keep each pull request to one logical change, and explain *why* it is needed as well as what
  it changes.
- Add an entry under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md) for any user-visible
  change.
- Fill in the checklist in the pull request template.

## Code of conduct

Everyone taking part is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed under the
[MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE) licenses, without any additional terms or
conditions.

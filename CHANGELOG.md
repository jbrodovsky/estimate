# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Before 1.0, a
minor version bump (0.x) may contain breaking changes.

## [Unreleased]

### Added

- The core `estimate` crate scaffold: the `manifold`, `model`, `covariance`, `filter`,
  `gaussian`, `particle`, `discrete` and `fusion` modules, and `FilterError`.
- The `Manifold` trait ($\boxplus$ / $\boxminus$), with `manifold::axioms::check` for testing
  implementations.
- The `std`, `alloc` and `libm` features. The core is `no_std` and builds for
  `thumbv7em-none-eabihf`.
- A doc-comment standard, enforced by `cargo xtask doc-lint`, with KaTeX math in rustdoc.
- The MkDocs user guide and `docs/design.md`.
- CI and a `justfile` that mirrors it (`just ci`).
- Community files: CONTRIBUTING, CITATION, CODE_OF_CONDUCT, SECURITY, and issue and pull
  request templates.

[Unreleased]: https://github.com/jbrodovsky/estimate/commits/main

# estimate — working notes for agents

- Code reads like the equations. Name variables after the math (x, p, f, h, q, r, nu, k) and put the equation in a doc comment above the code that implements it. Variable names should represent the quality not the mathematical symbol (e.g. $x$ should be called `state`)
- No panics in library code: no `unwrap`, `expect`, `panic!`, `todo!` or `[]` indexing. Return `FilterError`.
- The core is `no_std`. Anything needing allocation goes behind the `alloc` feature.
- Relax lints only in `[workspace.lints]` in Cargo.toml, with a reason. Never `#[allow]` at a call
  site.
- Toolchain is pinned in rust-toolchain.toml; keep `rust-version` and clippy's `msrv` in sync.
- See docs/design.md before changing a public trait.
- Read the README for project philosophy and design guide

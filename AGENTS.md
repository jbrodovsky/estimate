# estimate — working notes for agents

- Code reads like the equations. Name variables after the math (x, p, f, h, q, r, nu, k) and put the equation in a doc comment above the code that implements it. Variable names should represent the quality not the mathematical symbol (e.g. $x$ should be called `state`)
- No panics in library code: no `unwrap`, `expect`, `panic!`, `todo!` or `[]` indexing. Return `FilterError`.
- The core is `no_std`. Anything needing allocation goes behind the `alloc` feature.
- Relax lints only in `[workspace.lints]` in Cargo.toml, with a reason. Never `#[allow]` at a call
  site.
- Toolchain is pinned in rust-toolchain.toml; keep `rust-version` and clippy's `msrv` in sync.
- See docs/design.md before changing a public trait.
- Read the README for project philosophy and design guide
- I prefer larger rust modules in single *.rs files. Only move to folder-based modules when it makes sense that we'll have several sub-modules

## Doc comments

Every public item is documented in one shape, in this order, leaving out sections that don't
apply. `cargo xtask doc-lint` (CI job `doc-lint`, `just doc-lint`) enforces it; doctests run under
`cargo test`.

````rust
/// Moves a state along a tangent vector.
///
/// Optional details: invariants, references, and the equation in a ```math block.
///
/// ```math
/// x \boxplus \delta = x \cdot \operatorname{Exp}(\delta)
/// ```
///
/// # Arguments
///
/// * `state` - The state $x \in \mathcal{M}$.
/// * `delta` - The tangent step $\delta$, with `tangent_dim()` rows.
///
/// # Returns
///
/// The moved state $x \boxplus \delta$.
///
/// # Errors
///
/// [`FilterError::NonFinite`] if $\delta$ contains a NaN or $\pm\infty$.
///
/// # Examples
///
/// ```
/// # use estimate::FilterError;
/// let moved = step(&state, &delta)?;
/// assert!((moved.angle() - 0.5).abs() < 1e-12);
/// # Ok::<(), FilterError>(())
/// ```
pub fn step(state: &Circle, delta: &Vector1<f64>) -> Result<Circle, FilterError>
````

| Item                                                     | Summary | `# Arguments`      | `# Returns`  | `# Errors`         | `# Examples` |
|----------------------------------------------------------|---------|--------------------|--------------|--------------------|--------------|
| `pub fn`, inherent `pub` method, trait method            | yes     | if non-`self` args | if not `()`  | iff returns `Result` | yes        |
| `pub` trait, struct, enum, union, type alias             | yes     | –                  | –            | –                  | yes          |
| Module or crate root declaring anything `pub`            | yes     | –                  | –            | –                  | yes          |
| Stub module, field, variant, const, static, assoc. item  | yes     | –                  | –            | –                  | optional     |

- **Summary:** the first line, alone in its paragraph, ending in `.`, `?` or `!`. Every other
  paragraph ends in punctuation too (clippy `doc_paragraphs_missing_punctuation`).
- **Headings:** level-1 only (`# Examples`, not `## Examples` or `# Example`). The standard
  sections go in the order above, and `# Examples` comes last. Custom sections (`# Invariant`,
  `# For implementors`) go before `# Examples`.
- **Arguments:** one `` * `name` - description `` line per parameter, in signature order, with
  continuation lines indented. Leave out `self`.
- **Examples:** at least one runnable block (` ``` `, ` ```rust ` or ` ```no_run `) that asserts
  something. Use `# ` lines for setup the reader doesn't need, such as a toy `Manifold` impl. Write
  fallible examples with `?` and end them with `# Ok::<(), E>(())`, never `unwrap`. Variable names
  follow the naming rule above (`state`, not `x`).
- **Equations** are KaTeX, in LaTeX notation matching the mkdocs user guide (`\boxplus`,
  `\mathcal{M}`, `\lVert\delta\rVert`). Write display math in a ` ```math ` block, and inline math
  as `$...$`, including single symbols in prose (`$x$`, `$\delta$`, `$r_1$`). GitHub renders both
  forms, and rustdoc loads KaTeX through `estimate/katex-header.html`.
  - Don't use ` ```text ` blocks, `$$...$$` paragraphs or Unicode math (⊞, δ, ℝⁿ) in doc prose.
    Unicode is fine in code comments inside a doctest, since that is code.
  - rustdoc's Markdown has no math mode, so inside inline `$...$` a backslash before ASCII
    punctuation is eaten: `\,`, `\{`, `\|`, `\\`. Write `\quad`, `\lbrace`, `\lVert`, or move the
    equation into a ` ```math ` block, which keeps TeX verbatim (`\\` and `\,` are fine there).
  - Wrap long TeX lines inside the block, since a newline is just whitespace to TeX.
- **Never ` ```ignore `.** It never compiles, so it goes stale silently. Doctests may use
  dev-dependencies (proptest, approx).
- `# Panics` should never appear, because library code doesn't panic. `# Safety` should never
  appear either, because `unsafe` is forbidden.
- Private items may follow the template, but only public ones are checked. Trait `impl` blocks,
  `#[cfg(test)]` and `#[doc(hidden)]` items are skipped.

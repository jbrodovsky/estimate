//! The manifold layer: states that are not vector spaces.
//!
//! A state lives on an n-dimensional manifold 𝓜; all uncertainty lives in its tangent space ℝⁿ.
//! Two operators define the structure (Hertzberg et al., 2013):
//!
//! ```text
//! ⊞ : 𝓜 × ℝⁿ → 𝓜        x ⊞ (y ⊟ x) = y
//! ⊟ : 𝓜 × 𝓜 → ℝⁿ        (x ⊞ δ) ⊟ x = δ
//! ```
//!
//! Planned contents:
//!
//! - the `Manifold` trait, with associated `Scalar` and tangent `Dim` (decision: associated
//!   types, see `docs/design.md`)
//! - [`euclidean`] — ℝⁿ, the trivial case
//! - [`so2`], [`so3`] — rotations; perturbation side (left/right) as a type parameter
//! - later: SE(2), SE(3), SE₂(3), products via `#[derive(Manifold)]`

pub mod euclidean;
pub mod so2;
pub mod so3;

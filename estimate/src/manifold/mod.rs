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
//! - [`Manifold`] — the trait, with associated `Scalar` and tangent `Dim` (decision: associated
//!   types, see `docs/design.md`)
//! - [`axioms`] — checks of both axioms for any implementor, for property tests
//! - [`euclidean`] — ℝⁿ, the trivial case
//! - [`so2`], [`so3`] — rotations; perturbation side (left/right) as a type parameter
//! - later: SE(2), SE(3), SE₂(3), products via `#[derive(Manifold)]`

pub mod axioms;
pub mod euclidean;
pub mod so2;
pub mod so3;

use nalgebra::allocator::Allocator;
use nalgebra::{DefaultAllocator, Dim, OVector, RealField};

/// A tangent vector δ ∈ ℝⁿ of the manifold `X`.
pub type Tangent<X> = OVector<<X as Manifold>::Scalar, <X as Manifold>::Dim>;

/// An n-dimensional manifold 𝓜 with the ⊞/⊟ operators.
///
/// ```text
/// ⊞ : 𝓜 × ℝⁿ → 𝓜        x ⊞ (y ⊟ x) = y
/// ⊟ : 𝓜 × 𝓜 → ℝⁿ        (x ⊞ δ) ⊟ x = δ
/// ```
///
/// On ℝⁿ, ⊞ is `+` and ⊟ is `−`. On a Lie group, x ⊞ δ = x · Exp(δ) and y ⊟ x = Log(x⁻¹ · y)
/// (right perturbation). The second axiom need only hold for ‖δ‖ inside the injectivity radius
/// (‖δ‖ < π on SO(2) and SO(3)), where ⊟ undoes ⊞.
///
/// `Dim` is `Const<N>` for a stack-allocated tangent space or `Dyn` (with `alloc`) for one sized
/// at runtime; [`tangent_dim`](Manifold::tangent_dim) gives n in both cases. Every tangent
/// vector passed to or returned from ⊞/⊟ has exactly n rows.
///
/// # Example
///
/// The circle S¹ as an angle θ ∈ [−π, π):
///
/// ```
/// use core::f64::consts::PI;
/// use estimate::manifold::{Manifold, axioms};
/// use estimate::nalgebra::{U1, Vector1};
///
/// #[derive(Clone, Debug)]
/// struct Circle(f64);
///
/// /// wrap(a) = ((a + π) mod 2π) − π
/// fn wrap(a: f64) -> f64 {
///     (a + PI).rem_euclid(2.0 * PI) - PI
/// }
///
/// impl Manifold for Circle {
///     type Scalar = f64;
///     type Dim = U1;
///     fn tangent_dim(&self) -> usize {
///         1
///     }
///     /// θ ⊞ δ = wrap(θ + δ)
///     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
///         Self(wrap(self.0 + delta.x))
///     }
///     /// φ ⊟ θ = wrap(φ − θ)
///     fn boxminus(&self, x: &Self) -> Vector1<f64> {
///         Vector1::new(wrap(self.0 - x.0))
///     }
/// }
///
/// let (x, y) = (Circle(3.0), Circle(-3.0));
/// // The short way round: y ⊟ x crosses ±π instead of going back through 0.
/// assert!((y.boxminus(&x).x - (2.0 * PI - 6.0)).abs() < 1e-12);
/// axioms::check(&x, &y, &Vector1::new(0.5), 1e-12)?;
/// # Ok::<(), axioms::AxiomViolation<f64>>(())
/// ```
///
/// # Testing an implementation
///
/// [`axioms::check`] verifies both axioms for one sample of (x, y, δ). Call it from a property
/// test with samples drawn from your manifold:
///
/// ```ignore
/// proptest! {
///     #[test]
///     fn my_manifold_axioms(x in my_state(), y in my_state(), delta in small_tangent()) {
///         estimate::manifold::axioms::check(&x, &y, &delta, 1e-9)?;
///     }
/// }
/// ```
pub trait Manifold: Clone
where
    DefaultAllocator: Allocator<Self::Dim>,
{
    /// Scalar type of the tangent space, e.g. `f32` or `f64`.
    type Scalar: RealField;

    /// Tangent dimension n: `Const<N>` or `Dyn`.
    type Dim: Dim;

    /// The tangent dimension n of this state.
    ///
    /// For `Dim = Const<N>` this is `N`; for `Dyn` it is the runtime size.
    fn tangent_dim(&self) -> usize;

    /// x ⊞ δ: move from x along the tangent vector δ.
    ///
    /// ```text
    /// ⊞ : 𝓜 × ℝⁿ → 𝓜
    /// ```
    ///
    /// `delta` has [`tangent_dim`](Manifold::tangent_dim) rows.
    #[must_use]
    fn boxplus(&self, delta: &Tangent<Self>) -> Self;

    /// y ⊟ x: the tangent vector δ at x that reaches y, so that x ⊞ δ = y.
    ///
    /// ```text
    /// ⊟ : 𝓜 × 𝓜 → ℝⁿ
    /// ```
    ///
    /// `self` is y and the argument is x: `y.boxminus(&x)` reads y ⊟ x.
    #[must_use]
    fn boxminus(&self, x: &Self) -> Tangent<Self>;
}

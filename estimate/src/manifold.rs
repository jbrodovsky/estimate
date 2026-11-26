//! The manifold layer: states that are not vector spaces.
//!
//! A state lives on an $n$-dimensional manifold $\mathcal{M}$; all uncertainty lives in its tangent
//! space $\mathbb{R}^n$.
//! Two operators define the structure (Hertzberg et al., 2013):
//!
//! ```math
//! \begin{aligned}
//! \boxplus &: \mathcal{M} \times \mathbb{R}^n \to \mathcal{M},
//!   & x \boxplus (y \boxminus x) &= y \\
//! \boxminus &: \mathcal{M} \times \mathcal{M} \to \mathbb{R}^n,
//!   & (x \boxplus \delta) \boxminus x &= \delta
//! \end{aligned}
//! ```
//!
//! - [`Manifold`] — the trait, with associated `Scalar` and tangent `Dim` (decision: associated
//!   types, see `docs/design.md`)
//! - [`axioms`] — checks of both axioms for any implementor, for property tests
//! - [`euclidean`] — $\mathbb{R}^n$, the trivial case
//! - [`so2`], [`so3`] — rotations; perturbation side (left/right) as a type parameter
//! - later: $SE(2)$, $SE(3)$, $SE_2(3)$, products via `#[derive(Manifold)]`
//!
//! # Examples
//!
//! $\mathbb{R}^1$ as a manifold, where $\boxplus$ is `+` and $\boxminus$ is `-`, checked against
//! both axioms:
//!
//! ```
//! use estimate::manifold::{Manifold, axioms};
//! use estimate::nalgebra::{U1, Vector1};
//!
//! #[derive(Clone, Debug)]
//! struct Position(f64);
//!
//! impl Manifold for Position {
//!     type Scalar = f64;
//!     type Dim = U1;
//!     fn tangent_dim(&self) -> usize {
//!         1
//!     }
//!     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
//!         Self(self.0 + delta.x)
//!     }
//!     fn boxminus(&self, x: &Self) -> Vector1<f64> {
//!         Vector1::new(self.0 - x.0)
//!     }
//! }
//!
//! let (start, end) = (Position(1.0), Position(4.0));
//! assert_eq!(end.boxminus(&start), Vector1::new(3.0));
//! axioms::check(&start, &end, &Vector1::new(0.5), 1e-12)?;
//! # Ok::<(), axioms::AxiomViolation<f64>>(())
//! ```

use nalgebra::allocator::Allocator;
use nalgebra::{DefaultAllocator, Dim, OVector, RealField};

/// A tangent vector $\delta \in \mathbb{R}^n$ of the manifold `X`.
///
/// # Examples
///
/// ```
/// # use core::f64::consts::PI;
/// # use estimate::manifold::Manifold;
/// # use estimate::nalgebra::{U1, Vector1};
/// # #[derive(Clone, Debug)]
/// # struct Circle(f64);
/// # fn wrap(angle: f64) -> f64 {
/// #     (angle + PI).rem_euclid(2.0 * PI) - PI
/// # }
/// # impl Manifold for Circle {
/// #     type Scalar = f64;
/// #     type Dim = U1;
/// #     fn tangent_dim(&self) -> usize {
/// #         1
/// #     }
/// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
/// #         Self(wrap(self.0 + delta.x))
/// #     }
/// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
/// #         Vector1::new(wrap(self.0 - x.0))
/// #     }
/// # }
/// use estimate::manifold::Tangent;
///
/// let delta: Tangent<Circle> = Vector1::new(0.25);
/// assert_eq!(delta.nrows(), Circle(0.0).tangent_dim());
/// ```
pub type Tangent<X> = OVector<<X as Manifold>::Scalar, <X as Manifold>::Dim>;

/// An $n$-dimensional manifold $\mathcal{M}$ with the $\boxplus$ and $\boxminus$ operators.
///
/// ```math
/// \begin{aligned}
/// \boxplus &: \mathcal{M} \times \mathbb{R}^n \to \mathcal{M},
///   & x \boxplus (y \boxminus x) &= y \\
/// \boxminus &: \mathcal{M} \times \mathcal{M} \to \mathbb{R}^n,
///   & (x \boxplus \delta) \boxminus x &= \delta
/// \end{aligned}
/// ```
///
/// On $\mathbb{R}^n$, $\boxplus$ is `+` and $\boxminus$ is `-`. On a Lie group,
/// $x \boxplus \delta = x \cdot \operatorname{Exp}(\delta)$ and
/// $y \boxminus x = \operatorname{Log}(x^{-1} y)$ (right perturbation). The second axiom need only
/// hold for $\lVert\delta\rVert$ inside the injectivity radius ($\lVert\delta\rVert < \pi$ on
/// $SO(2)$ and $SO(3)$), where $\boxminus$ undoes $\boxplus$.
///
/// `Dim` is `Const<N>` for a stack-allocated tangent space or `Dyn` (with `alloc`) for one sized
/// at runtime; [`tangent_dim`](Manifold::tangent_dim) gives $n$ in both cases. Every tangent
/// vector passed to or returned from $\boxplus$ and $\boxminus$ has exactly $n$ rows.
///
/// # Testing an implementation
///
/// [`axioms::check`] verifies both axioms for one sample of $(x, y, \delta)$. Call it from a
/// property test with samples drawn from your manifold; the [`axioms`] module docs show one with
/// proptest.
///
/// # Examples
///
/// The circle $S^1$ as an angle $\theta \in [-\pi, \pi)$:
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
/// fn wrap(angle: f64) -> f64 {
///     (angle + PI).rem_euclid(2.0 * PI) - PI
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
/// let (start, end) = (Circle(3.0), Circle(-3.0));
/// // The short way round: end ⊟ start crosses ±π instead of going back through 0.
/// assert!((end.boxminus(&start).x - (2.0 * PI - 6.0)).abs() < 1e-12);
/// axioms::check(&start, &end, &Vector1::new(0.5), 1e-12)?;
/// # Ok::<(), axioms::AxiomViolation<f64>>(())
/// ```
pub trait Manifold: Clone
where
    DefaultAllocator: Allocator<Self::Dim>,
{
    /// Scalar type of the tangent space, e.g. `f32` or `f64`.
    type Scalar: RealField;

    /// Tangent dimension n: `Const<N>` or `Dyn`.
    type Dim: Dim;

    /// The tangent dimension $n$ of this state.
    ///
    /// # Returns
    ///
    /// `N` for `Dim = Const<N>`, or the runtime size for `Dyn`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::f64::consts::PI;
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{U1, Vector1};
    /// # #[derive(Clone, Debug)]
    /// # struct Circle(f64);
    /// # fn wrap(angle: f64) -> f64 {
    /// #     (angle + PI).rem_euclid(2.0 * PI) - PI
    /// # }
    /// # impl Manifold for Circle {
    /// #     type Scalar = f64;
    /// #     type Dim = U1;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         1
    /// #     }
    /// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
    /// #         Self(wrap(self.0 + delta.x))
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
    /// #         Vector1::new(wrap(self.0 - x.0))
    /// #     }
    /// # }
    /// assert_eq!(Circle(0.5).tangent_dim(), 1);
    /// ```
    fn tangent_dim(&self) -> usize;

    /// $x \boxplus \delta$: move from $x$ along the tangent vector $\delta$.
    ///
    /// ```math
    /// \boxplus : \mathcal{M} \times \mathbb{R}^n \to \mathcal{M}
    /// ```
    ///
    /// # Arguments
    ///
    /// * `delta` - The tangent vector $\delta$, with [`tangent_dim`](Manifold::tangent_dim) rows.
    ///
    /// # Returns
    ///
    /// The state $x \boxplus \delta$, where $x$ is `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::f64::consts::PI;
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{U1, Vector1};
    /// # #[derive(Clone, Debug)]
    /// # struct Circle(f64);
    /// # fn wrap(angle: f64) -> f64 {
    /// #     (angle + PI).rem_euclid(2.0 * PI) - PI
    /// # }
    /// # impl Manifold for Circle {
    /// #     type Scalar = f64;
    /// #     type Dim = U1;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         1
    /// #     }
    /// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
    /// #         Self(wrap(self.0 + delta.x))
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
    /// #         Vector1::new(wrap(self.0 - x.0))
    /// #     }
    /// # }
    /// let heading = Circle(3.0);
    /// let turned = heading.boxplus(&Vector1::new(0.5));
    /// // 3.5 rad is past π, so it wraps to 3.5 − 2π.
    /// assert!((turned.0 - (3.5 - 2.0 * PI)).abs() < 1e-12);
    /// ```
    #[must_use]
    fn boxplus(&self, delta: &Tangent<Self>) -> Self;

    /// $y \boxminus x$: the tangent vector $\delta$ at $x$ that reaches $y$.
    ///
    /// That is, $x \boxplus (y \boxminus x) = y$.
    ///
    /// ```math
    /// \boxminus : \mathcal{M} \times \mathcal{M} \to \mathbb{R}^n
    /// ```
    ///
    /// `self` is $y$ and the argument is $x$: `y.boxminus(&x)` reads $y \boxminus x$.
    ///
    /// # Arguments
    ///
    /// * `x` - The base point $x$, whose tangent space the result lives in.
    ///
    /// # Returns
    ///
    /// The tangent vector $y \boxminus x$, with [`tangent_dim`](Manifold::tangent_dim) rows.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::f64::consts::PI;
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{U1, Vector1};
    /// # #[derive(Clone, Debug)]
    /// # struct Circle(f64);
    /// # fn wrap(angle: f64) -> f64 {
    /// #     (angle + PI).rem_euclid(2.0 * PI) - PI
    /// # }
    /// # impl Manifold for Circle {
    /// #     type Scalar = f64;
    /// #     type Dim = U1;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         1
    /// #     }
    /// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
    /// #         Self(wrap(self.0 + delta.x))
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
    /// #         Vector1::new(wrap(self.0 - x.0))
    /// #     }
    /// # }
    /// let (start, end) = (Circle(3.0), Circle(-3.0));
    /// let delta = end.boxminus(&start);
    /// // The short way round, across ±π, rather than back through 0.
    /// assert!((delta.x - (2.0 * PI - 6.0)).abs() < 1e-12);
    /// ```
    #[must_use]
    fn boxminus(&self, x: &Self) -> Tangent<Self>;
}

/// Checks of the $\boxplus$/$\boxminus$ axioms for any [`Manifold`], for property-based tests.
///
/// ```math
/// \begin{aligned}
/// (x \boxplus \delta) \boxminus x &= \delta,
///   & r_1 &= \lVert (x \boxplus \delta) \boxminus x - \delta \rVert \\
/// x \boxplus (y \boxminus x) &= y,
///   & r_2 &= \lVert (x \boxplus (y \boxminus x)) \boxminus y \rVert
/// \end{aligned}
/// ```
///
/// Equality on $\mathcal{M}$ is measured through $\boxminus$ itself, so the checks need nothing
/// from the state type beyond the trait. The two axioms together rule out a degenerate $\boxminus$:
/// one that always returns zero passes $r_2$ but fails $r_1$.
///
/// The checks are plain functions with no allocation, so they run under `no_std`. The sampling of
/// $(x, y, \delta)$ is the implementor's: draw $\delta$ inside the injectivity radius
/// ($\lVert\delta\rVert < \pi$ on rotations), where $\boxminus$ undoes $\boxplus$.
/// [`AxiomViolation`](axioms::AxiomViolation) implements `Error`, so `?` turns it into a proptest
/// failure.
///
/// # Examples
///
/// A property test of the circle $S^1$. In your own tests, put `#[test]` on the function inside
/// `proptest!`; here it is called directly so the doctest runs it:
///
/// ```
/// # use core::f64::consts::PI;
/// # use estimate::manifold::Manifold;
/// # use estimate::nalgebra::{U1, Vector1};
/// # #[derive(Clone, Debug)]
/// # struct Circle(f64);
/// # fn wrap(angle: f64) -> f64 {
/// #     (angle + PI).rem_euclid(2.0 * PI) - PI
/// # }
/// # impl Manifold for Circle {
/// #     type Scalar = f64;
/// #     type Dim = U1;
/// #     fn tangent_dim(&self) -> usize {
/// #         1
/// #     }
/// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
/// #         Self(wrap(self.0 + delta.x))
/// #     }
/// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
/// #         Vector1::new(wrap(self.0 - x.0))
/// #     }
/// # }
/// use estimate::manifold::axioms;
/// use proptest::prelude::*;
///
/// proptest! {
///     fn circle_satisfies_axioms(x in -PI..PI, y in -PI..PI, d in -3.0..3.0_f64) {
///         axioms::check(&Circle(x), &Circle(y), &Vector1::new(d), 1e-9)?;
///     }
/// }
///
/// circle_satisfies_axioms();
/// ```
pub mod axioms {

    use core::fmt;

    use nalgebra::allocator::Allocator;
    use nalgebra::{DefaultAllocator, Dim};

    use super::{Manifold, Tangent};

    /// An axiom that failed for one sample, with the residual that broke it.
    ///
    /// # Examples
    ///
    /// ```
    /// use estimate::manifold::axioms::AxiomViolation;
    ///
    /// let violation = AxiomViolation::BoxplusBoxminus { residual: 0.5, tol: 1e-9 };
    /// assert_eq!(violation.to_string(), "(x ⊞ δ) ⊟ x ≠ δ: residual 0.5 > tolerance 0.000000001");
    /// ```
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum AxiomViolation<T> {
        /// A tangent vector had the wrong number of rows.
        TangentDim {
            /// The state's [`Manifold::tangent_dim`].
            expected: usize,
            /// Rows of the tangent vector given to or returned by $\boxplus$ or $\boxminus$.
            found: usize,
        },
        /// $(x \boxplus \delta) \boxminus x \neq \delta$: residual $r_1$ exceeded the tolerance.
        BoxplusBoxminus {
            /// The residual $r_1 = \lVert (x \boxplus \delta) \boxminus x - \delta \rVert$.
            residual: T,
            /// The tolerance it exceeded.
            tol: T,
        },
        /// $x \boxplus (y \boxminus x) \neq y$: residual $r_2$ exceeded the tolerance.
        BoxminusBoxplus {
            /// The residual $r_2 = \lVert (x \boxplus (y \boxminus x)) \boxminus y \rVert$.
            residual: T,
            /// The tolerance it exceeded.
            tol: T,
        },
    }

    impl<T: fmt::Display> fmt::Display for AxiomViolation<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::TangentDim { expected, found } => {
                    write!(f, "tangent vector has {found} rows, expected tangent_dim = {expected}")
                }
                Self::BoxplusBoxminus { residual, tol } => {
                    write!(f, "(x ⊞ δ) ⊟ x ≠ δ: residual {residual} > tolerance {tol}")
                }
                Self::BoxminusBoxplus { residual, tol } => {
                    write!(f, "x ⊞ (y ⊟ x) ≠ y: residual {residual} > tolerance {tol}")
                }
            }
        }
    }

    impl<T: fmt::Debug + fmt::Display> core::error::Error for AxiomViolation<T> {}

    /// Checks that a tangent vector $\delta$ has $n$ = `x.tangent_dim()` rows.
    ///
    /// Only a `Dyn` manifold can get this wrong: a `Const<N>` tangent vector always has $N$ rows.
    ///
    /// # Arguments
    ///
    /// * `x` - The state whose tangent space $\delta$ should belong to.
    /// * `delta` - The tangent vector $\delta$ to check.
    ///
    /// # Returns
    ///
    /// `Ok(())` if $\delta$ has $n$ rows.
    ///
    /// # Errors
    ///
    /// [`AxiomViolation::TangentDim`] if it does not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{DVector, Dyn};
    /// # #[derive(Clone, Debug)]
    /// # struct Position(DVector<f64>);
    /// # impl Manifold for Position {
    /// #     type Scalar = f64;
    /// #     type Dim = Dyn;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         self.0.len()
    /// #     }
    /// #     fn boxplus(&self, delta: &DVector<f64>) -> Self {
    /// #         Self(&self.0 + delta)
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> DVector<f64> {
    /// #         &self.0 - &x.0
    /// #     }
    /// # }
    /// use estimate::manifold::axioms::{AxiomViolation, check_tangent_dim};
    ///
    /// let position = Position(DVector::zeros(3));
    /// assert_eq!(check_tangent_dim(&position, &DVector::zeros(3)), Ok(()));
    /// assert_eq!(
    ///     check_tangent_dim(&position, &DVector::zeros(2)),
    ///     Err(AxiomViolation::TangentDim { expected: 3, found: 2 }),
    /// );
    /// ```
    pub fn check_tangent_dim<X: Manifold>(x: &X, delta: &Tangent<X>) -> Result<(), AxiomViolation<X::Scalar>>
    where
        DefaultAllocator: Allocator<X::Dim>,
    {
        let n = x.tangent_dim();
        let rows = delta.nrows();
        if rows == n {
            Ok(())
        } else {
            Err(AxiomViolation::TangentDim {
                expected: n,
                found: rows,
            })
        }
    }

    /// Checks the first axiom, $(x \boxplus \delta) \boxminus x = \delta$, for one sample.
    ///
    /// ```math
    /// r_1 = \lVert (x \boxplus \delta) \boxminus x - \delta \rVert \le \mathrm{tol}
    /// ```
    ///
    /// # Arguments
    ///
    /// * `x` - The base point $x$.
    /// * `delta` - The step $\delta$, inside the injectivity radius.
    /// * `tol` - The largest accepted residual $r_1$.
    ///
    /// # Returns
    ///
    /// `Ok(())` if $r_1 \le$ `tol`.
    ///
    /// # Errors
    ///
    /// [`AxiomViolation::TangentDim`] if $\delta$ or $(x \boxplus \delta) \boxminus x$ has the
    /// wrong size, and [`AxiomViolation::BoxplusBoxminus`] if $r_1$ exceeds `tol` or is not a
    /// number.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::f64::consts::PI;
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{U1, Vector1};
    /// # #[derive(Clone, Debug)]
    /// # struct Circle(f64);
    /// # fn wrap(angle: f64) -> f64 {
    /// #     (angle + PI).rem_euclid(2.0 * PI) - PI
    /// # }
    /// # impl Manifold for Circle {
    /// #     type Scalar = f64;
    /// #     type Dim = U1;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         1
    /// #     }
    /// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
    /// #         Self(wrap(self.0 + delta.x))
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
    /// #         Vector1::new(wrap(self.0 - x.0))
    /// #     }
    /// # }
    /// use estimate::manifold::axioms::{AxiomViolation, check_boxplus_boxminus};
    ///
    /// check_boxplus_boxminus(&Circle(3.0), &Vector1::new(0.5), 1e-12)?;
    /// // ‖δ‖ = 4 > π is outside the injectivity radius: ⊟ takes the short way back.
    /// let beyond = check_boxplus_boxminus(&Circle(0.0), &Vector1::new(4.0), 1e-12);
    /// assert!(matches!(beyond, Err(AxiomViolation::BoxplusBoxminus { .. })));
    /// # Ok::<(), estimate::manifold::axioms::AxiomViolation<f64>>(())
    /// ```
    pub fn check_boxplus_boxminus<X: Manifold>(
        x: &X,
        delta: &Tangent<X>,
        tol: X::Scalar,
    ) -> Result<(), AxiomViolation<X::Scalar>>
    where
        DefaultAllocator: Allocator<X::Dim>,
    {
        check_tangent_dim(x, delta)?;
        let xp = x.boxplus(delta);
        let d = xp.boxminus(x);
        check_tangent_dim(x, &d)?;
        let r = d.metric_distance(delta);
        if r <= tol {
            Ok(())
        } else {
            Err(AxiomViolation::BoxplusBoxminus { residual: r, tol })
        }
    }

    /// Checks the second axiom, $x \boxplus (y \boxminus x) = y$, for one sample.
    ///
    /// ```math
    /// r_2 = \lVert (x \boxplus (y \boxminus x)) \boxminus y \rVert \le \mathrm{tol}
    /// ```
    ///
    /// # Arguments
    ///
    /// * `x` - The base point $x$.
    /// * `y` - The target point $y$.
    /// * `tol` - The largest accepted residual $r_2$.
    ///
    /// # Returns
    ///
    /// `Ok(())` if $r_2 \le$ `tol`.
    ///
    /// # Errors
    ///
    /// [`AxiomViolation::TangentDim`] if a tangent vector has the wrong size, and
    /// [`AxiomViolation::BoxminusBoxplus`] if $r_2$ exceeds `tol` or is not a number.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::f64::consts::PI;
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{U1, Vector1};
    /// # #[derive(Clone, Debug)]
    /// # struct Circle(f64);
    /// # fn wrap(angle: f64) -> f64 {
    /// #     (angle + PI).rem_euclid(2.0 * PI) - PI
    /// # }
    /// # impl Manifold for Circle {
    /// #     type Scalar = f64;
    /// #     type Dim = U1;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         1
    /// #     }
    /// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
    /// #         Self(wrap(self.0 + delta.x))
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
    /// #         Vector1::new(wrap(self.0 - x.0))
    /// #     }
    /// # }
    /// use estimate::manifold::axioms::check_boxminus_boxplus;
    ///
    /// // Across ±π: y ⊟ x is the short way round, and x ⊞ (y ⊟ x) still lands on y.
    /// check_boxminus_boxplus(&Circle(3.0), &Circle(-3.0), 1e-12)?;
    /// # Ok::<(), estimate::manifold::axioms::AxiomViolation<f64>>(())
    /// ```
    pub fn check_boxminus_boxplus<X: Manifold>(x: &X, y: &X, tol: X::Scalar) -> Result<(), AxiomViolation<X::Scalar>>
    where
        DefaultAllocator: Allocator<X::Dim>,
    {
        let delta = y.boxminus(x);
        check_tangent_dim(x, &delta)?;
        let yp = x.boxplus(&delta);
        let e = yp.boxminus(y);
        check_tangent_dim(y, &e)?;
        let r = e.norm();
        if r <= tol {
            Ok(())
        } else {
            Err(AxiomViolation::BoxminusBoxplus { residual: r, tol })
        }
    }

    /// Checks both axioms for one sample $(x, y, \delta)$.
    ///
    /// ```math
    /// (x \boxplus \delta) \boxminus x = \delta, \qquad x \boxplus (y \boxminus x) = y
    /// ```
    ///
    /// For `Dim = Const<N>`, also checks that `tangent_dim` reports $N$.
    ///
    /// # Arguments
    ///
    /// * `x` - The base point $x$.
    /// * `y` - The target point $y$.
    /// * `delta` - The step $\delta$, inside the injectivity radius.
    /// * `tol` - The largest accepted residual for either axiom.
    ///
    /// # Returns
    ///
    /// `Ok(())` if both axioms hold within `tol`.
    ///
    /// # Errors
    ///
    /// The first [`AxiomViolation`] found.
    ///
    /// # Examples
    ///
    /// ```
    /// # use core::f64::consts::PI;
    /// # use estimate::manifold::Manifold;
    /// # use estimate::nalgebra::{U1, Vector1};
    /// # #[derive(Clone, Debug)]
    /// # struct Circle(f64);
    /// # fn wrap(angle: f64) -> f64 {
    /// #     (angle + PI).rem_euclid(2.0 * PI) - PI
    /// # }
    /// # impl Manifold for Circle {
    /// #     type Scalar = f64;
    /// #     type Dim = U1;
    /// #     fn tangent_dim(&self) -> usize {
    /// #         1
    /// #     }
    /// #     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
    /// #         Self(wrap(self.0 + delta.x))
    /// #     }
    /// #     fn boxminus(&self, x: &Self) -> Vector1<f64> {
    /// #         Vector1::new(wrap(self.0 - x.0))
    /// #     }
    /// # }
    /// use estimate::manifold::axioms;
    ///
    /// axioms::check(&Circle(3.0), &Circle(-3.0), &Vector1::new(0.5), 1e-12)?;
    /// # Ok::<(), estimate::manifold::axioms::AxiomViolation<f64>>(())
    /// ```
    pub fn check<X: Manifold>(x: &X, y: &X, delta: &Tangent<X>, tol: X::Scalar) -> Result<(), AxiomViolation<X::Scalar>>
    where
        DefaultAllocator: Allocator<X::Dim>,
    {
        if let Some(n) = X::Dim::try_to_usize() {
            for s in [x, y] {
                let found = s.tangent_dim();
                if found != n {
                    return Err(AxiomViolation::TangentDim { expected: n, found });
                }
            }
        }
        check_boxplus_boxminus(x, delta, tol.clone())?;
        check_boxminus_boxplus(x, y, tol)
    }

    #[cfg(test)]
    mod tests {
        // Tests run with `std` even when the crate is built `no_std`.
        #[cfg(not(feature = "std"))]
        extern crate std;

        use core::f64::consts::PI;
        use std::string::ToString;

        use nalgebra::{U1, Vector1};
        use proptest::prelude::*;

        use super::*;

        /// The circle $S^1$ as an angle $\theta \in [-\pi, \pi)$.
        ///
        /// ```math
        /// \theta \boxplus \delta = \operatorname{wrap}(\theta + \delta), \qquad
        /// \varphi \boxminus \theta = \operatorname{wrap}(\varphi - \theta)
        /// ```
        #[derive(Clone, Debug)]
        struct Circle(f64);

        /// Wraps an angle into $[-\pi, \pi)$:
        /// $\operatorname{wrap}(a) = ((a + \pi) \bmod 2\pi) - \pi$.
        fn wrap(a: f64) -> f64 {
            (a + PI).rem_euclid(2.0 * PI) - PI
        }

        impl Manifold for Circle {
            type Scalar = f64;
            type Dim = U1;
            fn tangent_dim(&self) -> usize {
                1
            }
            fn boxplus(&self, delta: &Vector1<f64>) -> Self {
                Self(wrap(self.0 + delta.x))
            }
            fn boxminus(&self, x: &Self) -> Vector1<f64> {
                Vector1::new(wrap(self.0 - x.0))
            }
        }

        /// A broken $\boxminus$ that always returns zero: $x \boxplus (y \boxminus x) = x$, so only
        /// $r_1$ catches it.
        #[derive(Clone, Debug)]
        struct ZeroMinus(f64);

        impl Manifold for ZeroMinus {
            type Scalar = f64;
            type Dim = U1;
            fn tangent_dim(&self) -> usize {
                1
            }
            fn boxplus(&self, delta: &Vector1<f64>) -> Self {
                Self(self.0 + delta.x)
            }
            fn boxminus(&self, _x: &Self) -> Vector1<f64> {
                Vector1::zeros()
            }
        }

        /// A $\boxplus$ that overshoots: $x \boxplus \delta = x + 2\delta$ on the line, so
        /// $r_1 = \lVert\delta\rVert$.
        #[derive(Clone, Debug)]
        struct DoublePlus(f64);

        impl Manifold for DoublePlus {
            type Scalar = f64;
            type Dim = U1;
            fn tangent_dim(&self) -> usize {
                1
            }
            fn boxplus(&self, delta: &Vector1<f64>) -> Self {
                Self(self.0 + 2.0 * delta.x)
            }
            fn boxminus(&self, x: &Self) -> Vector1<f64> {
                Vector1::new(self.0 - x.0)
            }
        }

        /// Reports the wrong tangent dimension for a `Const` manifold.
        #[derive(Clone, Debug)]
        struct WrongDim(f64);

        impl Manifold for WrongDim {
            type Scalar = f64;
            type Dim = U1;
            fn tangent_dim(&self) -> usize {
                2
            }
            fn boxplus(&self, delta: &Vector1<f64>) -> Self {
                Self(self.0 + delta.x)
            }
            fn boxminus(&self, x: &Self) -> Vector1<f64> {
                Vector1::new(self.0 - x.0)
            }
        }

        proptest! {
            #[test]
            fn circle_satisfies_axioms(x in -PI..PI, y in -PI..PI, d in -3.0..3.0_f64) {
                check(&Circle(x), &Circle(y), &Vector1::new(d), 1e-9)?;
            }

            #[test]
            fn zero_minus_fails_first_axiom(x in -10.0..10.0_f64, d in 0.1..3.0_f64) {
                let r = check(&ZeroMinus(x), &ZeroMinus(x + 1.0), &Vector1::new(d), 1e-9);
                let is_r1 = matches!(r, Err(AxiomViolation::BoxplusBoxminus { .. }));
                prop_assert!(is_r1, "{:?}", r);
            }

            #[test]
            fn zero_minus_passes_second_axiom(x in -10.0..10.0_f64, y in -10.0..10.0_f64) {
                // y ⊟ x = 0, so x ⊞ (y ⊟ x) = x and (x ⊞ 0) ⊟ y = 0: r₂ alone is fooled.
                check_boxminus_boxplus(&ZeroMinus(x), &ZeroMinus(y), 1e-9)?;
            }

            #[test]
            fn double_plus_fails_first_axiom(x in -10.0..10.0_f64, d in 0.1..3.0_f64) {
                let r = check_boxplus_boxminus(&DoublePlus(x), &Vector1::new(d), 1e-9);
                let is_r1 = matches!(r, Err(AxiomViolation::BoxplusBoxminus { .. }));
                prop_assert!(is_r1, "{:?}", r);
            }
        }

        #[test]
        fn wrong_const_dim_is_reported() {
            let r = check(&WrongDim(0.0), &WrongDim(1.0), &Vector1::new(0.5), 1e-9);
            assert_eq!(r, Err(AxiomViolation::TangentDim { expected: 1, found: 2 }));
        }

        #[test]
        fn nan_residual_is_a_violation() {
            let r = check_boxplus_boxminus(&Circle(0.0), &Vector1::new(f64::NAN), 1e-9);
            assert!(matches!(r, Err(AxiomViolation::BoxplusBoxminus { .. })), "{r:?}");
        }

        #[test]
        fn violation_messages_name_the_axiom() {
            let v = AxiomViolation::BoxplusBoxminus {
                residual: 0.5,
                tol: 1e-9,
            };
            assert!(v.to_string().starts_with("(x ⊞ δ) ⊟ x ≠ δ"));
            let v = AxiomViolation::<f64>::BoxminusBoxplus {
                residual: 0.5,
                tol: 1e-9,
            };
            assert!(v.to_string().starts_with("x ⊞ (y ⊟ x) ≠ y"));
        }

        #[cfg(feature = "alloc")]
        mod dynamic {
            use nalgebra::{DVector, Dyn};

            use super::*;

            /// $\mathbb{R}^n$ with $n$ chosen at runtime, as a `Dyn` manifold.
            #[derive(Clone, Debug)]
            struct Line(DVector<f64>);

            impl Manifold for Line {
                type Scalar = f64;
                type Dim = Dyn;
                fn tangent_dim(&self) -> usize {
                    self.0.len()
                }
                fn boxplus(&self, delta: &DVector<f64>) -> Self {
                    Self(&self.0 + delta)
                }
                fn boxminus(&self, x: &Self) -> DVector<f64> {
                    &self.0 - &x.0
                }
            }

            fn sample(n: usize) -> impl Strategy<Value = DVector<f64>> {
                proptest::collection::vec(-10.0..10.0_f64, n).prop_map(DVector::from_vec)
            }

            proptest! {
                #[test]
                fn dyn_line_satisfies_axioms(
                    (x, y, d) in (1_usize..8).prop_flat_map(|n| (sample(n), sample(n), sample(n)))
                ) {
                    check(&Line(x), &Line(y), &d, 1e-9)?;
                }
            }

            #[test]
            fn wrong_dyn_delta_is_reported_not_panicked() {
                let x = Line(DVector::zeros(3));
                let r = check(&x, &x, &DVector::zeros(2), 1e-9);
                assert_eq!(r, Err(AxiomViolation::TangentDim { expected: 3, found: 2 }));
            }
        }
    }
}

/// $\mathbb{R}^n$ as a manifold: $\boxplus$ is `+`, $\boxminus$ is `-`.
///
/// Implemented directly on `nalgebra::OVector<T, D>`.
pub mod euclidean {}
/// $SO(2)$: planar rotations. Commutative, so left and right perturbations coincide.
pub mod so2 {}
/// $SO(3)$: 3-D rotations, stored as a unit quaternion.
///
/// ```math
/// \begin{aligned}
/// \text{right (local):}\quad & x \boxplus \delta = x \cdot \operatorname{Exp}(\delta),
///   & y \boxminus x &= \operatorname{Log}(x^{-1} y) \\
/// \text{left (global):}\quad & x \boxplus \delta = \operatorname{Exp}(\delta) \cdot x,
///   & y \boxminus x &= \operatorname{Log}(y\, x^{-1})
/// \end{aligned}
/// ```
///
/// Built in-house: `sophus_lie` is not usable `no_std` (it forces `std` on its dependencies) and
/// pins an older nalgebra.
pub mod so3 {}

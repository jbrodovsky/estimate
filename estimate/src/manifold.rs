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
//! - [`euclidean`] — $\mathbb{R}^n$, the trivial case, where $\boxplus$ and $\boxminus$ reduce to
//!   vector addition and subtraction
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
/// ```math
/// x \boxplus \delta = x + \delta, \qquad y \boxminus x = y - x
/// ```
///
/// One blanket impl on `nalgebra::OVector<T, D>` covers `SVector<T, N>` (stack-allocated,
/// `no_std`) and `DVector<T>` (heap-allocated, with the `alloc` feature), for any `T: RealField`
/// (`f32`, `f64`). A vector state needs no wrapper type: the state is its own tangent vector.
///
/// The axioms hold up to floating-point rounding, about one ULP of the operands: $1 + 10^{-17}$
/// rounds to $1$, so $(x \boxplus \delta) \boxminus x$ need not equal $\delta$ exactly.
///
/// # Dimensions
///
/// For `DVector`, a $\delta$ whose length differs from $x$'s makes nalgebra's `+` and `-` panic.
/// Matching lengths is the caller's job (the `Manifold` signature decision in `docs/design.md`);
/// [`axioms::check_tangent_dim`] reports a mismatch as an error in tests.
///
/// # Examples
///
/// ```
/// use estimate::manifold::{Manifold, axioms};
/// use estimate::nalgebra::Vector3;
///
/// let (start, end) = (Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 6.0, 3.0));
/// let delta = end.boxminus(&start);
/// assert_eq!(delta, Vector3::new(3.0, 4.0, 0.0));
/// assert_eq!(start.boxplus(&delta), end);
/// axioms::check(&start, &end, &delta, 1e-12)?;
///
/// # #[cfg(feature = "alloc")]
/// # {
/// // The same impl serves a state sized at runtime.
/// use estimate::nalgebra::DVector;
/// let state = DVector::from_vec(vec![1.0, 2.0]);
/// assert_eq!(state.tangent_dim(), 2);
/// # }
/// # Ok::<(), axioms::AxiomViolation<f64>>(())
/// ```
pub mod euclidean {
    use super::{Allocator, DefaultAllocator, Dim, Manifold, OVector, RealField, Tangent};

    impl<T, D> Manifold for OVector<T, D>
    where
        T: RealField,
        D: Dim,
        DefaultAllocator: Allocator<D>,
    {
        type Scalar = T;
        type Dim = D;
        fn tangent_dim(&self) -> usize {
            self.nrows()
        }
        /// $x \boxplus \delta = x + \delta$.
        fn boxplus(&self, delta: &Tangent<Self>) -> Self {
            self + delta
        }
        /// $y \boxminus x = y - x$.
        fn boxminus(&self, x: &Self) -> Tangent<Self> {
            self - x
        }
    }
    #[cfg(test)]
    mod tests {
        use nalgebra::{RealField, SVector, Scalar, Vector3};
        use proptest::prelude::*;
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        use super::super::{Manifold, axioms};

        /// Elements $\pm 10^e$ with $e \in [-30, 30)$, in scalar type `T`: magnitudes spread over
        /// 60 decades, well inside the `f32` range so sums never overflow.
        fn element<T: RealField>() -> impl Strategy<Value = T> {
            (any::<bool>(), -30.0..30.0_f64).prop_map(|(negative, exponent)| {
                let magnitude = 10.0_f64.powf(exponent);
                nalgebra::convert(if negative { -magnitude } else { magnitude })
            })
        }

        /// A vector in $\mathbb{R}^N$ whose $N$ elements are each drawn from `element`.
        fn svector_variable<S, const N: usize>(element: S) -> impl Strategy<Value = SVector<S::Value, N>>
        where
            S: Strategy,
            S::Value: Scalar,
        {
            prop::array::uniform::<S, N>(element).prop_map(SVector::from)
        }

        /// Axiom tolerance for one sample: a few ULPs of its largest operands.
        ///
        /// Each residual component is two roundings of relative error $\epsilon / 2$ on operands
        /// no larger than
        /// $m_i = \max(\lvert x_i \rvert, \lvert y_i \rvert, \lvert \delta_i \rvert)$:
        ///
        /// ```math
        /// \lvert r_i \rvert \le 2 \epsilon m_i
        /// \quad \Rightarrow \quad
        /// \lVert r \rVert \le 2 \epsilon \lVert m \rVert
        /// ```
        macro_rules! ulp_tolerance {
            ($scalar:ty, $state:expr, $other:expr, $delta:expr) => {{
                let scale = $state.abs().sup(&$other.abs()).sup(&$delta.abs());
                2.0 * <$scalar>::EPSILON * scale.norm()
            }};
        }

        /// One axiom proptest per `name: scalar, N;` row, on `SVector<scalar, N>`, at
        /// [`ulp_tolerance`].
        macro_rules! svector_axioms {
            ($($name:ident: $scalar:ty, $n:literal;)*) => {
                proptest! {
                    $(
                        #[test]
                        fn $name(
                            state in svector_variable::<_, $n>(element::<$scalar>()),
                            other in svector_variable::<_, $n>(element::<$scalar>()),
                            delta in svector_variable::<_, $n>(element::<$scalar>()),
                        ) {
                            let tol = ulp_tolerance!($scalar, state, other, delta);
                            axioms::check(&state, &other, &delta, tol)?;
                        }
                    )*
                }
            };
        }

        svector_axioms! {
            r1_f32: f32, 1;
            r2_f32: f32, 2;
            r3_f32: f32, 3;
            r4_f32: f32, 4;
            r6_f32: f32, 6;
            r9_f32: f32, 9;
            r15_f32: f32, 15;
            r1_f64: f64, 1;
            r2_f64: f64, 2;
            r3_f64: f64, 3;
            r4_f64: f64, 4;
            r6_f64: f64, 6;
            r9_f64: f64, 9;
            r15_f64: f64, 15;
        }

        /// $1 + 10^{-17}$ rounds to $1$, so $(x \boxplus \delta) \boxminus x = 0 \ne \delta$:
        /// the axioms are not exact, and the tolerance cannot be zero.
        #[test]
        fn not_exact() {
            let (state, delta) = (SVector::<f64, 1>::new(1.0), SVector::<f64, 1>::new(1e-17));
            assert!(axioms::check(&state, &state, &delta, 0.0).is_err());
        }

        /// A quarter of [`ulp_tolerance`] fails on some samples, so the residual really is about
        /// one ULP and the bound is doing work.
        ///
        /// One dimension, so the residual norm is a single component, and a fixed seed, so the
        /// count is reproducible.
        #[test]
        fn tolerance_is_not_vacuous() {
            let sample = (
                svector_variable::<_, 1>(element::<f64>()),
                svector_variable::<_, 1>(element::<f64>()),
                svector_variable::<_, 1>(element::<f64>()),
            );
            let mut runner = TestRunner::deterministic();
            let too_tight = (0..10_000)
                .filter_map(|_| sample.new_tree(&mut runner).ok())
                .map(|tree| tree.current())
                .filter(|(state, other, delta)| {
                    let tol: f64 = 0.25 * ulp_tolerance!(f64, state, other, delta);
                    axioms::check(state, other, delta, tol).is_err()
                })
                .count();
            std::println!("{too_tight} of 10000 samples exceed a quarter of the tolerance");
            assert!(too_tight > 0);
        }

        /// A hand-checked $\mathbb{R}^3$ step: $\boxplus$ and $\boxminus$ are `+` and `-`.
        #[test]
        fn boxplus_and_boxminus_are_vector_arithmetic() {
            let (start, end) = (Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 6.0, 3.0));
            assert_eq!(end.boxminus(&start), Vector3::new(3.0, 4.0, 0.0));
            assert_eq!(start.boxplus(&Vector3::new(3.0, 4.0, 0.0)), end);
            assert_eq!(start.tangent_dim(), 3);
        }

        #[cfg(feature = "alloc")]
        mod dynamic {
            use nalgebra::{DVector, Scalar};
            use proptest::prelude::*;

            use super::{axioms, element};

            /// Largest tangent dimension $n$ sampled; $n$ is drawn from $[1, n_{\max}]$.
            const MAX_DIM: usize = 16;

            /// A vector in $\mathbb{R}^n$ whose $n$ elements are each drawn from `element`.
            fn dvector_variable<S>(element: S, n: usize) -> impl Strategy<Value = DVector<S::Value>>
            where
                S: Strategy,
                S::Value: Scalar,
            {
                proptest::collection::vec(element, n).prop_map(DVector::from_vec)
            }

            /// One axiom proptest per `name: scalar;` row, on `DVector<scalar>` with $n$ drawn at
            /// runtime, so $x$, $y$ and $\delta$ share one $n$; at [`ulp_tolerance`].
            macro_rules! dvector_axioms {
                ($($name:ident: $scalar:ty;)*) => {
                    proptest! {
                        $(
                            #[test]
                            fn $name(
                                (state, other, delta) in (1..=MAX_DIM).prop_flat_map(|n| (
                                    dvector_variable(element::<$scalar>(), n),
                                    dvector_variable(element::<$scalar>(), n),
                                    dvector_variable(element::<$scalar>(), n),
                                ))
                            ) {
                                let tol = ulp_tolerance!($scalar, state, other, delta);
                                axioms::check(&state, &other, &delta, tol)?;
                            }
                        )*
                    }
                };
            }

            dvector_axioms! {
                rn_f32: f32;
                rn_f64: f64;
            }

            #[test]
            fn mismatched_delta_is_reported_not_panicked() {
                let state = DVector::<f64>::zeros(3);
                let r = axioms::check(&state, &state, &DVector::zeros(2), 1e-9);
                assert_eq!(r, Err(axioms::AxiomViolation::TangentDim { expected: 3, found: 2 }));
            }
        }
    }
}
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

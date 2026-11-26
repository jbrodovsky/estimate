//! Checks of the ⊞/⊟ axioms for any [`Manifold`], for property-based tests.
//!
//! ```text
//! (x ⊞ δ) ⊟ x = δ        r₁ = ‖(x ⊞ δ) ⊟ x − δ‖
//! x ⊞ (y ⊟ x) = y        r₂ = ‖(x ⊞ (y ⊟ x)) ⊟ y‖
//! ```
//!
//! Equality on 𝓜 is measured through ⊟ itself, so the checks need nothing from the state type
//! beyond the trait. The two axioms together rule out a degenerate ⊟: one that always returns
//! zero passes r₂ but fails r₁.
//!
//! The checks are plain functions with no allocation, so they run under `no_std`. The sampling
//! of (x, y, δ) is the implementor's: draw δ inside the injectivity radius (‖δ‖ < π on
//! rotations), where ⊟ undoes ⊞. [`AxiomViolation`] implements `Error`, so `?` turns it into a
//! proptest failure:
//!
//! ```ignore
//! proptest! {
//!     #[test]
//!     fn my_manifold_axioms(x in my_state(), y in my_state(), delta in small_tangent()) {
//!         estimate::manifold::axioms::check(&x, &y, &delta, 1e-9)?;
//!     }
//! }
//! ```

use core::fmt;

use nalgebra::allocator::Allocator;
use nalgebra::{DefaultAllocator, Dim};

use super::{Manifold, Tangent};

/// An axiom that failed for one sample, with the residual that broke it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AxiomViolation<T> {
    /// A tangent vector had the wrong number of rows.
    TangentDim {
        /// The state's [`Manifold::tangent_dim`].
        expected: usize,
        /// Rows of the tangent vector given to or returned by ⊞/⊟.
        found: usize,
    },
    /// (x ⊞ δ) ⊟ x ≠ δ: r₁ = ‖(x ⊞ δ) ⊟ x − δ‖ exceeded the tolerance.
    BoxplusBoxminus {
        /// r₁
        residual: T,
        /// The tolerance it exceeded.
        tol: T,
    },
    /// x ⊞ (y ⊟ x) ≠ y: r₂ = ‖(x ⊞ (y ⊟ x)) ⊟ y‖ exceeded the tolerance.
    BoxminusBoxplus {
        /// r₂
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

/// Checks that a tangent vector δ has n = `x.tangent_dim()` rows.
///
/// # Errors
///
/// [`AxiomViolation::TangentDim`] if it does not.
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

/// Checks the first axiom for one sample:
///
/// ```text
/// (x ⊞ δ) ⊟ x = δ        r₁ = ‖(x ⊞ δ) ⊟ x − δ‖ ≤ tol
/// ```
///
/// # Errors
///
/// [`AxiomViolation::TangentDim`] if δ or (x ⊞ δ) ⊟ x has the wrong size, and
/// [`AxiomViolation::BoxplusBoxminus`] if r₁ exceeds `tol` or is not a number.
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

/// Checks the second axiom for one sample:
///
/// ```text
/// x ⊞ (y ⊟ x) = y        r₂ = ‖(x ⊞ (y ⊟ x)) ⊟ y‖ ≤ tol
/// ```
///
/// # Errors
///
/// [`AxiomViolation::TangentDim`] if a tangent vector has the wrong size, and
/// [`AxiomViolation::BoxminusBoxplus`] if r₂ exceeds `tol` or is not a number.
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

/// Checks both axioms for one sample (x, y, δ):
///
/// ```text
/// (x ⊞ δ) ⊟ x = δ
/// x ⊞ (y ⊟ x) = y
/// ```
///
/// For `Dim = Const<N>`, also checks that `tangent_dim` reports N.
///
/// # Errors
///
/// The first [`AxiomViolation`] found.
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

    /// The circle S¹ as an angle θ ∈ [−π, π).
    ///
    /// ```text
    /// θ ⊞ δ = wrap(θ + δ)    φ ⊟ θ = wrap(φ − θ)
    /// ```
    #[derive(Clone, Debug)]
    struct Circle(f64);

    /// wrap(a) = ((a + π) mod 2π) − π
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

    /// A broken ⊟ that always returns zero: x ⊞ (y ⊟ x) = x, so only r₁ catches it.
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

    /// A ⊞ that overshoots: x ⊞ δ = x + 2δ on the line, so r₁ = ‖δ‖.
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

        /// ℝⁿ with n chosen at runtime, as a `Dyn` manifold.
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

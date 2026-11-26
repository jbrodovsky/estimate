//! Recursive Bayesian estimation in Rust, written the way the math is written.
//!
//! Filters only ever call the model's functions — $f(x)$, $h(x)$ and the noise models — and never
//! know what problem they are solving. States live on manifolds, so the same filter code handles
//! vectors, rotations and poses. Every filter is generic over [`nalgebra::Dim`]: `Const<N>` gives
//! stack-allocated `no_std` filters, `Dyn` gives heap-backed research filters.
//!
//! Layers, each depending only on the ones above it in this list:
//!
//! - [`manifold`] — geometry: $\boxplus$ and $\boxminus$
//! - [`model`] — what the problem is: capability traits
//! - [`covariance`] — covariance representations for Gaussian filters
//! - [`filter`] — how inference is done: belief, predict, update
//! - [`gaussian`], [`particle`], [`discrete`] — filter families
//! - [`fusion`] — time handling around any filter (delayed / out-of-sequence measurements)
//!
//! [`error`] sits outside the layers: every layer may return a [`FilterError`].
//!
//! # Examples
//!
//! A compass heading lives on the circle $S^1$, not on $\mathbb{R}$: turning 0.5 rad from 3.0 rad
//! wraps past $\pi$. Implementing [`Manifold`](manifold::Manifold) is all a state type needs, and
//! [`axioms::check`](manifold::axioms::check) tests the implementation:
//!
//! ```
//! use core::f64::consts::PI;
//! use estimate::manifold::{Manifold, axioms};
//! use estimate::nalgebra::{U1, Vector1};
//!
//! /// A heading θ ∈ [−π, π).
//! #[derive(Clone, Debug)]
//! struct Heading(f64);
//!
//! /// wrap(a) = ((a + π) mod 2π) − π
//! fn wrap(angle: f64) -> f64 {
//!     (angle + PI).rem_euclid(2.0 * PI) - PI
//! }
//!
//! impl Manifold for Heading {
//!     type Scalar = f64;
//!     type Dim = U1;
//!     fn tangent_dim(&self) -> usize {
//!         1
//!     }
//!     /// θ ⊞ δ = wrap(θ + δ)
//!     fn boxplus(&self, delta: &Vector1<f64>) -> Self {
//!         Self(wrap(self.0 + delta.x))
//!     }
//!     /// φ ⊟ θ = wrap(φ − θ)
//!     fn boxminus(&self, x: &Self) -> Vector1<f64> {
//!         Vector1::new(wrap(self.0 - x.0))
//!     }
//! }
//!
//! let heading = Heading(3.0);
//! let turned = heading.boxplus(&Vector1::new(0.5));
//! assert!((turned.0 - (3.5 - 2.0 * PI)).abs() < 1e-12);
//! axioms::check(&heading, &turned, &Vector1::new(0.5), 1e-12)?;
//! # Ok::<(), axioms::AxiomViolation<f64>>(())
//! ```
// Tests link `std` so they can assert on `to_string()` and friends in every feature
// combination. The library itself stays `no_std`: the non-test lib target and the bare-metal
// CI build check that.
#![cfg_attr(not(any(feature = "std", test)), no_std)]

pub mod covariance;
pub mod discrete;
pub mod error;
pub mod filter;
pub mod fusion;
pub mod gaussian;
pub mod manifold;
pub mod model;
pub mod particle;

/// Re-export of the linear algebra crate that appears in this crate's public API.
pub use nalgebra;

/// Re-export of the error types used in this crate.
pub use error::FilterError;

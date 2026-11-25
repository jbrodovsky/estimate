//! # estimate
//!
//! Recursive Bayesian estimation in Rust, written the way the math is written.
//!
//! Filters only ever call the model's functions — f(x), h(x) and the noise models — and never
//! know what problem they are solving. States live on manifolds, so the same filter code handles
//! vectors, rotations and poses. Every filter is generic over [`nalgebra::Dim`]: `Const<N>` gives
//! stack-allocated `no_std` filters, `Dyn` gives heap-backed research filters.
//!
//! Layers, each depending only on the ones above it in this list:
//!
//! - [`manifold`] — geometry: ⊞/⊟
//! - [`model`] — what the problem is: capability traits
//! - [`covariance`] — covariance representations for Gaussian filters
//! - [`filter`] — how inference is done: belief, predict, update
//! - [`gaussian`], [`particle`], [`discrete`] — filter families
//! - [`fusion`] — time handling around any filter (delayed / out-of-sequence measurements)
#![cfg_attr(not(feature = "std"), no_std)]

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

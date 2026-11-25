//! Errors returned by filters.
//!
//! The core never panics; every failure (non-positive-definite innovation covariance, non-finite
//! values, particle degeneracy, gated measurements) comes back as an error, and a filter that
//! returns an error leaves its belief unchanged.

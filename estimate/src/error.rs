//! Errors returned by filters.
//!
//! The core never panics; every failure (non-positive-definite innovation covariance, non-finite
//! values, particle degeneracy, gated measurements) comes back as a [`FilterError`].
//!
//! # Invariant
//!
//! A filter that returns `Err` leaves its belief unchanged: after a failed `predict` or `update`,
//! the belief is exactly what it was before the call, so the caller can skip the step and carry on.
//!
//! # For implementors
//!
//! Compute every intermediate (x⁻, P⁻, S, K, weights, …) into temporaries and assign to `self`
//! only after the last fallible step. Never write a partial result into the belief.
//!
//! `error` sits outside the layer list in the crate docs: every layer may return it.

/// Why a predict or update step failed.
///
/// A filter that returns `Err` leaves its belief unchanged (see the [module docs](self)).
/// [`Gated`](Self::Gated) is an ordinary, recoverable outcome rather than a fault; the other
/// variants usually indicate a modelling or numerical problem.
///
/// The enum is `#[non_exhaustive]`: new failure modes (for example a manifold mean that fails to
/// converge) are added by the filters that need them, without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FilterError {
    /// A covariance the filter must factor is not positive definite: its Cholesky factorization
    /// failed. Usually the innovation covariance S = H P Hᵀ + R.
    NotPositiveDefinite,
    /// A NaN or ±∞ appeared in the state, covariance, weights or a model output.
    NonFinite,
    /// Every weight is zero (log wⁱ = −∞) or non-finite, so normalization and
    /// N_eff = 1 / Σ (wⁱ)² are undefined. Also covers a histogram whose posterior mass is zero.
    Degenerate,
    /// The measurement failed the validation gate and was rejected.
    Gated,
    /// A `Dyn`-sized vector or matrix does not match the expected dimension.
    DimensionMismatch {
        /// The dimension the filter expected (usually the state's tangent dimension).
        expected: usize,
        /// The dimension it was given.
        found: usize,
    },
}

impl core::fmt::Display for FilterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotPositiveDefinite => write!(f, "covariance is not positive definite"),
            Self::NonFinite => write!(f, "non-finite value encountered"),
            Self::Degenerate => write!(f, "weights are degenerate"),
            Self::Gated => write!(f, "measurement was gated out"),
            Self::DimensionMismatch { expected, found } => {
                write!(f, "dimension mismatch: expected {expected}, found {found}")
            }
        }
    }
}

impl core::error::Error for FilterError {}

#[cfg(test)]
mod tests {
    use super::FilterError;
    use core::error::Error;

    const ALL: [FilterError; 5] = [
        FilterError::NotPositiveDefinite,
        FilterError::NonFinite,
        FilterError::Degenerate,
        FilterError::Gated,
        FilterError::DimensionMismatch { expected: 3, found: 4 },
    ];

    #[test]
    fn display() {
        assert_eq!(
            FilterError::NotPositiveDefinite.to_string(),
            "covariance is not positive definite"
        );
        assert_eq!(FilterError::NonFinite.to_string(), "non-finite value encountered");
        assert_eq!(FilterError::Degenerate.to_string(), "weights are degenerate");
        assert_eq!(FilterError::Gated.to_string(), "measurement was gated out");
        assert_eq!(
            FilterError::DimensionMismatch { expected: 3, found: 4 }.to_string(),
            "dimension mismatch: expected 3, found 4"
        );
    }

    #[test]
    fn messages_are_distinct() {
        for (i, a) in ALL.iter().enumerate() {
            for b in &ALL[i + 1..] {
                assert_ne!(a.to_string(), b.to_string());
            }
        }
    }

    #[test]
    fn is_an_error_without_source() {
        for e in &ALL {
            let e: &dyn Error = e;
            assert!(e.source().is_none());
        }
    }

    /// Filters move errors across threads and into `Box<dyn Error>`; keep it cheap and portable.
    #[test]
    fn is_copy_send_sync_static() {
        const fn assert_bounds<T: Copy + Send + Sync + 'static + Error>() {}
        assert_bounds::<FilterError>();
    }
}

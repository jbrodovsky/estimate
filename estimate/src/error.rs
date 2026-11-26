//! Errors returned by filters.
//!
//! The core never panics; every failure (non-positive-definite innovation covariance, non-finite
//! values, particle degeneracy, gated measurements) comes back as an error, and a filter that
//! returns an error leaves its belief unchanged.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Basic filter errors and health status
///
/// Variants indicate specific failure modes or health status of a filter. Filters should return these
/// errors when they encounter these specific conditions instead of panicking.
pub enum FilterError {
    /// S = H P Hᵀ + R (or any factored covariance): Cholesky failed
    NotPosititveDefinite,
    /// NaN or plus/minus infinity in state, covariance, weights or model output
    NonFinite,
    /// Particle degeneracy: all weights are effectively zero
    Degenerate,
    /// Measurement was gated out
    Gated,
    /// Dimension mismatch between expected and found values
    DimensionMismatch {
        /// Expected dimension
        expected: usize,
        /// Found dimension
        found: usize,
    },
}

impl core::fmt::Display for FilterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FilterError::NotPosititveDefinite => write!(f, "not positive definite"),
            FilterError::NonFinite => write!(f, "non-finite value encountered"),
            FilterError::Degenerate => write!(f, "particle degeneracy"),
            FilterError::Gated => write!(f, "measurement was gated out"),
            FilterError::DimensionMismatch { expected, found } => {
                write!(f, "Dimension mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl core::error::Error for FilterError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let err = FilterError::NotPosititveDefinite;
        assert_eq!(format!("{}", err), "not positive definite");
        let err = FilterError::NonFinite;
        assert_eq!(format!("{}", err), "non-finite value encountered");
        let err = FilterError::Degenerate;
        assert_eq!(format!("{}", err), "particle degeneracy");
        let err = FilterError::Gated;
        assert_eq!(format!("{}", err), "measurement was gated out");
        let err = FilterError::DimensionMismatch { expected: 3, found: 4 };
        assert_eq!(format!("{}", err), "Dimension mismatch: expected 3, found 4");
    }
}

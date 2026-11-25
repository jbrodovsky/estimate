//! Covariance representations for Gaussian filters.
//!
//! Decision: square-root and UD forms are a covariance-representation type parameter, not
//! separate filters. A filter never touches P directly; it asks the representation to propagate
//! (P ← F P Fᵀ + Q), compute a gain, and apply an update. Start with a dense representation.

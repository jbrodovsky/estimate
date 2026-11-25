//! Time handling around any filter.
//!
//! The core framework is discrete-time; time enters only through a model's `Input`. This module
//! is where delayed and out-of-sequence measurements are handled, as wrappers over any filter
//! rather than as part of the filter math. Candidate strategies (see `docs/design.md`):
//!
//! - rollback: a fixed-capacity history of (time, belief, input); on a late measurement, restore
//!   the snapshot before it, update, and re-predict forward. Exact and general.
//! - delayed horizon: run the filter on a fixed delay and propagate an output predictor to the
//!   present (the PX4 EKF2 approach). Cheaper; suits embedded.

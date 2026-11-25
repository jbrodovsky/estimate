//! Sampling filters: bootstrap/SIR particle filter (v0.2), Rao-Blackwellized PF (v0.3).
//!
//! Particle storage is a separate axis from state dimension: `Vec` behind `alloc`, fixed
//! capacity without it.

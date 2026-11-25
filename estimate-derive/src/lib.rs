//! Derive macros for `estimate`.
//!
//! Planned: `#[derive(Manifold)]` for composite (product) states. The user declares the tangent
//! dimension (`#[manifold(dim = 15)]`); the macro emits componentwise ⊞/⊟ and a compile-time
//! assertion that the declared dimension equals the sum of the fields'. Stable Rust lacks
//! `generic_const_exprs`, so the dimension cannot yet be computed from the fields.

//! SO(3): 3-D rotations, stored as a unit quaternion.
//!
//! ```text
//! right (local):  x ⊞ δ = x · Exp(δ)    y ⊟ x = Log(x⁻¹ · y)
//! left (global):  x ⊞ δ = Exp(δ) · x    y ⊟ x = Log(y · x⁻¹)
//! ```
//!
//! Built in-house: `sophus_lie` is not usable `no_std` (it forces `std` on its dependencies) and
//! pins an older nalgebra.

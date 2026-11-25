//! Filters: how inference is done.
//!
//! ```text
//! predict:  p(xₖ | z₁:ₖ₋₁) = ∫ p(xₖ | xₖ₋₁, uₖ) p(xₖ₋₁ | z₁:ₖ₋₁) dxₖ₋₁
//! update:   p(xₖ | z₁:ₖ)   ∝ p(zₖ | xₖ) p(xₖ | z₁:ₖ₋₁)
//! ```
//!
//! Planned traits: `Filter { State, Belief }`, `Predict<P>`, `Update<H>` (generic over the model,
//! static dispatch), and `UpdateReport`, which must expose log p(zₖ | z₁:ₖ₋₁).

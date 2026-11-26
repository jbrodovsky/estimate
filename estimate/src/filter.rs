//! Filters: how inference is done.
//!
//! ```text
//! predict:  p(xₖ | z₁:ₖ₋₁) = ∫ p(xₖ | xₖ₋₁, uₖ) p(xₖ₋₁ | z₁:ₖ₋₁) dxₖ₋₁
//! update:   p(xₖ | z₁:ₖ)   ∝ p(zₖ | xₖ) p(xₖ | z₁:ₖ₋₁)
//! ```
//!
//! Planned traits: `Filter { State, Belief }`, `Predict<P>`, `Update<H>` (generic over the model,
//! static dispatch), and `UpdateReport`, which must expose log p(zₖ | z₁:ₖ₋₁).

pub trait Filter {
    type State;
    type Belief; // (mean, cov) | particles | probability vector
    fn belief(&self) -> &Self::Belief;
}

pub trait Predict<P: ProcessModel<Self::State>>: Filter {
    fn predict(&mut self, model: &P, input: &P::Input) -> Result<(), FilterError>;
}

pub trait Update<H: MeasurementModel<Self::State>>: Filter {
    type Report: UpdateReport;
    fn update(&mut self, model: &H, z: &H::Z) -> Result<Self::Report, FilterError>;
}

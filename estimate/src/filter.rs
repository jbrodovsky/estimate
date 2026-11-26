//! Filters: how inference is done.
//!
//! ```math
//! \begin{aligned}
//! \text{predict:}\quad & p(x_k \mid z_{1:k-1})
//!   = \int p(x_k \mid x_{k-1}, u_k)\, p(x_{k-1} \mid z_{1:k-1})\, dx_{k-1} \\
//! \text{update:}\quad & p(x_k \mid z_{1:k}) \propto p(z_k \mid x_k)\, p(x_k \mid z_{1:k-1})
//! \end{aligned}
//! ```
//!
//! Planned traits: `Filter { State, Belief }`, `Predict<P>`, `Update<H>` (generic over the model,
//! static dispatch), and `UpdateReport`, which must expose $\log p(z_k \mid z_{1:k-1})$.
//!
//! # Examples
//!
//! A filter exposes its belief; what the belief is depends on the filter family:
//!
//! ```
//! use estimate::filter::Filter;
//!
//! /// A scalar Gaussian belief N(mean, variance).
//! struct Normal {
//!     mean: f64,
//!     variance: f64,
//! }
//!
//! struct ScalarFilter {
//!     belief: Normal,
//! }
//!
//! impl Filter for ScalarFilter {
//!     type State = f64;
//!     type Belief = Normal;
//!     fn belief(&self) -> &Normal {
//!         &self.belief
//!     }
//! }
//!
//! let filter = ScalarFilter { belief: Normal { mean: 2.0, variance: 0.5 } };
//! assert_eq!(filter.belief().mean, 2.0);
//! assert_eq!(filter.belief().variance, 0.5);
//! ```

/// A filter: something that holds a belief about a state.
///
/// The base of every filter family. `Predict<P>` and `Update<H>` will build on it (see the
/// [module docs](self)).
///
/// # Examples
///
/// ```
/// use estimate::filter::Filter;
///
/// /// A probability vector over discrete states.
/// struct Histogram {
///     probabilities: [f64; 3],
/// }
///
/// impl Filter for Histogram {
///     type State = usize;
///     type Belief = [f64; 3];
///     fn belief(&self) -> &[f64; 3] {
///         &self.probabilities
///     }
/// }
///
/// let filter = Histogram { probabilities: [0.25, 0.5, 0.25] };
/// assert_eq!(filter.belief().iter().sum::<f64>(), 1.0);
/// ```
pub trait Filter {
    /// The type of the state $x$ being estimated.
    type State;
    /// The filter's belief about the state: (mean, covariance), particles or a probability vector.
    type Belief;
    /// The current belief $p(x_k \mid z_{1:k})$.
    ///
    /// # Returns
    ///
    /// The belief after the most recent predict or update.
    ///
    /// # Examples
    ///
    /// ```
    /// # use estimate::filter::Filter;
    /// # struct Constant(f64);
    /// # impl Filter for Constant {
    /// #     type State = f64;
    /// #     type Belief = f64;
    /// #     fn belief(&self) -> &f64 {
    /// #         &self.0
    /// #     }
    /// # }
    /// let filter = Constant(1.5);
    /// assert_eq!(*filter.belief(), 1.5);
    /// ```
    fn belief(&self) -> &Self::Belief;
}

// sketched out for future use
// pub trait Predict<P: ProcessModel<Self::State>>: Filter {
//     fn predict(&mut self, model: &P, input: &P::Input) -> Result<(), FilterError>;
// }
//
// pub trait Update<H: MeasurementModel<Self::State>>: Filter {
//     type Report: UpdateReport;
//     fn update(&mut self, model: &H, z: &H::Z) -> Result<Self::Report, FilterError>;
// }

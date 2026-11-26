//! Models: what the problem is.
//!
//! Models own $f$, $h$ and the noise; they never see a filter. Base traits carry only types
//! (`ProcessModel { Input }`, `MeasurementModel { Z }`); capabilities layer on top, and each
//! filter's `impl` names only the ones it needs.
//!
//! | Capability             | Provides                                   | Required by         |
//! |------------------------|--------------------------------------------|---------------------|
//! | `GaussianProcess`      | $f(x, u)$, $Q(x, u)$                       | KF, EKF, UKF, IF    |
//! | `SampleProcess`        | $x' \sim p(\cdot \mid x, u)$               | PF, RBPF, EnKF      |
//! | `TransitionDensity`    | $\log p(x' \mid x, u)$                     | Histogram           |
//! | `GaussianMeasurement`  | $h(x)$, $R(x)$                             | KF, EKF, UKF, IF    |
//! | `Likelihood`           | $\log p(z \mid x)$                         | PF, Histogram, RBPF |
//! | `LinearizeProcess`     | $F = \partial f / \partial \delta$         | EKF, IF             |
//! | `LinearizeMeasurement` | $H = \partial h / \partial \delta$         | EKF, IF             |
//!
//! Jacobians are with respect to the tangent perturbation at zero:
//!
//! ```math
//! F = \left.
//!   \frac{\partial \left[ f(x \boxplus \delta, u) \boxminus f(x, u) \right]}{\partial \delta}
//!   \right|_{\delta = 0}
//! ```
//!
//! Decision: the linearization traits default to central finite differences of that definition;
//! overriding the method supplies an analytic Jacobian.

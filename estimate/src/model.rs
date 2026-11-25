//! Models: what the problem is.
//!
//! Models own f, h and the noise; they never see a filter. Base traits carry only types
//! (`ProcessModel { Input }`, `MeasurementModel { Z }`); capabilities layer on top, and each
//! filter's `impl` names only the ones it needs.
//!
//! | Capability             | Provides            | Required by          |
//! |------------------------|---------------------|----------------------|
//! | `GaussianProcess`      | f(x, u), Q(x, u)    | KF, EKF, UKF, IF     |
//! | `SampleProcess`        | x′ ~ p(· \| x, u)   | PF, RBPF, EnKF       |
//! | `TransitionDensity`    | log p(x′ \| x, u)   | Histogram            |
//! | `GaussianMeasurement`  | h(x), R(x)          | KF, EKF, UKF, IF     |
//! | `Likelihood`           | log p(z \| x)       | PF, Histogram, RBPF  |
//! | `LinearizeProcess`     | F = ∂f/∂δ           | EKF, IF              |
//! | `LinearizeMeasurement` | H = ∂h/∂δ           | EKF, IF              |
//!
//! Jacobians are with respect to the tangent perturbation at zero,
//! F = ∂[f(x ⊞ δ, u) ⊟ f(x, u)]/∂δ. Decision: the linearization traits default to central finite
//! differences of that definition; overriding the method supplies an analytic Jacobian.

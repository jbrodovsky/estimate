# Filters

| Filter | Family | Belief | Model capabilities | Milestone |
|---|---|---|---|---|
| Kalman filter (KF) | Gaussian | mean, covariance | `GaussianProcess`, `GaussianMeasurement` (linear) | [v0.1](https://github.com/jbrodovsky/estimate/issues/10) |
| Extended / error-state KF | Gaussian | mean on $\mathcal{M}$, covariance in tangent space | + `LinearizeProcess`, `LinearizeMeasurement` | [v0.1](https://github.com/jbrodovsky/estimate/issues/11) |
| Unscented KF (UKF) | Gaussian | sigma points through $\boxplus$/$\boxminus$ | `GaussianProcess`, `GaussianMeasurement` | [v0.2](https://github.com/jbrodovsky/estimate/issues/14) |
| Information filter (IF/EIF) | Gaussian | $Y = P^{-1}$, $y = P^{-1}x$ | + linearization for EIF | [v0.2](https://github.com/jbrodovsky/estimate/issues/15) |
| Particle filter (SIR) | Sampling | weighted particles | `SampleProcess`, `Likelihood` | [v0.2](https://github.com/jbrodovsky/estimate/issues/16) |
| Histogram / grid filter | Discrete | probability per cell | `TransitionDensity`, `Likelihood` | [v0.2](https://github.com/jbrodovsky/estimate/issues/17) |
| Rollback fusion | Wrapper | any | any | [v0.2](https://github.com/jbrodovsky/estimate/issues/18) |
| Delayed-horizon fusion | Wrapper | any | any | [v0.2](https://github.com/jbrodovsky/estimate/issues/19) |
| Ensemble KF (EnKF) | Gaussian / sampling | ensemble | `SampleProcess`, `GaussianMeasurement` | [v0.3](https://github.com/jbrodovsky/estimate/issues/20) |
| Rao-Blackwellized PF | Sampling + Gaussian | particles, each with a KF | conditionally linear substructure | [v0.3](https://github.com/jbrodovsky/estimate/issues/21) |

Covariance for the Gaussian filters is a type parameter: a dense representation first, with
square-root and UD forms later. See [Design decisions](../design.md).

Each filter has at least one worked [example](../examples/index.md).

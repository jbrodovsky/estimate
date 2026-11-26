# Models

Models own $f$, $h$ and the noise; they never see a filter. Base traits carry only types
(`ProcessModel { Input }`, `MeasurementModel { Z }`). Capabilities layer on top, and each filter's
`impl` names only the ones it needs, so a model works with every filter whose requirements it meets.

| Capability             | Provides                          | Required by          |
|------------------------|-----------------------------------|----------------------|
| `GaussianProcess`      | $f(x, u)$, $Q(x, u)$              | KF, EKF, UKF, IF     |
| `SampleProcess`        | $x' \sim p(\cdot \mid x, u)$      | PF, RBPF, EnKF       |
| `TransitionDensity`    | $\log p(x' \mid x, u)$            | Histogram            |
| `GaussianMeasurement`  | $h(x)$, $R(x)$                    | KF, EKF, UKF, IF     |
| `Likelihood`           | $\log p(z \mid x)$                | PF, Histogram, RBPF  |
| `LinearizeProcess`     | $F = \partial f / \partial\delta$ | EKF, IF              |
| `LinearizeMeasurement` | $H = \partial h / \partial\delta$ | EKF, IF              |

## Jacobians

Jacobians are taken with respect to the tangent perturbation at zero:

$$
F = \left.\frac{\partial \left[ f(x \boxplus \delta, u) \boxminus f(x, u) \right]}{\partial \delta}\right|_{\delta = 0}
$$

The linearization traits default to central finite differences of this definition. Override the
method to supply an analytic Jacobian.

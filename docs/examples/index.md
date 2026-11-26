# Examples

!!! note "Models first"
    Each example page starts with the mathematical model: the system, the state, and what is
    observed. Code and results are added as each example is implemented. Progress is tracked
    under the [`example` label](https://github.com/jbrodovsky/estimate/labels/example).

Every filter gets at least one example, and most get two: one in navigation, where state
estimation is the everyday tool, and one somewhere else, to show that the filters never know
what problem they are solving. Within an example the same model `impl`s are reused across filters,
so swapping the filter is a one-line change. That is the point to demonstrate.

## Coverage

| Filter | [Tracking](tracking.md) | [Navigation](navigation.md) | [Attitude](attitude.md) | [Epidemic](epidemic.md) | [Battery](battery.md) | [Lorenz-96](lorenz96.md) | [Machine health](machine-health.md) |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| KF               | ● | | | | | | |
| EKF / ESKF       | ● | ● | ● | | ● | ● | |
| UKF              | ● | ● | ● | ● | ● | ● | |
| Information      | ● | | | | | | |
| Particle         | | ● | | ● | | ○ | |
| Histogram        | | ● | | | | | ● |
| Rollback fusion  | | ● | | ● | | | |
| Delayed horizon  | | ● | | | | | |
| EnKF             | | | | ● | | ● | |
| RBPF             | ○ | ● | | | | | ● |

● planned  ○ optional

## The examples

[**2-D target tracking**](tracking.md): a nearly-constant-velocity target in $\mathbb{R}^4$, seen
by a position sensor, a range-bearing radar, or three radars from an unknown start. The linear
baseline, and where the consistency checks are introduced.

[**2-D 3-DOF navigation**](navigation.md): a wheeled vehicle with pose in
$\mathbb{R}^2 \times SO(2)$, propagated by odometry and corrected by GNSS, a compass, landmarks or
a range finder. Covers the most filters: EKF, UKF, global and grid localization, delayed GNSS and
an RBPF that learns odometry biases. Also the walkthrough for writing your own `Manifold`.

[**Attitude on SO(3)**](attitude.md): gyro propagation with accelerometer and magnetometer
updates. Left vs. right perturbations, the quaternion double cover, and the observability of yaw.

[**Epidemic (SEIR)**](epidemic.md): an outbreak tracked from noisy, delayed case counts while
learning the transmission rate. Count likelihoods, a positive-reals manifold, model comparison by
marginal likelihood, and ensemble forecasting.

[**Battery state of charge**](battery.md): an equivalent-circuit cell model, run unchanged on the
desktop and on a Cortex-M4F with `f32` and no allocation. Includes the flat-OCV observability
problem and parameter tracking as the cell ages.

[**Lorenz-96**](lorenz96.md): the 40-variable chaotic benchmark from data assimilation. How
filter cost scales with dimension, and an EnKF with fewer members than state dimensions.

[**Machine health**](machine-health.md): a hidden condition inferred from vibration. Exact
discrete Bayes first, then a switching linear trend where the RBPF samples the mode and runs a KF
per particle.

## Notation

Shared by every example page:

| Symbol | Meaning |
|---|---|
| $k$, $\Delta t$ | step index; time between steps |
| $x_k$, $u_k$, $z_k$ | state, input, measurement |
| $f$, $h$ | process and measurement functions |
| $Q$, $R$ | process and measurement noise covariances |
| $F$, $H$ | Jacobians with respect to the tangent perturbation: $F = \partial[f(x \boxplus \delta) \boxminus f(x)]/\partial\delta$ at $\delta = 0$ |
| $\boxplus$, $\boxminus$ | the manifold operators (see [Manifolds](../concepts/manifolds.md)); $+$ and $-$ on $\mathbb{R}^n$ |
| $\tilde u$ | a measured (noisy) quantity |
| $\mathcal{N}(x; \mu, \Sigma)$ | Gaussian density; $\mathcal{N}(\mu, \Sigma)$ as a distribution |

Process and measurement noises are zero-mean, white and mutually independent unless a page says
otherwise.

## Open questions

These are settled as part of the harness issue, [#22](https://github.com/jbrodovsky/estimate/issues/22).

- **Where examples live.** Recommended: a separate, unpublished workspace crate
  (`estimate-examples`) so simulation and plotting dependencies (`rand`, `rand_distr`, a plotting
  or CSV crate) never touch the core crate's dependency tree. The battery example needs its own
  bare-metal crate for the MCU runtime.
- **Plots in the docs.** Either examples write CSV and a small script renders figures for these
  pages, or examples render directly with a Rust plotting crate.
- **Shared harness.** A seeded truth simulator and metrics (RMSE, NEES, NIS, runtime) shared by
  all examples, so filter comparisons are like for like.

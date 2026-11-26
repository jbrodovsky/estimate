# Lorenz-96 data assimilation

The standard chaotic benchmark from geophysical data assimilation: a ring of 40 variables that
behaves like a single latitude circle of an atmospheric quantity, half of them observed. It uses
`Dyn` dimensions, shows how each Gaussian filter's cost scales with the state dimension, and is the
regime the EnKF was built for.

Issues: [#35](https://github.com/jbrodovsky/estimate/issues/35) (UKF, EKF, optional Lorenz-63 PF),
[#38](https://github.com/jbrodovsky/estimate/issues/38) (EnKF)

## System

$$
\frac{dx_i}{dt} = (x_{i+1} - x_{i-2})\, x_{i-1} - x_i + F, \qquad i = 1, \dots, n, \qquad n = 40,\ F = 8,
$$

with cyclic indices ($x_{-1} = x_{n-1}$, $x_0 = x_n$, $x_{n+1} = x_1$). The quadratic term
conserves energy and advects it around the ring, the linear term dissipates, and $F$ forces. At
$F = 8$ the system is chaotic, with an error-doubling time of about $0.4$ time units. One time
unit corresponds to roughly five days of atmospheric error growth (Lorenz and Emanuel, 1998).

## State

$$
x \in \mathbb{R}^n \text{ with } n \text{ chosen at runtime } (\texttt{Dyn}).
$$

## Process model

The flow map over one observation interval $\Delta t$, computed with RK4 substeps of
$h = 0.01$, plus optional additive model error:

$$
x_{k+1} = \Phi_{\Delta t}(x_k) + w_k, \qquad w_k \sim \mathcal{N}(0, q I_n).
$$

The standard twin experiment uses a perfect model, $q = 0$, and relies on covariance inflation to
keep the filters from collapsing. Both settings are run.

**Jacobian.** The EKF needs $M = \partial\Phi_{\Delta t} / \partial x$. With central finite
differences (the library default) that costs $2n$ extra flow evaluations per step. The analytic
alternative integrates the tangent-linear model $\dot M = J(x) M$ alongside the state, where $J$ is
sparse with four non-zero entries per row:

$$
\frac{\partial \dot x_i}{\partial x_{i-2}} = -x_{i-1}, \qquad
\frac{\partial \dot x_i}{\partial x_{i-1}} = x_{i+1} - x_{i-2}, \qquad
\frac{\partial \dot x_i}{\partial x_{i}} = -1, \qquad
\frac{\partial \dot x_i}{\partial x_{i+1}} = x_{i-1}.
$$

Overriding `LinearizeProcess` with it is also a demonstration of supplying an analytic Jacobian.

## What is observed

Every other variable, with independent Gaussian noise, every $\Delta t$:

$$
z_k = H x_k + v_k, \qquad H = \begin{bmatrix} e_1 & e_3 & \cdots & e_{n-1} \end{bmatrix}^\top \in \mathbb{R}^{20 \times 40}, \qquad
v_k \sim \mathcal{N}(0, \sigma_o^2 I_{20}).
$$

The observation is linear. All of the difficulty is in the chaotic dynamics and in the unobserved
half of the state, which is recovered only through the dynamics' coupling.

## Filters and their cost

| Filter | Flow evaluations per step | Covariance storage | Notes |
|---|---|---|---|
| EKF (finite differences) | $1 + 2n = 81$ | $n \times n$ | the tangent-linear model replaces the $2n$ with one extra integration |
| UKF | $2n + 1 = 81$ | $n \times n$ | no Jacobians |
| EnKF | $N = 20$ | none: $n \times N$ anomalies | $N < n$ |

## Variant: EnKF with fewer members than state dimensions (v0.3)

With $N = 20$ members the sample covariance has rank at most $N - 1 = 19 < 40$. Two things keep the
filter working:

- **Multiplicative inflation:** each forecast anomaly is scaled about the ensemble mean before the
  update, $x^{(i)} \leftarrow \bar x + \lambda\,(x^{(i)} - \bar x)$, with $\lambda \approx 1.02$–$1.10$,
  to counter the underestimated spread caused by sampling error and an imperfect model.
- **Enough members to span the unstable directions:** error growth in this system is confined to
  roughly 13–14 unstable and neutral directions. An ensemble larger than that can work without
  localization, and $N = 20$ is chosen to sit above it. Below it, localization is needed. That is
  out of scope for v0.3 and noted as future work.

The example sweeps $N$ and $\lambda$ and shows filter divergence without inflation.

## Variant: Lorenz-63 with a particle filter (optional)

A particle filter does not scale to $n = 40$: the weights degenerate. In three dimensions it is
practical, which makes the contrast worth showing:

$$
\dot x = s(y - x), \qquad \dot y = x(r - z) - y, \qquad \dot z = x y - b z, \qquad s = 10,\ r = 28,\ b = 8/3,
$$

observing $x$ only, with noise, every $0.1$ time units. The PF tracks the switches between the two
lobes of the attractor, which a Gaussian filter can lose.

## Scenario

!!! note "Nominal values"
    The standard twin-experiment settings for this benchmark. Starting points, to be tuned when the
    example is written.

| Quantity | Symbol | Value |
|---|---|---|
| Dimension, forcing | $n, F$ | 40, 8 |
| Observation interval | $\Delta t$ | 0.05 (about 6 hours) |
| RK4 substep | $h$ | 0.01 |
| Observation noise | $\sigma_o$ | 1.0 |
| Spin-up | | 1000 steps from a random state, discarded |
| Initial belief | | truth plus $\mathcal{N}(0, I)$, with covariance from a long free run (climatology) |
| Ensemble size, inflation | $N, \lambda$ | 20; swept over 1.00–1.15 |
| Run length | | 10,000 assimilation cycles, after a 1000-cycle burn-in |

## Metrics

- Time-averaged RMSE over all 40 variables and over the unobserved 20 alone, against $\sigma_o$ and
  against the climatological spread. A working filter sits well below $\sigma_o$.
- Spread-skill ratio: ensemble or covariance spread against actual error, ideally 1
- Wall-clock time per assimilation cycle as $n$ grows (20, 40, 80, 160)

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| Baseline, FD Jacobians | EKF | `GaussianProcess`, `GaussianMeasurement`, `LinearizeProcess` | v0.2 |
| Baseline, analytic tangent-linear model | EKF | `LinearizeProcess` overridden | v0.2 |
| Sigma points | UKF | `GaussianProcess`, `GaussianMeasurement` | v0.2 |
| $N < n$ with inflation | EnKF | `SampleProcess`, `GaussianMeasurement` | v0.3 |
| Lorenz-63 | Particle filter | `SampleProcess`, `Likelihood` | v0.2 (optional) |

## Code

!!! info "To come"
    Written with [#35](https://github.com/jbrodovsky/estimate/issues/35).

## Results

!!! info "To come"

## References

- E. N. Lorenz. "Predictability: a problem partly solved." *Proc. ECMWF Seminar on
  Predictability*, 1996.
- E. N. Lorenz, K. A. Emanuel. "Optimal sites for supplementary weather observations: simulation
  with a small model." *J. Atmospheric Sciences* 55(3), 1998.
- E. N. Lorenz. "Deterministic nonperiodic flow." *J. Atmospheric Sciences* 20(2), 1963.
- G. Evensen. *Data Assimilation: The Ensemble Kalman Filter.* Springer, 2nd ed., 2009.
- M. Bocquet, A. Carrassi. "Four-dimensional ensemble variational data assimilation and the
  unstable subspace." *Tellus A* 69(1), 2017.

# 2-D 3-DOF ground vehicle navigation

A wheeled vehicle drives in the plane. Its pose is propagated from wheel odometry and corrected by
whatever is available: GNSS, a compass, known landmarks, or a range finder in a known map. This one
system demonstrates more filters than any other example. It is also the walkthrough for
**implementing `Manifold` yourself** for a product state, until `#[derive(Manifold)]` exists.

Issues: [#24](https://github.com/jbrodovsky/estimate/issues/24) (EKF),
[#28](https://github.com/jbrodovsky/estimate/issues/28) (UKF, particle filter),
[#29](https://github.com/jbrodovsky/estimate/issues/29) (histogram),
[#30](https://github.com/jbrodovsky/estimate/issues/30) (delayed GNSS),
[#39](https://github.com/jbrodovsky/estimate/issues/39) (RBPF)

## State

$$
x = (p, \theta) \in \mathcal{M} = \mathbb{R}^2 \times SO(2), \qquad \text{tangent space } \mathbb{R}^3.
$$

$\boxplus$ and $\boxminus$ act componentwise:

$$
(p, \theta) \boxplus \begin{bmatrix} \delta_p \\ \delta_\theta \end{bmatrix} = \left(p + \delta_p,\ \theta \boxplus \delta_\theta\right),
\qquad
(p_1, \theta_1) \boxminus (p_2, \theta_2) = \begin{bmatrix} p_1 - p_2 \\ \theta_1 \boxminus \theta_2 \end{bmatrix},
\qquad \theta_1 \boxminus \theta_2 \in (-\pi, \pi].
$$

This is the product manifold, not SE(2): the position perturbation is expressed in the world frame
and is not coupled to the heading. It is the simplest correct choice and the one a user would
write first. SE(2) is planned separately.

## Process model

Odometry supplies the input $u_k = (\tilde v_k, \tilde\omega_k)$, forward speed and yaw rate,
measured with noise $\tilde u_k = u_k + n_k$, where $n_k \sim \mathcal{N}(0, \Sigma_u)$ and
$\Sigma_u = \operatorname{diag}(\sigma_v^2, \sigma_\omega^2)$. The unicycle model over a step
$\Delta t$:

$$
f(x, \tilde u) = \left( p + \tilde v\, \Delta t \begin{bmatrix} \cos\theta \\ \sin\theta \end{bmatrix},\ \ \theta \boxplus \tilde\omega\, \Delta t \right).
$$

Input noise maps into the tangent space through $G = \partial f / \partial u$:

$$
G = \begin{bmatrix} \Delta t \cos\theta & 0 \\ \Delta t \sin\theta & 0 \\ 0 & \Delta t \end{bmatrix},
\qquad
Q = G\, \Sigma_u\, G^\top + Q_0,
$$

where a small additive $Q_0$ covers wheel slip and other unmodeled motion. The tangent-space
Jacobian, $F = \partial [f(x \boxplus \delta, u) \boxminus f(x, u)] / \partial \delta$ at
$\delta = 0$, is

$$
F = \begin{bmatrix} 1 & 0 & -\tilde v\, \Delta t \sin\theta \\ 0 & 1 & \tilde v\, \Delta t \cos\theta \\ 0 & 0 & 1 \end{bmatrix}.
$$

For sampling filters, the same model draws $n_k$ and pushes the noisy input through $f$
(`SampleProcess`).

## What is observed

### GNSS position

$$
z = p + v, \qquad v \sim \mathcal{N}(0, \sigma_g^2 I_2), \qquad H = \begin{bmatrix} I_2 & 0 \end{bmatrix}.
$$

### Compass heading

$$
z = \theta \boxplus v \in SO(2), \qquad v \sim \mathcal{N}(0, \sigma_c^2), \qquad z \boxminus h(x) = z \boxminus \theta.
$$

The innovation is computed with $\boxminus$, so a heading of $179°$ against a measurement of
$-179°$ gives an innovation of $2°$, not $-358°$. The v0.1 example drives a trajectory that
crosses $\pm\pi$ several times and compares with a naive EKF that treats $\theta$ as a real number.

### Landmark range and bearing

Known landmarks $m_j \in \mathbb{R}^2$ with known correspondences. With $\Delta_j = m_j - p$:

$$
z_j = h_j(x) \boxplus v_j, \qquad
v_j \sim \mathcal{N}\!\left(0, \operatorname{diag}(\sigma_r^2, \sigma_b^2)\right),
$$

$$
h_j(x) = \begin{bmatrix} \lVert \Delta_j \rVert \\ \operatorname{atan2}(\Delta_{j,y}, \Delta_{j,x}) \boxminus \theta \end{bmatrix}
\in \mathbb{R} \times SO(2).
$$

### Range finder in a known map

$B$ beams at fixed angles $\alpha_i$ relative to the heading. The expected range
$\hat r_i(x) = \operatorname{raycast}(\text{map}, p, \theta + \alpha_i)$ comes from the occupancy
map. Each beam's likelihood is a mixture of a hit model and uniform clutter, and the beams are
treated as independent:

$$
p(z \mid x) = \prod_{i=1}^{B} \left[ w_{\mathrm{hit}}\, \mathcal{N}\!\left(z_i;\ \hat r_i(x), \sigma_{\mathrm{hit}}^2\right) + \frac{1 - w_{\mathrm{hit}}}{r_{\max}} \right],
\qquad 0 \le z_i \le r_{\max}.
$$

This is a `Likelihood` only. There is no Gaussian form to linearize, which is why it is used with
the particle and histogram filters.

## Variant: global localization (particle filter)

The start is unknown: particles are drawn uniformly over the free space with uniform heading. The
posterior is multimodal in a map with repeated structure, and it collapses as distinguishing
measurements arrive. Partway through, the vehicle is **kidnapped**: moved without the filter
being told. Recovery uses augmented MCL. Short- and long-term averages of the measurement
likelihood, $w_{\mathrm{fast}}$ and $w_{\mathrm{slow}}$, are tracked, and each resampled particle
is replaced by a uniformly drawn pose with probability
$\max\!\left(0,\ 1 - w_{\mathrm{fast}} / w_{\mathrm{slow}}\right)$. The likelihood comes from the
`UpdateReport`'s $\log p(z_k \mid z_{1:k-1})$.

## Variant: grid localization (histogram filter)

The state is discretized into cells, $(x, y)$ at resolution $\Delta_{xy}$ and $\theta$ into
$N_\theta$ bins, with one probability per cell:

$$
\text{predict:}\quad P^-(j) = \sum_i T(j \mid i, u_k)\, P(i), \qquad
\text{update:}\quad P(j) \propto p(z_k \mid j)\, P^-(j).
$$

The transition $T$ is the motion model above, evaluated on the grid as a local stencil
(`TransitionDensity`). The likelihood is the range-finder model evaluated at each cell's center.

The minimal version is the classic 1-D corridor: $N$ cells, doors at known cells, a motion of $u$
cells with probabilities of undershoot or overshoot by one cell, and a door detector with
$p(\text{door} \mid \text{at a door}) > p(\text{door} \mid \text{not at a door})$.

## Variant: delayed GNSS

GNSS fixes are stamped at the time of validity $t_s$ but arrive at $t_a = t_s + \tau$, with
$\tau \sim \mathcal{U}(150, 300)$ ms. At a 10 Hz fix rate the delay spread exceeds the fix period,
so fixes sometimes arrive out of order. Odometry runs at 100 Hz. Three strategies are compared on
the same data:

1. **Apply on arrival:** use $z(t_s)$ as if it were $z(t_a)$. Biased by roughly $v\tau$ along the
   track.
2. **Rollback:** restore the stored belief at $t_s$, apply the update, and re-predict through the
   stored inputs on $(t_s, t_a]$. Exact.
3. **Delayed horizon:** run the filter at $t - \tau_{\max}$, so every fix arrives on time relative
   to it, and propagate an output predictor from the delayed estimate to the present with the
   buffered inputs, plus a complementary correction. Approximate, with fixed memory.

## Variant: map matching with odometry biases (RBPF)

The state is split into a nonlinear part, the pose, and a part that is linear and Gaussian given
the pose: the wheel-speed scale factor $s$ and the gyro bias $b$.

$$
x^n = (p, \theta) \in \mathbb{R}^2 \times SO(2), \qquad x^l = (s, b) \in \mathbb{R}^2.
$$

The dynamics are linear in $x^l$ given $x^n$:

$$
x^n_{k+1} = f^n(x^n_k, \tilde u_k) \boxplus \left( A^n(x^n_k, \tilde u_k)\, x^l_k + w^n_k \right), \qquad
x^l_{k+1} = x^l_k + w^l_k,
$$

$$
f^n(x^n, \tilde u) = (p,\ \theta \boxplus \tilde\omega\Delta t), \qquad
A^n(x^n, \tilde u) = \begin{bmatrix} \tilde v\Delta t\cos\theta & 0 \\ \tilde v\Delta t\sin\theta & 0 \\ 0 & -\Delta t \end{bmatrix}.
$$

Expanding the $\boxplus$: $p_{k+1} = p_k + s_k \tilde v \Delta t\,(\cos\theta_k, \sin\theta_k)$ and
$\theta_{k+1} = \theta_k \boxplus (\tilde\omega - b_k)\Delta t$, plus noise.

The measurement is map matching against a known map (the range finder above, or a terrain or
magnetic-anomaly map) and depends on the pose only: $z_k \sim p(z_k \mid x^n_k)$.

Each particle samples a pose trajectory, and each particle's KF tracks $x^l$. Because $x^l$ never
appears in the measurement, it is learned **only through the sampled pose**. The particle's own
step,

$$
\xi_k = x^n_{k+1} \boxminus f^n(x^n_k, \tilde u_k) = A^n x^l_k + w^n_k,
$$

acts as a linear measurement of $x^l_k$ in that particle's KF. This is the mixed
linear/nonlinear model of the marginalized particle filter (Schön, Gustafsson and Nordlund, 2005).

!!! warning "Design implication for the RBPF"
    This variant needs the RBPF ([#21](https://github.com/jbrodovsky/estimate/issues/21)) to
    support $x^l$ entering the nonlinear state's dynamics, not only the triangular case where
    $x^l$ is observed through $z_k$.

## Scenario

!!! note "Nominal values"
    These are starting points for the simulation and will be tuned when the example is written.

| Quantity | Symbol | Value |
|---|---|---|
| Odometry rate | $1/\Delta t$ | 100 Hz |
| Speed noise | $\sigma_v$ | 0.05 m/s |
| Yaw-rate noise | $\sigma_\omega$ | 0.01 rad/s |
| GNSS noise / rate | $\sigma_g$ | 2 m at 10 Hz |
| Compass noise | $\sigma_c$ | 3° |
| Landmark range / bearing noise | $\sigma_r, \sigma_b$ | 0.3 m, 2° |
| Range finder | $B, r_{\max}, \sigma_{\mathrm{hit}}$ | 36 beams, 10 m, 0.1 m |
| Grid | $\Delta_{xy}, N_\theta$ | 0.25 m, 36 bins |
| GNSS delay | $\tau$ | 150–300 ms |
| True scale factor, gyro bias (RBPF) | $s, b$ | 1.03, 0.5 °/s |

## Metrics

- Position RMSE and heading error $\lvert \hat\theta \boxminus \theta \rvert$
- NEES on the 3-D tangent space
- Global localization: time to converge, and time to recover after the kidnapping
- Delayed fusion: error, runtime per step and memory for each strategy
- RBPF: bias convergence, and estimator variance against a bootstrap PF over the full
  5-dimensional state at equal particle count

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| GNSS + compass | EKF | `GaussianProcess`, `GaussianMeasurement`, linearization | v0.1 |
| Landmarks | UKF | `GaussianProcess`, `GaussianMeasurement` | v0.2 |
| Global localization | Particle filter | `SampleProcess`, `Likelihood` | v0.2 |
| Grid localization | Histogram | `TransitionDensity`, `Likelihood` | v0.2 |
| Delayed GNSS | Rollback, delayed horizon over the EKF | as the EKF | v0.2 |
| Map matching + biases | RBPF | mixed conditionally linear model | v0.3 |

## Code

!!! info "To come"
    Written with [#24](https://github.com/jbrodovsky/estimate/issues/24).

## Results

!!! info "To come"

## References

- S. Thrun, W. Burgard, D. Fox. *Probabilistic Robotics.* MIT Press, 2005. (Beam model, grid and
  Monte Carlo localization, augmented MCL.)
- T. Schön, F. Gustafsson, P.-J. Nordlund. "Marginalized particle filters for mixed
  linear/nonlinear state-space models." *IEEE Trans. Signal Processing* 53(7), 2005.
- C. Hertzberg, R. Wagner, U. Frese, L. Schröder. "Integrating generic sensor fusion algorithms
  with sound state representations through encapsulation of manifolds." *Information Fusion*
  14(1), 2013.

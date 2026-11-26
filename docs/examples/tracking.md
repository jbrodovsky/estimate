# 2-D target tracking

A target moves in the plane at nearly constant velocity, observed by one or more sensors. This is
the linear baseline of the example suite. It is where the consistency checks (NEES, NIS) are
introduced, and the first place the same process model is reused unchanged across filters.

Issues: [#23](https://github.com/jbrodovsky/estimate/issues/23) (KF, EKF),
[#27](https://github.com/jbrodovsky/estimate/issues/27) (UKF, information filter),
[#41](https://github.com/jbrodovsky/estimate/issues/41) (RBPF, stretch)

## State

$$
x = \begin{bmatrix} p_x & p_y & v_x & v_y \end{bmatrix}^\top \in \mathbb{R}^4,
\qquad p = (p_x, p_y), \quad v = (v_x, v_y)
$$

The state is a plain vector, so $\boxplus$ and $\boxminus$ are $+$ and $-$, and the filters use
`Const<4>` dimensions.

## Process model

Continuous white-noise acceleration: the velocity is driven by white noise of power spectral
density $\tilde q$,

$$
\dot p = v, \qquad \dot v = w(t), \qquad \mathbb{E}\!\left[w(t)\, w(\tau)^\top\right] = \tilde q\, I_2\, \delta(t - \tau).
$$

Discretized exactly over a step $\Delta t$:

$$
x_{k+1} = F x_k + w_k, \qquad
F = \begin{bmatrix} I_2 & \Delta t\, I_2 \\ 0 & I_2 \end{bmatrix}, \qquad
Q = \tilde q \begin{bmatrix} \tfrac{\Delta t^3}{3} I_2 & \tfrac{\Delta t^2}{2} I_2 \\[2pt] \tfrac{\Delta t^2}{2} I_2 & \Delta t\, I_2 \end{bmatrix}.
$$

There is no control input. The model provides `GaussianProcess` (and, being linear, its own exact
Jacobian $F$).

## What is observed

### Cartesian position sensor

$$
z_k = H x_k + v_k, \qquad H = \begin{bmatrix} I_2 & 0 \end{bmatrix}, \qquad v_k \sim \mathcal{N}(0, \sigma_p^2 I_2).
$$

Linear and Gaussian: the KF is exact here.

### Range-bearing radar

A radar at a known position $s = (s_x, s_y)$ measures range and bearing to the target. With
$\Delta = p - s$ and $r = \lVert \Delta \rVert$:

$$
z_k = h(x_k) \boxplus v_k, \qquad
h(x) = \begin{bmatrix} r \\ \operatorname{atan2}(\Delta_y, \Delta_x) \end{bmatrix} \in \mathbb{R} \times SO(2), \qquad
v_k \sim \mathcal{N}\!\left(0, \operatorname{diag}(\sigma_r^2, \sigma_b^2)\right).
$$

The bearing lives on SO(2), so the innovation $z \boxminus h(x)$ wraps to $(-\pi, \pi]$ rather than
jumping by $2\pi$ when the target crosses the $\pm\pi$ line behind the radar. The Jacobian is

$$
H = \frac{\partial h}{\partial x} = \begin{bmatrix}
\Delta_x / r & \Delta_y / r & 0 & 0 \\
-\Delta_y / r^2 & \Delta_x / r^2 & 0 & 0
\end{bmatrix}.
$$

The curvature of $h$ grows like $1/r$, so the first-order approximation the EKF relies on gets
worse as the target passes close to the radar. The KF/EKF and UKF comparison is built around that
close pass.

### Three radars, unknown start (information filter)

Radars at $s_1, s_2, s_3$ give independent measurements $z^{(j)}_k = h_j(x_k) \boxplus v^{(j)}_k$.
In information form, with $Y = P^{-1}$ and $y = Y \hat x$, the extended information filter update
is a sum of per-sensor contributions:

$$
Y_k = Y_k^- + \sum_{j} H_j^\top R_j^{-1} H_j, \qquad
y_k = y_k^- + \sum_{j} H_j^\top R_j^{-1} \left[ \left(z^{(j)}_k \boxminus h_j(\bar x)\right) + H_j \bar x \right],
$$

with each $H_j$ evaluated at the linearization point $\bar x$. The filter starts from $Y_0 = 0$:
no information at all about the target, which the covariance form cannot represent. The EIF still
needs a linearization point, so $\bar x_0$ is a coarse guess (e.g. triangulated from the first
bearings) that carries zero weight.

### Maneuvering target (RBPF, stretch)

The target switches between a discrete set of motion modes, $m_k \in \{\mathrm{CV}, \mathrm{CT}(+\omega_0), \mathrm{CT}(-\omega_0)\}$,
following a Markov chain with transition matrix $\Pi$. Given the mode, the motion is linear:
constant velocity uses $F$ above, and a coordinated turn at known rate $\omega$ uses

$$
F_{\mathrm{CT}}(\omega) = \begin{bmatrix}
1 & 0 & \frac{\sin \omega\Delta t}{\omega} & -\frac{1 - \cos \omega\Delta t}{\omega} \\
0 & 1 & \frac{1 - \cos \omega\Delta t}{\omega} & \frac{\sin \omega\Delta t}{\omega} \\
0 & 0 & \cos \omega\Delta t & -\sin \omega\Delta t \\
0 & 0 & \sin \omega\Delta t & \cos \omega\Delta t
\end{bmatrix}.
$$

With the Cartesian sensor, this is a jump Markov linear system. The RBPF samples the mode sequence
$m_{0:k}$ and runs one KF per particle on $x_k$.

## Scenario

!!! note "Nominal values"
    These are starting points for the simulation and will be tuned when the example is written.

| Quantity | Symbol | Value |
|---|---|---|
| Time step | $\Delta t$ | 1 s |
| Acceleration noise PSD | $\tilde q$ | 0.5 m²/s³ |
| Target speed | $\lVert v \rVert$ | 20 m/s |
| Position sensor noise | $\sigma_p$ | 10 m |
| Radar range noise | $\sigma_r$ | 10 m |
| Radar bearing noise | $\sigma_b$ | 1° |
| Closest approach to the radar | | ≈ 100 m, from a start ≈ 5 km away |
| Turn rate (maneuvering variant) | $\omega_0$ | 3°/s |

## Metrics

Over $M$ Monte Carlo runs:

- RMSE of position and velocity
- NEES $\epsilon_k = (x_k - \hat x_k)^\top P_k^{-1} (x_k - \hat x_k)$, averaged over runs, against
  the two-sided 95 % bounds of $\chi^2_{4M}/M$
- NIS $\nu_k^\top S_k^{-1} \nu_k$ against the $\chi^2_{mM}/M$ bounds, where $m$ is the measurement
  dimension

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| Cartesian sensor | KF | `GaussianProcess`, `GaussianMeasurement` | v0.1 |
| Range-bearing radar | EKF | + `LinearizeMeasurement` | v0.1 |
| Range-bearing radar | UKF | `GaussianProcess`, `GaussianMeasurement` | v0.2 |
| Three radars, $Y_0 = 0$ | Extended information filter | + linearization | v0.2 |
| Maneuvering target | RBPF over the mode | conditionally linear given $m_k$ | v0.3 (stretch) |

## Code

!!! info "To come"
    Written with [#23](https://github.com/jbrodovsky/estimate/issues/23).

## Results

!!! info "To come"

## References

- Y. Bar-Shalom, X. R. Li, T. Kirubarajan. *Estimation with Applications to Tracking and
  Navigation.* Wiley, 2001. (Kinematic models, NEES/NIS consistency tests.)

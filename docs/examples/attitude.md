# Attitude estimation (AHRS) on SO(3)

Estimate the 3-D orientation of a rigid body from a gyroscope, corrected by an accelerometer
(which way is down) and a magnetometer (which way is north). The example covers both perturbation
conventions on SO(3), and how the choice changes the Jacobians.

Issues: [#25](https://github.com/jbrodovsky/estimate/issues/25) (EKF),
[#31](https://github.com/jbrodovsky/estimate/issues/31) (UKF)

## State

$$
x = R \in SO(3), \qquad \text{tangent space } \mathbb{R}^3,
$$

stored as a unit quaternion $q$, with $R = R(q)$ rotating body to world. The world frame is
East-North-Up. Both perturbation sides are implemented, as a type parameter:

$$
\begin{aligned}
\text{right (local, body frame):}\quad & R \boxplus \delta = R \operatorname{Exp}(\delta), \\
\text{left (global, world frame):}\quad & R \boxplus \delta = \operatorname{Exp}(\delta) R.
\end{aligned}
$$

$q$ and $-q$ are the same rotation. $\boxminus$ picks the representative with non-negative scalar
part before taking $\operatorname{Log}$, so $\lVert y \boxminus x \rVert \in [0, \pi]$ is always the
shorter way round. The scenario includes a rotation through $\pi$ to exercise this.

## Process model

The gyroscope measures body angular rate with white noise, $\tilde\omega = \omega + n_g$, with
$n_g$ of spectral density $\sigma_g^2$. Over a step $\Delta t$:

$$
R_{k+1} = f(R_k, \tilde\omega_k) = R_k \operatorname{Exp}(\tilde\omega_k \Delta t), \qquad
Q \approx \sigma_g^2\, \Delta t\, I_3.
$$

The Jacobian depends on the perturbation side:

$$
\begin{aligned}
\text{right:}\quad & f(R \operatorname{Exp}(\delta)) = R \operatorname{Exp}(\tilde\omega\Delta t) \operatorname{Exp}\!\left(\operatorname{Exp}(\tilde\omega\Delta t)^\top \delta\right)
&&\Rightarrow\quad F = \operatorname{Exp}(\tilde\omega\Delta t)^\top, \\
\text{left:}\quad & f(\operatorname{Exp}(\delta) R) = \operatorname{Exp}(\delta)\, f(R)
&&\Rightarrow\quad F = I_3.
\end{aligned}
$$

With the left convention, gyro propagation leaves the error untouched: the error is in the world
frame, and the gyro rotates the body. Gyro noise enters in the body frame. Under the left
convention it maps to $R\, Q\, R^\top$, which is the same $Q$ here because $Q$ is isotropic.

A gyro-bias state $b_g \in \mathbb{R}^3$ (state $SO(3) \times \mathbb{R}^3$) is a natural
extension, left out of the first version to keep the example on SO(3) alone.

## What is observed

Both sensors measure a known world-frame vector expressed in the body frame:

$$
h_{\mathbf{v}}(R) = R^\top \mathbf{v}.
$$

- **Accelerometer:** at low dynamics it measures the reaction to gravity, $\mathbf{v} = \mathbf{g}_0 = (0, 0, g)$,
  with noise $\sigma_a$. Linear acceleration is unmodeled. It shows up as large innovations, and
  the NIS is used to gate it.
- **Magnetometer:** the local field direction, $\mathbf{v} = \mathbf{m}_0 = (0, \cos I, -\sin I)$
  for dip angle $I$, normalized, with noise $\sigma_m$.

Linearizing with $\operatorname{Exp}(\delta) \approx I + [\delta]_\times$ and
$-[\delta]_\times \mathbf{a} = [\mathbf{a}]_\times \delta$:

$$
\text{right:}\quad H = \left[ R^\top \mathbf{v} \right]_\times, \qquad
\text{left:}\quad H = R^\top [\mathbf{v}]_\times.
$$

Gravity alone leaves rotation about the vertical (yaw) unobservable: $H$ annihilates
perturbations about the vertical, $\delta \parallel \mathbf{g}_0$ (left) or
$\delta \parallel R^\top \mathbf{g}_0$ (right). The magnetometer restores it. The example shows the yaw variance
growing without bound when the magnetometer is switched off.

## Scenario

!!! note "Nominal values"
    These are starting points for the simulation and will be tuned when the example is written.

| Quantity | Symbol | Value |
|---|---|---|
| IMU rate | $1/\Delta t$ | 100 Hz |
| Gyro noise density | $\sigma_g$ | 0.005 rad/s/√Hz |
| Accelerometer noise | $\sigma_a$ | 0.05 m/s² |
| Magnetometer noise (unit vector) | $\sigma_m$ | 0.02 |
| Magnetic dip | $I$ | 60° |
| Motion | | a slow coning motion, then a tumble that includes a rotation through $\pi$ |
| Initial attitude error | | 30° (EKF), 150° (UKF stress test) |

## Metrics

- Attitude error angle $\lVert \hat R \boxminus R \rVert$ in degrees, reported in the body frame
  for both conventions so they are comparable
- NEES on the 3-D tangent space
- For v0.2: EKF vs. UKF convergence from large initial errors and at high angular rates

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| Right and left perturbation | EKF | `GaussianProcess`, `GaussianMeasurement`, linearization (analytic) | v0.1 |
| Large errors, high rates | UKF | `GaussianProcess`, `GaussianMeasurement` | v0.2 |

## Code

!!! info "To come"
    Written with [#25](https://github.com/jbrodovsky/estimate/issues/25).

## Results

!!! info "To come"

## References

- J. Solà, J. Deray, D. Atchuthan. "A micro Lie theory for state estimation in robotics."
  arXiv:1812.01537, 2018.
- J. Solà. "Quaternion kinematics for the error-state Kalman filter." arXiv:1711.02508, 2017.

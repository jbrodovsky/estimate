# Examples

!!! note "Plan"
    This page is the example roadmap, tracked under the
    [`example` label](https://github.com/jbrodovsky/estimate/labels/example). Each example lands
    with the filters it demonstrates and then gets its own page with the model, the equations,
    results and plots.

Every filter gets at least one example, and most get two: one in navigation, where state
estimation is the everyday tool, and one somewhere else, to show that the filters never know
what problem they are solving. The same model `impl`s are reused across filters within an
example, so swapping the filter is a one-line change. That is the point to demonstrate.

## Coverage

| Filter | 2-D tracking | 2-D navigation | Attitude (SO(3)) | Epidemic | Battery (MCU) | Lorenz-96 | Machine health |
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

● planned  ○ optional comparison

## 2-D target tracking

A target moving at nearly constant velocity, $x = (p_x, p_y, v_x, v_y) \in \mathbb{R}^4$. The
linear baseline, and the place to show consistency checks (NEES/NIS) and `Const<4>` filters.

Issues: [#23](https://github.com/jbrodovsky/estimate/issues/23), [#27](https://github.com/jbrodovsky/estimate/issues/27), [#41](https://github.com/jbrodovsky/estimate/issues/41)

| Variant | Filter | Shows | Milestone |
|---|---|---|---|
| Cartesian position sensor | KF | the linear baseline | v0.1 |
| Range-bearing radar | EKF vs UKF | linearization error close to the sensor; UKF consistency | v0.1 / v0.2 |
| Three radars, unknown start | Information filter | fusion as a sum of information; starting from $Y = 0$ | v0.2 |
| Maneuvering target (CV ↔ coordinated turn) | RBPF over the mode | jump Markov linear system | v0.3 (optional) |

## 2-D 3-DOF ground vehicle navigation

A wheeled vehicle with pose $(p, \theta) \in \mathbb{R}^2 \times SO(2)$, a unicycle model driven by
odometry $u = (v, \omega)$, and a mix of GNSS, compass and landmark range-bearing measurements. It
is also the example of **implementing `Manifold` yourself** for a product state, until
`#[derive(Manifold)]` exists.

Issues: [#24](https://github.com/jbrodovsky/estimate/issues/24), [#28](https://github.com/jbrodovsky/estimate/issues/28), [#29](https://github.com/jbrodovsky/estimate/issues/29), [#30](https://github.com/jbrodovsky/estimate/issues/30), [#39](https://github.com/jbrodovsky/estimate/issues/39)

| Variant | Filter | Shows | Milestone |
|---|---|---|---|
| GNSS + compass | EKF on $\mathbb{R}^2 \times SO(2)$ | heading through ±π just works; a naive angle EKF does not | v0.1 |
| Landmark range-bearing | UKF | sigma points through $\boxplus$ on a user manifold | v0.2 |
| Global localization, unknown start | Particle filter | multimodal posterior; kidnapped-robot recovery | v0.2 |
| Grid (Markov) localization in a known map | Histogram | discretized $(x, y, \theta)$ with a range-finder likelihood; a 1-D corridor as the minimal version | v0.2 |
| GNSS fixes arriving 150–300 ms late | Rollback, delayed horizon | apply-on-arrival vs. rollback vs. delayed horizon: error and cost | v0.2 |
| Map matching + odometry biases | RBPF | sample the pose; a per-particle KF tracks wheel scale factor and gyro bias | v0.3 |

## 3-D attitude (AHRS)

Gyro propagation on SO(3) with accelerometer and magnetometer updates. Shows left vs. right
perturbations and the quaternion double cover. An EKF in v0.1, with a UKF comparison once v0.2
lands.

Issues: [#25](https://github.com/jbrodovsky/estimate/issues/25), [#31](https://github.com/jbrodovsky/estimate/issues/31)

## Epidemic (SEIR) with an unknown transmission rate

A non-navigation problem where the measurement model is non-Gaussian and the data arrive late.

$$
x = (S, E, I, R, \beta), \qquad z_k \sim \operatorname{NegBin}\!\left(\rho \cdot \text{new infections}_k,\ \phi\right)
$$

Compartments are positive, and $\beta$ is a parameter learned online by augmenting the state. A
positive-reals manifold with $x \boxplus \delta = x\, e^{\delta}$ keeps them positive without
clamping: a good small demonstration that manifolds are not only for rotations.

Issues: [#32](https://github.com/jbrodovsky/estimate/issues/32), [#33](https://github.com/jbrodovsky/estimate/issues/33), [#37](https://github.com/jbrodovsky/estimate/issues/37)

| Variant | Filter | Shows | Milestone |
|---|---|---|---|
| Daily case counts | Particle filter | count likelihood at low numbers early on; parameter learning | v0.2 |
| Gaussian approximation | UKF | where a Gaussian belief is fine and where it breaks (small counts) | v0.2 |
| Ensemble forecasting | EnKF (stochastic and EAKF) | the approach used in seasonal-flu forecasting | v0.3 |
| Reports by onset date, arriving days late | Rollback fusion | out-of-sequence measurements in a domain far from robotics | v0.2 |
| SIR vs. SEIR | any, via `UpdateReport` | model comparison from accumulated $\log p(z_k \mid z_{1:k-1})$ | v0.2 |

## Battery state of charge on a microcontroller

A Thevenin equivalent-circuit model: $x = (\text{SoC}, V_{RC})$, input current $I$, and a
terminal-voltage measurement through a nonlinear open-circuit-voltage curve:

$$
V = \operatorname{OCV}(\text{SoC}) - V_{RC} - R_0 I
$$

This is the **notebook-to-microcontroller** demonstration: the same model and filter code runs on
the desktop with plots and on `thumbv7em-none-eabihf` with `f32`, `Const` dimensions and no
allocation.

Issues: [#26](https://github.com/jbrodovsky/estimate/issues/26), [#34](https://github.com/jbrodovsky/estimate/issues/34)

| Variant | Filter | Shows | Milestone |
|---|---|---|---|
| SoC on the MCU | EKF | `no_std`, fixed size, panic-free on a real target | v0.1 |
| Flat OCV region | UKF | the comparison where the curve's slope is near zero | v0.2 |
| Capacity fade, $R_0$ growth | EKF/UKF with augmented state | slow parameter tracking | v0.2 |

## Lorenz-96 data assimilation

The standard chaotic benchmark from geophysical data assimilation: $n = 40$ variables with every
other one observed. Shows `Dyn` dimensions and how each filter scales.

Issues: [#35](https://github.com/jbrodovsky/estimate/issues/35), [#38](https://github.com/jbrodovsky/estimate/issues/38)

| Variant | Filter | Shows | Milestone |
|---|---|---|---|
| $N = 20$ members, $N < n$ | EnKF with inflation | the filter built for this regime | v0.3 |
| $2n + 1 = 81$ sigma points | UKF | cost comparison against the EnKF | v0.2 |
| Finite-difference Jacobians | EKF | baseline | v0.2 |
| Lorenz-63, $n = 3$ | Particle filter | the low-dimensional regime where a PF is practical (optional) | v0.2 |

## Machine health monitoring

A condition-monitoring problem with discrete and hybrid states.

Issues: [#36](https://github.com/jbrodovsky/estimate/issues/36), [#40](https://github.com/jbrodovsky/estimate/issues/40)

| Variant | Filter | Shows | Milestone |
|---|---|---|---|
| Health in {healthy, worn, faulty} from vibration RMS | Histogram (exact discrete Bayes) | HMM filtering, marginals, a maintenance-alert threshold | v0.2 |
| Switching linear dynamics: mode + temperature/vibration trend | RBPF | sample the discrete mode, a KF per particle; fault-onset detection | v0.3 |

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

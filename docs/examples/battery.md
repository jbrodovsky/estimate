# Battery state of charge on a microcontroller

Estimate a lithium-ion cell's state of charge (SoC) from current and terminal voltage, using an
equivalent-circuit model. This is the **notebook-to-microcontroller** demonstration: the same model
and filter code runs on the desktop with plots and metrics, and on a Cortex-M4F
(`thumbv7em-none-eabihf`) with `f32`, `Const` dimensions, no allocation and no panics.

Issues: [#26](https://github.com/jbrodovsky/estimate/issues/26) (EKF on the MCU),
[#34](https://github.com/jbrodovsky/estimate/issues/34) (UKF, parameter tracking)

## System

A first-order Thevenin model: an open-circuit voltage source $\operatorname{OCV}(\text{SoC})$, a series
resistance $R_0$, and one RC pair $(R_1, C_1)$ for the polarization dynamics. The current $I$ is
positive on discharge, $C_{\mathrm{cap}}$ is the capacity in ampere-seconds, and $\eta$ is the coulombic efficiency.

$$
\frac{d\,\text{SoC}}{dt} = -\frac{\eta\, I}{C_{\mathrm{cap}}}, \qquad
\frac{dV_1}{dt} = -\frac{V_1}{R_1 C_1} + \frac{I}{C_1}, \qquad
V = \operatorname{OCV}(\text{SoC}) - V_1 - R_0 I.
$$

## State

$$
x = (\text{SoC}, V_1) \in \mathbb{R}^2.
$$

## Process model

Holding the measured current $\tilde I_k$ constant over the sample period $\Delta t$ (zero-order
hold) discretizes the model exactly:

$$
x_{k+1} = A x_k + B \tilde I_k + w_k, \qquad
A = \begin{bmatrix} 1 & 0 \\ 0 & a \end{bmatrix}, \qquad
B = \begin{bmatrix} -\eta\, \Delta t / C_{\mathrm{cap}} \\ R_1 (1 - a) \end{bmatrix}, \qquad
a = e^{-\Delta t / (R_1 C_1)}.
$$

The process is linear. Current-sensor noise $\sigma_I$ enters through $B$:
$Q_k = \sigma_I^2 B B^\top + Q_0$, where $Q_0$ covers model error. Integrating the SoC alone is
coulomb counting, which drifts with any current bias. The voltage measurement is what corrects
that drift.

## What is observed

The terminal voltage:

$$
z_k = \operatorname{OCV}(\text{SoC}_k) - V_{1,k} - R_0 \tilde I_k + v_k, \qquad v_k \sim \mathcal{N}(0, \sigma_V^2),
\qquad
H = \begin{bmatrix} \operatorname{OCV}'(\text{SoC}_k) & -1 \end{bmatrix}.
$$

The only nonlinearity is the OCV curve. It is tabulated (a monotone piecewise-cubic fit through
measured points) with its derivative available analytically, so the Jacobian is exact. Outside
$[0, 1]$ the curve is extended linearly. The filter treats SoC as a real number, and saturation is
not imposed.

The OCV curve of common chemistries has a **flat region** in mid-SoC where
$\operatorname{OCV}' \approx 0$. There, the voltage barely constrains SoC, the SoC variance grows as
in coulomb counting, and linearization error dominates. That is where the UKF comparison in v0.2
is made.

## Variant: parameter tracking (v0.2)

Capacity fades and resistance grows as the cell ages. Augment the state with $R_0$ and the inverse
capacity $c = 1/C_{\mathrm{cap}}$, both as slow random walks:

$$
x = (\text{SoC}, V_1, R_0, c),
$$

$$
\text{SoC}_{k+1} = \text{SoC}_k - \eta\, \Delta t\, c_k\, \tilde I_k, \qquad
R_{0,k+1} = R_{0,k} + w^{R}_k, \qquad
c_{k+1} = c_k + w^{c}_k.
$$

Using $c$ instead of $C_{\mathrm{cap}}$ keeps the SoC update bilinear rather than rational. $R_0$ is observable
from the instantaneous voltage step when the current changes. $c$ is observable over long
excursions in SoC, where the OCV curve has slope. The simulated cell loses 20 % of its capacity
and gains 50 % in $R_0$ over the run.

## The embedded target

- `no_std`, `f32`, `Const<2>` (or `Const<4>` augmented) dimensions, no allocation
- The model and filter live in one `no_std` module, compiled into both the desktop example and a
  bare-metal binary
- The binary runs under QEMU (an `lm3s6965evb` Cortex-M board with semihosting) so that CI can
  check its estimates against the desktop run on the same input file
- Reported: flash and RAM size, and cycles per filter step

The `f32` desktop run is compared against an `f64` run to quantify the precision loss.

## Scenario

!!! note "Nominal values"
    Representative of a ~3 Ah cylindrical cell. Starting points, to be tuned when the example is
    written.

| Quantity | Symbol | Value |
|---|---|---|
| Sample period | $\Delta t$ | 1 s |
| Capacity | $C_{\mathrm{cap}}$ | 3.0 Ah $= 10{,}800$ A·s |
| Series resistance | $R_0$ | 30 mΩ |
| RC pair | $R_1, C_1$ | 15 mΩ, 2000 F ($\tau = 30$ s) |
| Coulombic efficiency | $\eta$ | 1.0 |
| Current noise, bias | $\sigma_I$ | 10 mA, plus a 20 mA uncompensated bias |
| Voltage noise | $\sigma_V$ | 5 mV |
| Current profile | | HPPC-style pulse discharge, then a dynamic drive-cycle profile |
| Initial SoC | | true 0.90, filter starts at 0.60 with $\sigma_{\text{SoC}} = 0.3$ |

## Metrics

- SoC error and $\pm 3\sigma$ envelope over time, compared with coulomb counting
- Convergence time from the wrong initial SoC
- NEES
- $f32$ vs. $f64$ difference
- On target: flash, RAM, cycles per step

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| SoC, desktop and MCU | EKF | `GaussianProcess`, `GaussianMeasurement`, linearization (analytic) | v0.1 |
| Flat OCV region | UKF | `GaussianProcess`, `GaussianMeasurement` | v0.2 |
| Capacity fade, $R_0$ growth | EKF / UKF on the augmented state | as above | v0.2 |

## Code

!!! info "To come"
    Written with [#26](https://github.com/jbrodovsky/estimate/issues/26).

## Results

!!! info "To come"

## References

- G. L. Plett. "Extended Kalman filtering for battery management systems of LiPB-based HEV
  battery packs. Part 2: Modeling and identification." *Journal of Power Sources* 134(2), 2004.
- G. L. Plett. *Battery Management Systems, Volume II: Equivalent-Circuit Methods.* Artech House,
  2015.

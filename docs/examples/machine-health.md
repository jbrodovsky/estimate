# Machine health monitoring

Infer the hidden condition of a machine from vibration readings, and decide when to call for
maintenance. The first version has a purely discrete state, filtered exactly. The second adds a
continuous trend whose dynamics switch with the hidden condition, the setting the RBPF is built for.

Issues: [#36](https://github.com/jbrodovsky/estimate/issues/36) (histogram),
[#40](https://github.com/jbrodovsky/estimate/issues/40) (RBPF)

## Discrete state (histogram filter)

### State

$$
m_k \in \{\mathrm{H}, \mathrm{W}, \mathrm{F}\} \quad \text{(healthy, worn, faulty)}.
$$

### Process model

One step is one inspection interval. Without intervention a machine only degrades; a repair
(input $u_k$) returns it to healthy. With $\Pi_{ij}(u) = p(m_{k+1} = j \mid m_k = i, u_k = u)$:

$$
\Pi(\text{run}) = \begin{bmatrix}
1 - p_{\mathrm{HW}} - p_{\mathrm{HF}} & p_{\mathrm{HW}} & p_{\mathrm{HF}} \\
0 & 1 - p_{\mathrm{WF}} & p_{\mathrm{WF}} \\
0 & 0 & 1
\end{bmatrix},
\qquad
\Pi(\text{repair}) = \begin{bmatrix} 1 & 0 & 0 \\ 1 & 0 & 0 \\ 1 & 0 & 0 \end{bmatrix}.
$$

$p_{\mathrm{HF}}$ allows sudden failures that skip the worn state. The model provides
`TransitionDensity`, $\log \Pi_{ij}(u)$.

### What is observed

The RMS vibration amplitude $z_k > 0$, log-normal with a condition-dependent level:

$$
\log z_k \mid m_k \sim \mathcal{N}\!\left(\mu_{m_k}, s_{m_k}^2\right), \qquad \mu_{\mathrm{H}} < \mu_{\mathrm{W}} < \mu_{\mathrm{F}}.
$$

The levels overlap, so a single reading is ambiguous and evidence has to accumulate over time.

### Filter

The histogram filter is exact here: the belief is a vector of three probabilities $\alpha_k$:

$$
\alpha_k(j) \propto p(z_k \mid m_k = j) \sum_i \Pi_{ij}(u_k)\, \alpha_{k-1}(i).
$$

It is the same filter as [grid localization](navigation.md#variant-grid-localization-histogram-filter)
with a different `TransitionDensity` and `Likelihood`.

### Decision

Raise an alarm when $\alpha_k(\mathrm{F}) > \tau$. Sweeping $\tau$ traces the trade-off between
detection delay and false-alarm rate. Optionally, a cost-based rule repairs when the expected cost
of running exceeds the cost of repair, $c_{\mathrm{F}}\, \alpha_k(\mathrm{F}) > c_{\mathrm{repair}}$.

## Hybrid state (RBPF, v0.3)

Real degradation is gradual: a worn bearing's vibration level drifts upward before failure. Add a
continuous trend whose dynamics depend on the hidden condition.

### State

$$
m_k \in \{\mathrm{H}, \mathrm{W}, \mathrm{F}\}, \qquad
\xi_k = (\ell_k, d_k) \in \mathbb{R}^2,
$$

where $\ell_k$ is the log vibration level and $d_k$ its drift rate.

### Process model

The mode follows $\Pi$ as above. Given the mode, the trend is linear and Gaussian:

$$
\ell_{k+1} = \ell_k + d_k\, \Delta t + w^{\ell}_k, \qquad
d_{k+1} = a\, d_k + (1 - a)\, \bar d_{m_{k+1}} + w^{d}_k,
$$

that is,

$$
\xi_{k+1} = A\, \xi_k + b(m_{k+1}) + w_k, \qquad
w_k \sim \mathcal{N}\!\left(0, Q(m_{k+1})\right),
$$

$$
A = \begin{bmatrix} 1 & \Delta t \\ 0 & a \end{bmatrix}, \qquad
b(m) = \begin{bmatrix} 0 \\ (1 - a)\, \bar d_m \end{bmatrix},
$$

with mean drift $\bar d_{\mathrm{H}} = 0 < \bar d_{\mathrm{W}} < \bar d_{\mathrm{F}}$. The drift
relaxes toward the current mode's mean at rate $1 - a$.

### What is observed

$$
z_k = \log(\text{RMS vibration}_k) = C \xi_k + v_k, \qquad C = \begin{bmatrix} 1 & 0 \end{bmatrix}, \qquad v_k \sim \mathcal{N}(0, r).
$$

### Filter

This is a jump Markov linear system. The exact posterior is a mixture of $3^k$ Gaussians, one per
mode history, so it has to be approximated. The RBPF samples mode histories and runs one KF per
particle on $\xi$:

$$
\begin{aligned}
m^{(i)}_k &\sim q(\cdot \mid m^{(i)}_{k-1}, z_k), \\
\log w^{(i)}_k &\mathrel{+}= \log \mathcal{N}\!\left(z_k;\ C \hat\xi^{(i)-}_k,\ C P^{(i)-}_k C^\top + r\right) + \log \frac{\Pi_{m_{k-1} m_k}}{q(m_k \mid m_{k-1}, z_k)}.
\end{aligned}
$$

With only three modes, the optimal proposal $q(m_k \mid m_{k-1}, z_k) \propto \Pi_{m_{k-1} m_k}\, p(z_k \mid m_k, \cdot)$
can be computed exactly, by running each particle's KF prediction for all three modes. That is a
cheap variance reduction and worth showing against the prior proposal $q = \Pi$.

The interacting multiple model (IMM) filter is the classic deterministic alternative. It is not in
the library, but it is mentioned for comparison.

## Scenario

!!! note "Nominal values"
    Starting points, to be tuned when the example is written.

| Quantity | Symbol | Value |
|---|---|---|
| Inspection interval | $\Delta t$ | 1 hour |
| Degradation probabilities per hour | $p_{\mathrm{HW}}, p_{\mathrm{WF}}, p_{\mathrm{HF}}$ | $10^{-3}$, $5 \times 10^{-3}$, $10^{-5}$ |
| Log-RMS levels (discrete model) | $\mu_{\mathrm{H}}, \mu_{\mathrm{W}}, \mu_{\mathrm{F}}$ | 0.0, 0.4, 1.0, with $s_m = 0.25$ |
| Drift means (hybrid model) | $\bar d_{\mathrm{H}}, \bar d_{\mathrm{W}}, \bar d_{\mathrm{F}}$ | 0, 0.005, 0.05 per hour |
| Drift relaxation | $a$ | 0.95 |
| Measurement noise | $r$ | 0.05² |
| Run | | several run-to-failure histories with repairs, 5000 hours each |
| Particles | | 200 |

## Metrics

- Detection delay (true onset of F to alarm) against false alarms per 1000 hours, as $\tau$ varies
- Mode classification accuracy and the calibration of $\alpha_k$
- RBPF: detection delay against the discrete-only model, and estimator variance with the optimal
  vs. the prior proposal

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| Discrete condition | Histogram (exact) | `TransitionDensity`, `Likelihood` | v0.2 |
| Condition plus trend | RBPF | conditionally linear given $m_k$ | v0.3 |

## Code

!!! info "To come"
    Written with [#36](https://github.com/jbrodovsky/estimate/issues/36).

## Results

!!! info "To come"

## References

- L. R. Rabiner. "A tutorial on hidden Markov models and selected applications in speech
  recognition." *Proc. IEEE* 77(2), 1989.
- A. Doucet, N. de Freitas, K. Murphy, S. Russell. "Rao-Blackwellised particle filtering for
  dynamic Bayesian networks." *UAI*, 2000.
- A. Doucet, S. Godsill, C. Andrieu. "On sequential Monte Carlo sampling methods for Bayesian
  filtering." *Statistics and Computing* 10(3), 2000.

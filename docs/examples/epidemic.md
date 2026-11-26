# Epidemic (SEIR) with an unknown transmission rate

Track an outbreak in a closed population from noisy, delayed daily case counts, while learning
the transmission rate. Nothing here is navigation, and the measurement model is not Gaussian. The
same filters still apply, which is the point of the example.

Issues: [#32](https://github.com/jbrodovsky/estimate/issues/32) (particle filter, UKF, model comparison),
[#33](https://github.com/jbrodovsky/estimate/issues/33) (delayed reports),
[#37](https://github.com/jbrodovsky/estimate/issues/37) (EnKF)

!!! note "Illustrative, not calibrated"
    The parameters describe a generic respiratory outbreak for demonstration. They are not fitted
    to any real disease or dataset.

## System

A population of size $N$ is split into susceptible, exposed (infected, not yet infectious),
infectious and removed compartments, $N = S + E + I + R$. With transmission rate $\beta$, latent
period $1/\sigma$ and infectious period $1/\gamma$:

$$
\dot S = -\beta \frac{S I}{N}, \qquad
\dot E = \beta \frac{S I}{N} - \sigma E, \qquad
\dot I = \sigma E - \gamma I, \qquad
\dot R = \gamma I,
$$

with the basic reproduction number $\mathcal{R}_0 = \beta / \gamma$. The transmission rate is
unknown and drifts (behavior, seasonality, interventions). It is modeled as a random walk in log
space and estimated by augmenting the state with it.

## Two state representations

The filters need different things, so the model provides two representations of the same system.

### Gaussian and ensemble filters: positive reals

For the UKF and EnKF the state is continuous and strictly positive:

$$
x = (S, E, I, R, \beta) \in \mathbb{R}_{>0}^5, \qquad
x \boxplus \delta = x \odot e^{\delta}, \qquad
y \boxminus x = \log(y \oslash x),
$$

with elementwise operations. A Gaussian in the tangent space is a log-normal belief on the state.
No update can produce a negative compartment, so no clamping is needed. This is a manifold that
has nothing to do with rotations. It is written in the example first, and promoted into the
library if it proves generally useful.

Process: integrate the ODE over one day, $\Delta t = 1$, with RK4, plus a log-space random walk
on $\beta$:

$$
x_{k+1} = \Phi_{\Delta t}(x_k) \boxplus w_k, \qquad
w_k \sim \mathcal{N}\!\left(0, \operatorname{diag}(q_S, q_E, q_I, q_R, \sigma_\beta^2)\right).
$$

The filter does not enforce $S + E + I + R = N$. The dynamics conserve it exactly, updates perturb
it slightly, and its drift is reported as a diagnostic.

### Particle filter: chain binomial

For the particle filter the model is the stochastic, integer-valued version. Over day $k$:

$$
\begin{aligned}
n^{SE}_k &\sim \operatorname{Bin}\!\left(S_k,\ 1 - e^{-\beta_k I_k / N}\right), &
S_{k+1} &= S_k - n^{SE}_k, \\
n^{EI}_k &\sim \operatorname{Bin}\!\left(E_k,\ 1 - e^{-\sigma}\right), &
E_{k+1} &= E_k + n^{SE}_k - n^{EI}_k, \\
n^{IR}_k &\sim \operatorname{Bin}\!\left(I_k,\ 1 - e^{-\gamma}\right), &
I_{k+1} &= I_k + n^{EI}_k - n^{IR}_k, \\
\varepsilon_k &\sim \mathcal{N}(0, \sigma_\beta^2), &
R_{k+1} &= R_k + n^{IR}_k, \\
& & \log\beta_{k+1} &= \log\beta_k + \varepsilon_k.
\end{aligned}
$$

Each particle also carries the day's symptom onsets, $C_k = n^{EI}_k$, which the measurement
depends on. Sampling keeps every count non-negative, so the particle state lives on plain
$\mathbb{R}^6$ (`SampleProcess`).

## What is observed

Reported cases by date of symptom onset. A fraction $\rho$ of onsets is reported, with
overdispersion $\phi$:

$$
z_k \sim \operatorname{NegBin}(\mu_k, \phi), \qquad
\operatorname{Var}(z_k) = \mu_k + \frac{\mu_k^2}{\phi},
$$

$$
p(z_k) = \frac{\Gamma(z_k + \phi)}{\Gamma(\phi)\, z_k!} \left(\frac{\phi}{\phi + \mu_k}\right)^{\phi} \left(\frac{\mu_k}{\phi + \mu_k}\right)^{z_k}.
$$

- **Particle filter:** $\mu_k = \rho\, C_k$, using the exact count likelihood (`Likelihood`).
- **UKF and EnKF:** expected onsets $\mu_k = \rho\, \sigma E_k \Delta t$, moment-matched to a
  Gaussian with state-dependent variance: $h(x) = \mu_k$, $R(x) = \mu_k + \mu_k^2 / \phi$
  (`GaussianMeasurement`). This approximation is poor at small counts, early in the outbreak,
  which is the comparison the example makes.

## Variant: delayed and backfilled reports

Real surveillance data arrive late. A case with onset on day $k$ is reported $d$ days later with
probability $\pi_d$, for $d = 0, \dots, D$. On day $t$, the count for onset day $k$ is incomplete
and is revised upward as late reports arrive.

The delay variant uses a Poisson observation, because Poisson thinning makes the report
increments independent:

$$
\Delta z_{k,d} \sim \operatorname{Poisson}\!\left(\rho\, \pi_d\, C_k\right), \qquad d = 0, \dots, D,
$$

with all increments mutually independent given $C_k$. Each increment is a separate measurement of
the state on day $k$ that arrives on day $k + d$. The rollback wrapper restores the belief at day
$k$, applies $\Delta z_{k,d}$ and re-predicts to the present, which is exact. Negative-binomial
increments are not independent, which is why this variant switches to Poisson.

Baselines: ignoring the incomplete recent days (no nowcast), and a filter fed the complete data in
order (the unattainable best case).

## Variant: model comparison

Each update reports $\log p(z_k \mid z_{1:k-1})$. Summing gives the log marginal likelihood of the
model,

$$
\log p(z_{1:K} \mid \mathcal{M}) = \sum_{k=1}^{K} \log p(z_k \mid z_{1:k-1}, \mathcal{M}),
$$

so running an SIR model (no $E$) and the SEIR model on the same data yields a log Bayes factor,
$\log p(z_{1:K} \mid \mathrm{SEIR}) - \log p(z_{1:K} \mid \mathrm{SIR})$. For the particle filter
this is the standard unbiased likelihood estimator.

## Variant: ensemble forecasting (EnKF, v0.3)

The ensemble lives in the positive-reals tangent space, so anomalies are log-ratios about the
ensemble mean. Both the stochastic EnKF and the EAKF (serial, deterministic, one observation at a
time) are run. After the last observation, the ensemble is propagated without updates to give
forecast intervals for incidence and peak timing, scored against the truth.

## Scenario

!!! note "Nominal values"
    These are starting points for the simulation and will be tuned when the example is written.

| Quantity | Symbol | Value |
|---|---|---|
| Population | $N$ | $10^6$ |
| Latent period | $1/\sigma$ | 5 days |
| Infectious period | $1/\gamma$ | 7 days |
| Initial reproduction number | $\mathcal{R}_0 = \beta_0/\gamma$ | 2.5, dropping to 1.2 on day 40 (an intervention) |
| Log-$\beta$ random walk | $\sigma_\beta$ | 0.05 per day |
| Reporting fraction | $\rho$ | 0.3 |
| Overdispersion | $\phi$ | 10 |
| Reporting delay | $\pi_d$ | discretized gamma, mean 4 days, $D = 14$ |
| Initial state | $E_0, I_0$ | 20, 10 |
| Particles / ensemble | | 2000 particles; 50 members |

## Metrics

- Error in $\beta_k$ (equivalently $\mathcal{R}_t = \beta_k S_k / (\gamma N)$) and in incidence, with
  interval coverage
- Log marginal likelihood for the model comparison
- Nowcast error over the last $D$ days in the delay variant
- Forecast: peak timing and peak height error, and interval coverage

## Variants

| Variant | Filter | Model capabilities | Milestone |
|---|---|---|---|
| Counts, parameter learning | Particle filter | `SampleProcess`, `Likelihood` | v0.2 |
| Gaussian approximation | UKF | `GaussianProcess`, `GaussianMeasurement` | v0.2 |
| SIR vs. SEIR | any, via `UpdateReport` | | v0.2 |
| Delayed reports | Rollback over the particle filter | as the particle filter | v0.2 |
| Forecasting | EnKF, EAKF | `SampleProcess`, `GaussianMeasurement` | v0.3 |

## Code

!!! info "To come"
    Written with [#32](https://github.com/jbrodovsky/estimate/issues/32).

## Results

!!! info "To come"

## References

- W. O. Kermack, A. G. McKendrick. "A contribution to the mathematical theory of epidemics."
  *Proc. R. Soc. Lond. A* 115, 1927.
- J. Shaman, A. Karspeck. "Forecasting seasonal outbreaks of influenza." *PNAS* 109(50), 2012.
- J. L. Anderson. "An ensemble adjustment Kalman filter for data assimilation." *Monthly Weather
  Review* 129(12), 2001.
- A. A. King, D. Nguyen, E. L. Ionides. "Statistical inference for partially observed Markov
  processes via the R package pomp." *Journal of Statistical Software* 69(12), 2016.

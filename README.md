# estimate

Recursive Bayesian estimation in Rust, written the way the math is written.

> **Status:** pre-alpha scaffold. Not published.

## Thesis

**estimate** is a library for recursive Bayesian estimation in Rust, written the way the math is written. Filters only ever call the model's functions — f(x), h(x), and the noise models — and never know what problem they are solving. States live on manifolds, so the same filter code handles vectors, rotations, and poses. One API runs from a desktop research notebook to a microcontroller.

**Positioning:** FilterPy's breadth, with manifold rigor and embedded deployability. No Rust crate currently occupies this niche: existing crates are single-filter (`adskalman`, `kfilter`), abandoned (`filter`, `ukf`, `eskf`), or domain-locked (`nyx-space`).

## Scope and non-goals

Scope is **recursive Bayesian estimation of state-space models**: computing p(xₖ | z₁:ₖ) one step at a time, for any domain — navigation, tracking, robotics, finance, epidemiology.

Non-goals for v1, stated up front to prevent scope creep:

- Smoothers (RTS, fixed-lag) — a separate trait later, not a distortion of the filter traits
- Factor graphs and batch MAP optimization
- Parameter learning (EM, gradient-based)
- MCMC and general probabilistic programming
- Domain-specific models (INS mechanization, sensor models) — these belong in downstream crates

## Design principles

1. **Math-first API.** Code reads like the equations. Models describe the problem; filters implement inference; neither knows the other's internals.
2. **Manifolds are first-class.** ⊞/⊟ is the abstraction the Gaussian and sampling filters are built on. Vector spaces are the trivial case, not the default assumption.
3. **Capability-based models.** A model implements what it can (Gaussian, sampleable, density, linearizable). Each filter requires only what it needs, so one model runs across filter families.
4. **Embedded-capable from day one.** The core is `no_std`, allocation-free, and panic-free; `std` and `alloc` are opt-in features.
5. **Static dispatch.** Generic, monomorphized code; no `dyn Filter`. Deterministic and inlinable.
6. **Correctness is verified, not claimed.** Every filter ships with Monte Carlo consistency tests against analytic cases.

## Architecture

Three layers, each depending only on the one below: **geometry** (manifolds), **models** (what the problem is), and **filters** (how inference is done). Scalar genericity (`T: RealField`) is omitted from the sketches below for brevity.

### Geometry: the manifold layer

A state lives on an n-dimensional manifold; all uncertainty lives in its tangent space ℝⁿ. Two operators define it, satisfying x ⊞ (y ⊟ x) = y and (x ⊞ δ) ⊟ x = δ.

```rust
pub trait Manifold: Clone {
    type TangentDim: Dim;                  // Const<N> or Dyn
    fn oplus(&self, d: &Tangent<Self>) -> Self;   // x ⊞ δ
    fn ominus(&self, other: &Self) -> Tangent<Self>; // x ⊟ y
}
```

- Provided impls: ℝⁿ (trivial), SO(2), SO(3), SE(2), SE(3), and unit quaternions.
- Perturbation side (left/global vs. right/local) is part of the type, e.g. `SO3<Right>`. It changes every Jacobian, so it is never a runtime flag.
- Composite states use `#[derive(Manifold)]` with a declared dimension; the macro emits a compile-time check that it equals the sum of its components.

### Models: capability traits

Models own f, h and the noise; they never see the filter. Base traits carry only types:

```rust
pub trait ProcessModel<X> { type Input; }       // Input = () if none; time goes here
pub trait MeasurementModel<X> { type Z; }
```

Capabilities are layered on top. A filter's impl names only the ones it needs.

| Capability              | Provides                 | Required by            |
| ----------------------- | ------------------------ | ---------------------- |
| `GaussianProcess`     | f(x, u), Q(x, u)         | KF, EKF, UKF, IF, EnKF |
| `SampleProcess`       | x′ \~ p(· \| x, u)     | PF, RBPF               |
| `TransitionDensity`   | log p(x′ \| x, u)       | Histogram              |
| `GaussianMeasurement` | h(x), R(x)               | KF, EKF, UKF, IF, EnKF |
| `Likelihood`          | log p(z \| x)            | PF, Histogram, RBPF    |
| `Linearize`           | ∂/∂δ Jacobians (F, H) | EKF, IF                |

Blanket impls make Gaussian models usable everywhere: every `GaussianProcess` is a `SampleProcess` (sample w in the tangent space, apply ⊞), and every `GaussianMeasurement` is a `Likelihood` (via z ⊟ h(x)). `Linearize` has a default finite-difference impl taken with respect to the tangent perturbation; users override it with analytic Jacobians.

### Filters: belief + predict + update

```rust
pub trait Filter {
    type State;
    type Belief;                    // (mean, cov) | particles | probability vector
    fn belief(&self) -> &Self::Belief;
}

pub trait Predict<P: ProcessModel<Self::State>>: Filter {
    fn predict(&mut self, model: &P, input: &P::Input) -> Result<(), FilterError>;
}

pub trait Update<H: MeasurementModel<Self::State>>: Filter {
    type Report: UpdateReport;
    fn update(&mut self, model: &H, z: &H::Z) -> Result<Self::Report, FilterError>;
}

// Requirements live on the impl, not the trait:
impl<X: Manifold, P: GaussianProcess<X> + Linearize<X>> Predict<P> for Ekf<X> { … }
impl<X: Manifold, P: GaussianProcess<X>>                 Predict<P> for Ukf<X> { … }
impl<X, P: SampleProcess<X>>                             Predict<P> for ParticleFilter<X> { … }
```

- Heterogeneous measurements are natural: one filter implements `Update<H>` for every compatible model, so GNSS, baro and terrain updates are just different `H` types.
- `UpdateReport` must expose `log_marginal_likelihood()` — the one diagnostic every filter can compute, enabling model comparison and parameter estimation downstream. Filter-specific reports add NIS and innovation (Gaussian) or effective sample size (particle).
- Errors (singular S, gated-out measurement, particle degeneracy) return `FilterError`; the core never panics.
- Covariance reset after a manifold update (P ← G P Gᵀ) is explicit and opt-in.

### Crate layout

```
estimate/            core: manifold, model, filter traits + all filters (no_std)
estimate-derive/     #[derive(Manifold)]
estimate-py/         PyO3 bindings (later)
```

## Dimensions and embedded strategy

One codebase serves desktop and embedded by writing every filter against nalgebra's `D: Dim`. `Const<N>` gives stack-allocated, `no_std` filters; `Dyn` gives heap-backed research filters. A desktop-first API was rejected: retrofitting static dimensions onto it would be a rewrite, not a port.

- **State dimension** is static where possible: it is part of the model, and mismatched H/P shapes become compile errors instead of in-flight panics.
- **Collection sizes** (particle count, histogram cells, ensemble size) are a separate axis: `Vec` behind the `alloc` feature, fixed capacity without it.
- **Trait-bound noise stays inside the crate.** Users see type aliases (`Ekf<X>`, `EkfDyn`) and the derive macro.
- **Composite dimensions:** stable Rust lacks `generic_const_exprs`, so `#[derive(Manifold)]` takes a declared dimension and statically asserts it. Migrates cleanly once the feature stabilizes.
- **Linear algebra:** commits to nalgebra. faer is faster for large matrices but dynamic-only; typical state sizes (6–50) favor static nalgebra.

Feature flags:

| Flag      | Default | Enables                                                                |
| --------- | ------- | ---------------------------------------------------------------------- |
| `std`   | on      | Convenience I/O                                                       |
| `alloc` | on      | `Dyn` dimensions, `Vec`-backed particle/histogram/ensemble storage |
| `libm`  | off     | `no_std` math functions                                              |
| `serde` | off     | Serialization of beliefs and reports                                   |

**Enforcement:** CI builds the core with `--no-default-features` for `thumbv7em-none-eabihf` on every PR, and clippy denies `unwrap`, `expect`, `panic` and unchecked indexing in the core. Hot paths are checked with `no-panic`.

## Filter catalog and roadmap

v0.1 proves the trait design on the simplest Gaussian filters; v0.3 proves it composes. The generic Bayes filter is the trait layer itself, not an implementation — for continuous states it requires integrals you can't compute; the histogram filter is its exact discrete realization.

| Filter                       | Release | Belief                   | Model capabilities required                          | Uses manifold                   |
| ---------------------------- | ------- | ------------------------ | ---------------------------------------------------- | ------------------------------- |
| Kalman (KF)                  | v0.1    | mean, cov                | Linear `GaussianProcess` + `GaussianMeasurement` | Vector only                     |
| Extended Kalman (EKF / ESKF) | v0.1    | mean, cov                | Gaussian + `Linearize`                             | Yes                             |
| Unscented Kalman (UKF)       | v0.2    | mean, cov                | Gaussian                                             | Yes                             |
| Information (IF / EIF)       | v0.2    | info vector, info matrix | Gaussian (+ `Linearize` for EIF)                   | Yes                             |
| Particle (bootstrap, SIR)    | v0.2    | weighted particles       | `SampleProcess` + `Likelihood`                   | ⊞ for noise, ⊟ for statistics |
| Histogram / grid             | v0.2    | probability vector       | `TransitionDensity` + `Likelihood`               | No                              |
| Ensemble Kalman (EnKF)       | v0.3    | ensemble                 | `SampleProcess` + `GaussianMeasurement`          | Yes                             |
| Rao-Blackwellized PF (RBPF)  | v0.3    | particles × Gaussians   | Partitioned model                                    | Yes                             |

**ESKF** is not a separate filter: it is the EKF on a manifold, where δ is the error state.

**RBPF is the design's acid test.** It should be pure composition — `Rbpf<ParticleFilter<Xn>, Ekf<Xl>>`, each particle carrying a Gaussian filter over the conditionally linear substate. Prototype it during v0.2 even though it ships in v0.3; if the traits can't express it cleanly, they're wrong.

Candidates after v0.3: IMM, cubature KF, auxiliary and regularized PFs, iterated EKF, invariant EKF.

## Verification strategy

Every filter merges only with statistical evidence that it is consistent, not just unit tests that it runs.

- **Analytic ground truth:** on linear-Gaussian problems, EKF, UKF, IF and EnKF must match the KF to numerical tolerance; the histogram filter must converge to the KF as grid resolution increases.
- **Monte Carlo consistency:** NEES and NIS tests against χ² bounds over N runs, on canonical benchmarks (constant-velocity tracking, pendulum, SO(3) attitude, range-bearing localization).
- **Manifold properties:** property-based tests (`proptest`) for the ⊞/⊟ axioms and Jacobian correctness (analytic vs. finite difference) on every provided manifold.
- **Cross-validation:** reference results from FilterPy on shared benchmarks.
- **Embedded:** the `no_std` build and the panic lints from the previous section.

## Adoption

The code alone won't make it the go-to library; FilterPy owes its position largely to its companion book and examples.

- **Worked examples across domains:** target tracking, robot localization, attitude estimation, a stochastic-volatility finance model, and an SIR epidemic model. Every example uses the same traits, which is the strongest demo of the thesis.
- **Docs that show the math:** each filter's rustdoc states its equations next to the code that implements them.
- **Python bindings (`estimate-py`):** Rust filters driven from Python notebooks. Scope these to the `Dyn` configuration; Python users write models in Python and pay a callback cost, or use built-in Rust models for speed.
- **Licensing:** dual MIT / Apache-2.0, the Rust ecosystem norm.

## Layout

```
estimate/          core crate (no_std): manifold, model, covariance, filter, filter families
estimate-derive/   #[derive(Manifold)] for composite states (planned)
docs/              user guide (MkDocs); docs/design.md records design decisions
mkdocs.yml         docs site config
```

Preview the docs with `uvx --with-requirements docs/requirements.txt mkdocs serve`.

## Features

| Flag      | Default | Enables                                                  |
| --------- | ------- | -------------------------------------------------------- |
| `std`   | on      | `std` support (implies `alloc`)                      |
| `alloc` | on      | `Dyn` dimensions, Vec-backed particle/ensemble storage |
| `libm`  | off     | float math for `no_std` targets                        |

## Roadmap

- **v0.1** — traits, `Manifold` (ℝⁿ, SO(2), SO(3)), KF, EKF/ESKF
- **v0.2** — UKF, information filter, particle filter, histogram filter, delayed fusion
- **v0.3** — EnKF, RBPF

## License

Dual-licensed under MIT or Apache-2.0, at your option.

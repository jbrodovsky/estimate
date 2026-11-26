# Design decisions

The full design doc lives in Claude Docs ("estimate — Design Doc"). This file records the
decisions that shape the code.

## Resolved

1. **Jacobians:** central finite differences of F = ∂[f(x ⊞ δ, u) ⊟ f(x, u)]/∂δ by default, as a
   default trait method; override with analytic Jacobians.
2. **Manifold dimension:** associated types (`type Scalar`, `type Dim`), not trait parameters.
   Keeps filter signatures to `Ekf<X>` instead of `Ekf<T, D, X>`. `Dim` may be `Const<N>` or `Dyn`;
   a `tangent_dim(&self)` method supplies the runtime size for `Dyn`.
3. **Time / delayed fusion:** the filter math stays discrete-time; time enters via the model's
   `Input`. Delayed and out-of-sequence measurements are handled in `fusion` as wrappers over
   any filter. Rollback (snapshot history + re-predict) first; delayed-horizon (EKF2-style) as an
   embedded-friendly alternative. Target v0.2.
4. **Square-root / UD forms:** a covariance-representation type parameter on Gaussian filters.
5. **Lie groups:** built in-house. `sophus_lie` 0.15 is marked `#![no_std]` but enables default
   (std) features on `sophus_autodiff`, nalgebra, rand and snafu, and pins nalgebra 0.33.
6. **Governance:** independent open-source project. Not published yet.
7. **Domain:** not registered yet.
8. **Errors:** one non-generic `FilterError` (`Copy`, `Eq`, `#[non_exhaustive]`, payloads only
   `usize`), so signatures stay `Result<_, FilterError>`. `Gated` carries no test statistic;
   callers who need it gate themselves using the `UpdateReport`. New variants (UKF mean not
   converging, measurement older than the rollback horizon) land with the issue that returns them.

## Constraints

- `no_std`, allocation-free, panic-free core. Enforced by clippy (`unwrap_used`, `expect_used`,
  `panic`, `indexing_slicing` denied) and a bare-metal CI build.
- Static dispatch only; no `dyn Filter`.
- nalgebra is public API and pinned to one minor version.

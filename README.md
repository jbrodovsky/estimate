# estimate

Recursive Bayesian estimation in Rust, written the way the math is written.

Filters only ever call the model's functions — f(x), h(x) and the noise models — and never know
what problem they are solving. States live on manifolds (⊞/⊟), so the same filter code handles
vectors, rotations and poses. One API runs from a desktop research notebook to a microcontroller.

> **Status:** pre-alpha scaffold. Not published.

## Layout

```
estimate/          core crate (no_std): manifold, model, covariance, filter, filter families
estimate-derive/   #[derive(Manifold)] for composite states (planned)
docs/              user guide (MkDocs); docs/design.md records design decisions
mkdocs.yml         docs site config
```

Preview the docs with `uvx --with-requirements docs/requirements.txt mkdocs serve`.

## Features

| Flag    | Default | Enables                                              |
|---------|---------|------------------------------------------------------|
| `std`   | on      | `std` support (implies `alloc`)                      |
| `alloc` | on      | `Dyn` dimensions, Vec-backed particle/ensemble storage |
| `libm`  | off     | float math for `no_std` targets                      |

## Roadmap

- **v0.1** — traits, `Manifold` (ℝⁿ, SO(2), SO(3)), KF, EKF/ESKF
- **v0.2** — UKF, information filter, particle filter, histogram filter, delayed fusion
- **v0.3** — EnKF, RBPF

## License

Dual-licensed under MIT or Apache-2.0, at your option.

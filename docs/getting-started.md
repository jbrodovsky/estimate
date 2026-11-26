# Getting started

!!! warning "Not published yet"
    Until the first release, depend on the repository directly.

```toml
[dependencies]
estimate = { git = "https://github.com/jbrodovsky/estimate" }
```

## Features

| Flag    | Default | Enables |
|---------|---------|---------|
| `std`   | on      | `std` support (implies `alloc`) |
| `alloc` | on      | `Dyn` dimensions, Vec-backed particle/ensemble storage |
| `libm`  | off     | float math for `no_std` targets |

For a microcontroller, turn off the defaults and use fixed (`Const<N>`) dimensions:

```toml
estimate = { git = "https://github.com/jbrodovsky/estimate", default-features = false, features = ["libm"] }
```

## The shape of a program

!!! note "Target API"
    This is the intended shape (issues #7, #9 and #10), not code that compiles today.

```rust
// 1. Describe the problem: models own f, h and the noise, and never see a filter.
struct ConstantVelocity { dt: f64, q: f64 }
impl GaussianProcess for ConstantVelocity { /* f(x, u), Q(x, u) */ }

struct PositionSensor { r: f64 }
impl GaussianMeasurement for PositionSensor { /* h(x), R(x) */ }

// 2. Pick a filter. Its impl names only the capabilities it needs.
let mut kf = Kf::new(x0, p0);

// 3. Run the recursion. Errors never panic and leave the belief unchanged.
kf.predict(&motion, &u)?;
let report = kf.update(&sensor, &z)?;
let log_likelihood = report.log_likelihood(); // log p(z_k | z_{1:k-1})
```

## Building the docs

```bash
uvx --with-requirements docs/requirements.txt mkdocs serve
```

API reference: `cargo doc --workspace --no-deps --open`.

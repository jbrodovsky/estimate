# estimate

Recursive Bayesian estimation in Rust, written the way the math is written.

Filters only ever call the model's functions: $f(x)$, $h(x)$ and the noise models. They never know
what problem they are solving. States live on manifolds ($\boxplus$ / $\boxminus$), so the same
filter code handles vectors, rotations and poses. One API runs from a desktop research notebook to
a microcontroller.

!!! warning "Status: pre-alpha"
    The crate is a scaffold and is not published. These pages describe the design and the
    roadmap; APIs shown here are the target shape and may change.

## The two equations

Every filter in the crate is an approximation of the same recursion:

$$
\begin{aligned}
\text{predict:}\quad & p(x_k \mid z_{1:k-1}) = \int p(x_k \mid x_{k-1}, u_k)\, p(x_{k-1} \mid z_{1:k-1})\, dx_{k-1} \\
\text{update:}\quad  & p(x_k \mid z_{1:k}) \propto p(z_k \mid x_k)\, p(x_k \mid z_{1:k-1})
\end{aligned}
$$

The filter families differ only in how they represent the belief: a Gaussian on a manifold, a set
of weighted particles, an ensemble, or a histogram over a grid.

## Layers

Each layer depends only on the ones above it:

| Layer | Role |
|---|---|
| [Manifolds](concepts/manifolds.md) | Geometry: $\boxplus$ / $\boxminus$ |
| [Models](concepts/models.md) | What the problem is: $f$, $h$, noise, as capability traits |
| Covariance | Covariance representations for Gaussian filters |
| [Filters](concepts/filters.md) | How inference is done: belief, predict, update |
| Families | [Gaussian, particle, discrete](filters/index.md) |
| Fusion | Delayed and out-of-sequence measurements, around any filter |

## Roadmap

| Milestone | Scope |
|---|---|
| [v0.1](https://github.com/jbrodovsky/estimate/milestone/1) | Traits, `Manifold` (ℝⁿ, SO(2), SO(3)), KF, EKF/ESKF, CI |
| [v0.2](https://github.com/jbrodovsky/estimate/milestone/2) | UKF, information filter, particle filter, histogram filter, delayed fusion |
| [v0.3](https://github.com/jbrodovsky/estimate/milestone/3) | EnKF, RBPF |

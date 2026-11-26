# Manifolds

A state lives on an $n$-dimensional manifold $\mathcal{M}$; all of its uncertainty lives in the
tangent space $\mathbb{R}^n$. Two operators define the structure (Hertzberg et al., 2013):

$$
\begin{aligned}
\boxplus &: \mathcal{M} \times \mathbb{R}^n \to \mathcal{M}, & x \boxplus (y \boxminus x) &= y \\
\boxminus &: \mathcal{M} \times \mathcal{M} \to \mathbb{R}^n, & (x \boxplus \delta) \boxminus x &= \delta
\end{aligned}
$$

A filter written in terms of $\boxplus$ and $\boxminus$ works on any manifold. On $\mathbb{R}^n$
they are just $+$ and $-$.

## Dimensions

`Manifold` has associated types `Scalar` and `Dim`, so filter signatures read `Ekf<X>` rather
than `Ekf<T, D, X>`. `Dim` is either:

- `Const<N>`: stack-allocated, `no_std`, no allocation
- `Dyn`: heap-backed (`alloc`), with the size supplied at runtime by `tangent_dim(&self)`

## Provided manifolds

| Manifold | Representation | Milestone |
|---|---|---|
| $\mathbb{R}^n$ | `nalgebra::OVector<T, D>` | v0.1 |
| SO(2) | planar rotation; left and right perturbations coincide | v0.1 |
| SO(3) | unit quaternion; perturbation side is a type parameter | v0.1 |
| SE(2), SE(3), SE₂(3) | | later |
| Products | `#[derive(Manifold)]` | later |

For SO(3), the two perturbation conventions are:

$$
\begin{aligned}
\text{right (local):}\quad & x \boxplus \delta = x \cdot \operatorname{Exp}(\delta), & y \boxminus x &= \operatorname{Log}(x^{-1} y) \\
\text{left (global):}\quad & x \boxplus \delta = \operatorname{Exp}(\delta) \cdot x, & y \boxminus x &= \operatorname{Log}(y\, x^{-1})
\end{aligned}
$$

## Your own states

Any type can implement `Manifold`. Until `#[derive(Manifold)]` lands, a composite state such as a
planar pose $(p, \theta) \in \mathbb{R}^2 \times SO(2)$ implements $\boxplus$ and $\boxminus$
componentwise by hand. The [2-D navigation example](../examples/navigation.md)
does exactly this.

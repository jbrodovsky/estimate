# Filters

A filter holds a belief and offers two operations, each generic over the model (static dispatch,
no `dyn Filter`):

- `Predict<P>`: push the belief through a process model
- `Update<H>`: condition the belief on a measurement

## Errors

The core never panics. Every failure (an innovation covariance that is not positive definite,
non-finite values, particle degeneracy, a gated measurement) comes back as a `FilterError`, and a
filter that returns an error **leaves its belief unchanged**.

## Update reports

Every update returns an `UpdateReport` exposing the measurement log-likelihood

$$
\log p(z_k \mid z_{1:k-1}),
$$

which is what you need for gating outliers, tuning noise and comparing models.

## Time

The filter math is discrete-time; time enters only through a model's `Input`. Delayed and
out-of-sequence measurements are handled by wrappers in `fusion` that work around any filter:

- **Rollback:** a fixed-capacity history of (time, belief, input). On a late measurement, restore
  the snapshot before it, update, and re-predict forward. Exact and general.
- **Delayed horizon:** run the filter a fixed delay behind real time and propagate an output
  predictor to the present (the PX4 EKF2 approach). Cheaper; suits embedded targets.

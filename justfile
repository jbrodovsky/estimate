# Local mirror of .github/workflows/ci.yml. `just ci` runs every CI job; keep the two in sync.
#
# The toolchain comes from rust-toolchain.toml, exactly as in CI, so lints match.

# Same flags as the CI `env:` block: any rustc, clippy or rustdoc warning fails.
export RUSTFLAGS := "-D warnings"
export RUSTDOCFLAGS := "-D warnings"

# List recipes.
default:
    @just --list

# Every CI job, in order.
ci: fmt clippy test rustdoc user-guide bare-metal

# rustfmt: check formatting.
fmt:
    cargo fmt --all --check

# Apply formatting (not a CI job; the fix for a failing `fmt`).
fmt-fix:
    cargo fmt --all

# clippy: the full feature matrix.
clippy: clippy-default clippy-no-std clippy-no-std-alloc clippy-no-std-libm

# clippy (default)
clippy-default:
    cargo clippy --workspace --all-targets

# clippy (no_std)
clippy-no-std:
    cargo clippy --workspace --all-targets --no-default-features

# clippy (no_std + alloc)
clippy-no-std-alloc:
    cargo clippy --workspace --all-targets --no-default-features --features alloc

# clippy (no_std + libm)
clippy-no-std-libm:
    cargo clippy --workspace --all-targets --no-default-features --features libm

# test
test:
    cargo test --workspace

# rustdoc
rustdoc:
    cargo doc --workspace --no-deps

# user guide (mkdocs), strict build.
user-guide:
    uvx --with-requirements docs/requirements.txt mkdocs build --strict

# bare metal (thumbv7em-none-eabihf): any std use in the core fails to link.
bare-metal:
    cargo build -p estimate --no-default-features --features libm --target thumbv7em-none-eabihf

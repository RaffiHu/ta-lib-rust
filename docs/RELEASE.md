# Release Checklist

## Packaging

- verify crate metadata in `crates/ta-lib-core/Cargo.toml`
- run `cargo package -p ta-lib-codegen`
- run `cargo package -p ta-lib`

## Verification

- run `cargo fmt --all --check`
- run `cargo clippy --workspace --all-targets`
- run `cargo test --workspace`

## Documentation

- confirm `README.md` reflects the current public API and parity status
- confirm `docs/PARITY.md` still matches the test baseline
- confirm examples compile through doctests or integration tests

## Release Cut

- tag the release against the upstream baseline documented in `README.md`
- include release notes that describe:
  - upstream baseline
  - public API shape
  - parity guarantees
  - known non-goals or limitations

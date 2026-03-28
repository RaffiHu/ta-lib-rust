# ta-lib-rust

`ta-lib-rust` is a Rust port of the TA-Lib core technical analysis library.

The current workspace contains:

- `crates/ta-lib-core/`: the publishable `ta-lib` crate
- `crates/ta-lib-codegen/`: metadata parser and build-time generator
- `upstream-ta-lib-c/`: checked-in upstream TA-Lib C reference used for parity testing

Upstream baseline:

- version: `0.6.4`
- commit: `1bdf5438`
- describe: `v0.6.4-97-g1bdf5438`

## Status

- all 161 upstream TA-Lib functions are implemented
- `Context` is the primary Rust API
- `Core` is the low-level global compatibility facade
- the workspace includes generated full-surface C-oracle parity tests for:
  - default cases
  - lookback-boundary cases
  - seeded deterministic cases
  - parameter-variant cases

Parity details are documented in [docs/PARITY.md](./docs/PARITY.md).

## API

The crate exposes two layers:

- `Context`: explicit state for unstable periods, compatibility mode, and candle settings
- `Core`: TA-Lib-style process-global facade using `initialize()` / `shutdown()`

Example with `Context`:

```rust
use ta_lib::Context;

let context = Context::new();
let input = [1.0, 2.0, 3.0, 4.0, 5.0];
let mut out = [0.0; 5];
let mut out_beg_idx = 0usize;
let mut out_nb_element = 0usize;

let ret = context.sma(0, 4, &input, 3, &mut out_beg_idx, &mut out_nb_element, &mut out);

assert_eq!(ret as i32, 0);
assert_eq!(out_beg_idx, 2);
assert_eq!(out_nb_element, 3);
assert_eq!(&out[..out_nb_element], &[2.0, 3.0, 4.0]);
```

Example with the compatibility facade:

```rust
use ta_lib::{Core, RetCode, initialize, shutdown};

assert_eq!(initialize(), RetCode::Success);

let lhs = [1.0, 2.0, 3.0];
let rhs = [4.0, 5.0, 6.0];
let mut out = [0.0; 3];
let mut out_beg_idx = 0usize;
let mut out_nb_element = 0usize;

let ret = Core::add(0, 2, &lhs, &rhs, &mut out_beg_idx, &mut out_nb_element, &mut out);

assert_eq!(ret, RetCode::Success);
assert_eq!(&out[..out_nb_element], &[5.0, 7.0, 9.0]);
assert_eq!(shutdown(), RetCode::Success);
```

## Development

Common commands:

```bash
~/.cargo/bin/cargo fmt --all
~/.cargo/bin/cargo clippy --workspace --all-targets
~/.cargo/bin/cargo test --workspace
just fmt
just clippy
just test
```

If `cargo` is not on `PATH` in non-interactive shells:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Release Notes

Before publishing, use the checklist in [docs/RELEASE.md](./docs/RELEASE.md).

# TA-Lib Rust Porting Plan

## Goal

Produce a native Rust implementation of the TA-Lib core library with behavioral parity to upstream C, including:

- the indicator calculations,
- lookback logic,
- parameter validation,
- output sizing and indexing behavior,
- public enums and constants,
- grouped/module organization that is maintainable over time.

## What we are porting

The upstream repository currently exposes `161` financial functions in `ta_func_api.xml`.

Important upstream assets:

- `upstream-ta-lib-c/ta_func_api.xml`: canonical function metadata.
- `upstream-ta-lib-c/src/ta_func/*.c`: indicator implementations.
- `upstream-ta-lib-c/src/tools/ta_regtest/`: regression tests and fixtures.
- `upstream-ta-lib-c/src/tools/gen_code/gen_code.c`: generator entry point.
- `upstream-ta-lib-c/src/tools/gen_code/gen_rust.c`: existing Rust codegen experiment.
- `upstream-ta-lib-c/src/ta_abstract/templates/*.template`: generated API templates.

## Recommended rewrite strategy

Do not hand-port the public API one function at a time from scratch.

Instead:

1. Use upstream metadata to generate Rust signatures, enums, docs, and module layout.
2. Hand-port the numeric kernels in Rust, preserving algorithmic behavior.
3. Verify each function against the C implementation with differential tests.
4. Keep generation and handwritten code clearly separated.

This reduces drift and avoids re-encoding the TA-Lib interface manually.

## Proposed repository structure

Recommended target layout:

- `crates/ta-lib-core/`: pure Rust library with the public API.
- `crates/ta-lib-codegen/`: generator that reads upstream metadata and emits Rust bindings/modules.
- `crates/ta-lib-testdata/`: optional shared test fixtures or differential-test helpers.
- `vendor/` or `upstream-ta-lib-c/`: pinned upstream source used as the reference implementation.
- `generated/` or `crates/ta-lib-core/src/generated/`: generated Rust code.
- `tests/`: cross-check and regression tests.
- `docs/`: design notes and parity tracking.

Current status:

- Workspace root and both initial crates exist.
- The codegen crate parses all 161 upstream functions and can emit a stable Rust index.
- The core crate implements the runtime/context model and compatibility facade.
- Seed functions are ported and verified: `ADD`, `SUB`, `MULT`, `DIV`, `SMA`, `EMA`, `RSI`.
- The current workspace passes `cargo test`, including differential tests against the upstream C library.

## Design decisions we should make early

### 1. API shape

We need to choose whether to preserve the C-style API exactly or expose an idiomatic Rust API with a compatibility layer.

Recommended:

- Keep a low-level compatibility API that mirrors TA-Lib behavior closely.
- Add a higher-level ergonomic layer later.

The compatibility layer should preserve:

- `startIdx` / `endIdx` semantics,
- `outBegIdx` / `outNBElement`,
- optional parameter defaults,
- return codes and validation behavior.

### 2. Numeric precision

TA-Lib supports both double and single precision variants in places.

Recommended:

- Make `f64` the primary implementation target first.
- Add `f32` compatibility after the `f64` path is correct.
- Avoid generic numeric abstractions too early; they usually slow the port down and complicate parity.

### 3. Unsafe vs safe Rust

Recommended:

- Write kernels in safe Rust unless there is a measured performance reason not to.
- Preserve performance with slices, explicit indexing, and benchmark coverage.
- Use `unsafe` only for carefully justified hot paths.

### 4. Generated vs handwritten boundaries

Recommended:

- Generate signatures, enums, dispatch tables, docs, and module declarations.
- Handwrite the actual math kernels and shared helpers.
- Never hand-edit generated files.

## Work breakdown

### Phase 0: foundation

- Initialize this repo as Git.
- Pin the upstream C snapshot.
- Create project layout and workspace.
- Decide licensing and attribution strategy.
- Decide naming policy for Rust modules and functions.

### Phase 1: metadata/codegen

- Parse `ta_func_api.xml`.
- Generate:
  - function signatures,
  - lookback signatures,
  - enum definitions,
  - module declarations,
  - parameter/default metadata,
  - basic Rust docs.
- Keep generation deterministic so diffs stay reviewable.

### Phase 2: core runtime pieces

- Port common return codes and error handling.
- Port shared helper routines from `src/ta_common/`.
- Port unstable-period and compatibility settings only if needed by the targeted indicators.
- Establish exact output-range behavior.

### Phase 3: port indicators in batches

Start with lower-risk functions:

- math transforms,
- overlap studies like `SMA`, `EMA`,
- volatility and momentum indicators,
- candle pattern functions last.

Port in batches and gate merges on differential tests.

### Phase 4: parity testing

- Build upstream C locally for reference outputs.
- For each ported function, compare Rust results to C results across:
  - nominal inputs,
  - edge cases,
  - invalid ranges,
  - optional parameter defaults,
  - short input slices,
  - NaN-sensitive cases if relevant.

### Phase 5: performance and ergonomics

- Benchmark hot indicators against C.
- Profile allocations and bounds-check overhead.
- Add higher-level ergonomic wrappers after compatibility is stable.

## Testing strategy

We need more than unit tests.

Recommended layers:

1. Golden differential tests against upstream C output.
2. Property tests for invariants where they make sense.
3. Snapshot tests for generated code shape.
4. Criterion benchmarks for hot functions.

Important note:

The upstream repo already contains regression material under `src/tools/ta_regtest/`. We should mine that rather than inventing all coverage ourselves.

## Tooling to install on Debian/WSL

Minimum:

- `git`
- `curl`
- `build-essential`
- `pkg-config`
- `cmake`
- `clang`
- `lld`
- `rustup`

Recommended:

- `just`
- `jq`
- `ripgrep`
- `fd-find`
- `python3`
- `python3-venv`
- `python3-pip`

For formatting, linting, testing, and benchmarking:

- Rust components:
  - `rustfmt`
  - `clippy`
  - `llvm-tools-preview`
- Cargo tools:
  - `cargo-nextest`
  - `cargo-deny`
  - `cargo-watch`
  - `cargo-criterion` or plain `criterion` in dev-dependencies

For optional deep verification:

- `valgrind` for checking the upstream C side if needed
- `hyperfine` for benchmark comparisons

Suggested install commands:

```bash
sudo apt update
sudo apt install -y \
  git curl build-essential pkg-config cmake clang lld \
  just jq ripgrep fd-find python3 python3-venv python3-pip valgrind hyperfine

curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustup default stable
rustup component add rustfmt clippy llvm-tools-preview
cargo install cargo-nextest cargo-deny cargo-watch
```

## Practical first milestones

The first useful milestones should be:

1. Create a Cargo workspace.
2. Build a standalone code generator that reads `ta_func_api.xml`.
3. Generate Rust signatures for all 161 functions.
4. Port and verify a small seed set:
   - `MULT`
   - `ADD`
   - `SUB`
   - `DIV`
   - `SMA`
   - `EMA`
   - `RSI`
5. Add a differential harness that runs the C reference and Rust implementation on the same inputs.

## Risks

Main risks:

- silently diverging from TA-Lib edge-case behavior,
- over-designing a generic Rust API before parity is established,
- mixing generated and handwritten code too early,
- underestimating the complexity of candlestick and stateful indicators,
- not capturing optional-parameter defaults exactly.

## Recommendation

The correct way to do this is:

- generator-driven public API,
- batchwise handwritten kernel ports,
- continuous differential verification against upstream C,
- `f64` parity first,
- ergonomics second.

That is the shortest path to a trustworthy Rust TA-Lib.

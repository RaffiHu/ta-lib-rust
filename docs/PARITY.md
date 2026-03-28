# Parity Status

This workspace targets behavioral parity with upstream TA-Lib `v0.6.4-97-g1bdf5438`.

## Current Guarantees

- all 161 upstream functions are implemented
- generated metadata covers the full upstream function inventory
- generated wrappers cover the full upstream surface
- the `Core` compatibility facade covers the generated public surface

## Differential Coverage

The test suite currently contains:

- curated oracle-backed regression tests for representative indicators and stateful behaviors
- generated full-surface C-oracle matrices for:
  - default cases
  - lookback-boundary cases
  - seeded deterministic cases
  - parameter-variant cases

## Stateful Coverage

Targeted parity checks currently cover:

- unstable-period-sensitive indicators such as `EMA`, `RSI`, `ATR`, `NATR`, `IMI`, `MFI`, `CMO`, `KAMA`, `MAMA`, `T3`, and `ADX`
- compatibility-mode-sensitive indicators such as `EMA`, `RSI`, and `CMO`
- candle-setting-sensitive candlestick behavior through direct `Context` and `Core` checks

## What This Does Not Claim

This repository does not claim a formal proof that every possible legal input is identical to upstream C.

What it does claim is:

- full implemented surface parity target
- strong empirical parity via generated full-surface C-oracle matrices
- targeted stateful parity checks for the main mutable TA-Lib runtime settings

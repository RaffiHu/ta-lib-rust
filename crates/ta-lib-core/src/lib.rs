#![doc = include_str!("../../../README.md")]

//! Core Rust implementation for the TA-Lib rewrite.

extern crate alloc;

#[cfg(feature = "std")]
mod compat;
mod context;
mod helpers;
mod indicators;
mod settings;

pub mod generated;

#[cfg(feature = "std")]
pub use compat::{Core, initialize, shutdown};
pub use context::Context;
pub use generated::{FUNCTIONS, FunctionGroup, FunctionInfo};
pub use helpers::{INTEGER_DEFAULT, REAL_DEFAULT};
pub use settings::{
    CandleSetting, CandleSettingType, Compatibility, FuncUnstId, MAType, RangeType, RetCode,
};

/// Upstream snapshot this workspace is currently aligned to.
pub const UPSTREAM_TA_LIB_DESCRIBE: &str = "v0.6.4-97-g1bdf5438";

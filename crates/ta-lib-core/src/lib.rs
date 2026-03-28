#![doc = include_str!("../../../README.md")]

//! Core Rust implementation for the TA-Lib rewrite.

mod compat;
mod context;
mod helpers;
mod indicators;
mod settings;

pub mod generated;

pub use compat::{Core, initialize, shutdown};
pub use context::Context;
pub use generated::{FUNCTIONS, FunctionGroup, FunctionInfo};
pub use helpers::{INTEGER_DEFAULT, REAL_DEFAULT};
pub use settings::{
    CandleSetting, CandleSettingType, Compatibility, FuncUnstId, MAType, RangeType, RetCode,
};

/// Upstream snapshot this workspace is currently aligned to.
pub const UPSTREAM_TA_LIB_DESCRIBE: &str = "v0.6.4-97-g1bdf5438";

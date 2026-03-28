//! TA-Lib - Technical Analysis Library (Rust implementation)
//!
//! This library provides technical analysis functions for financial market data analysis.
//! All types, enums, and function implementations are generated from the C source code
//! to maintain a single source of truth.

#![warn(missing_docs)]

// Declare the generated ta_func module
pub mod ta_func;

// Re-export all generated modules and types
pub use ta_func::*;

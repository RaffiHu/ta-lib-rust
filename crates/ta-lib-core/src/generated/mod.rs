//! Generated-surface scaffolding and build-time emitted wrapper metadata.

use alloc::vec::Vec;

/// Function group names from TA-Lib metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionGroup {
    /// Cycle indicators.
    CycleIndicators,
    /// Math operators.
    MathOperators,
    /// Math transforms.
    MathTransform,
    /// Momentum indicators.
    MomentumIndicators,
    /// Overlap studies.
    OverlapStudies,
    /// Pattern-recognition functions.
    PatternRecognition,
    /// Price transforms.
    PriceTransform,
    /// Statistical functions.
    StatisticFunctions,
    /// Volatility indicators.
    VolatilityIndicators,
    /// Volume indicators.
    VolumeIndicators,
}

impl FunctionGroup {
    /// Returns the upstream display name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CycleIndicators => "Cycle Indicators",
            Self::MathOperators => "Math Operators",
            Self::MathTransform => "Math Transform",
            Self::MomentumIndicators => "Momentum Indicators",
            Self::OverlapStudies => "Overlap Studies",
            Self::PatternRecognition => "Pattern Recognition",
            Self::PriceTransform => "Price Transform",
            Self::StatisticFunctions => "Statistic Functions",
            Self::VolatilityIndicators => "Volatility Indicators",
            Self::VolumeIndicators => "Volume Indicators",
        }
    }
}

/// Minimal metadata emitted by the generated compatibility surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionInfo {
    /// Upstream abbreviation, e.g. `RSI`.
    pub abbreviation: &'static str,
    /// Rust snake_case name.
    pub rust_name: &'static str,
    /// Upstream group.
    pub group: FunctionGroup,
    /// Short description.
    pub description: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmokeRealRun {
    pub ret_code: crate::RetCode,
    pub out_beg_idx: usize,
    pub out_nb_element: usize,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmokeIntegerRun {
    pub ret_code: crate::RetCode,
    pub out_beg_idx: usize,
    pub out_nb_element: usize,
    pub values: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmokeRealPairRun {
    pub ret_code: crate::RetCode,
    pub out_beg_idx: usize,
    pub out_nb_element: usize,
    pub left: Vec<f64>,
    pub right: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmokeIntegerPairRun {
    pub ret_code: crate::RetCode,
    pub out_beg_idx: usize,
    pub out_nb_element: usize,
    pub left: Vec<i32>,
    pub right: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SmokeRealTripleRun {
    pub ret_code: crate::RetCode,
    pub out_beg_idx: usize,
    pub out_nb_element: usize,
    pub first: Vec<f64>,
    pub second: Vec<f64>,
    pub third: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SmokeRun {
    Real(SmokeRealRun),
    Integer(SmokeIntegerRun),
    RealPair(SmokeRealPairRun),
    IntegerPair(SmokeIntegerPairRun),
    RealTriple(SmokeRealTripleRun),
}

impl SmokeRun {
    #[must_use]
    pub const fn ret_code(&self) -> crate::RetCode {
        match self {
            Self::Real(run) => run.ret_code,
            Self::Integer(run) => run.ret_code,
            Self::RealPair(run) => run.ret_code,
            Self::IntegerPair(run) => run.ret_code,
            Self::RealTriple(run) => run.ret_code,
        }
    }

    #[must_use]
    pub const fn out_beg_idx(&self) -> usize {
        match self {
            Self::Real(run) => run.out_beg_idx,
            Self::Integer(run) => run.out_beg_idx,
            Self::RealPair(run) => run.out_beg_idx,
            Self::IntegerPair(run) => run.out_beg_idx,
            Self::RealTriple(run) => run.out_beg_idx,
        }
    }

    #[must_use]
    pub const fn out_nb_element(&self) -> usize {
        match self {
            Self::Real(run) => run.out_nb_element,
            Self::Integer(run) => run.out_nb_element,
            Self::RealPair(run) => run.out_nb_element,
            Self::IntegerPair(run) => run.out_nb_element,
            Self::RealTriple(run) => run.out_nb_element,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SmokeCase {
    pub abbreviation: &'static str,
    pub rust_name: &'static str,
    pub default_run: fn(&crate::Context) -> SmokeRun,
    pub boundary_run: fn(&crate::Context) -> SmokeRun,
    pub seeded_run: fn(&crate::Context) -> SmokeRun,
    pub variant_run: fn(&crate::Context) -> SmokeRun,
    pub invalid_range_run: fn(&crate::Context) -> crate::RetCode,
    pub bad_param_run: fn(&crate::Context) -> crate::RetCode,
    pub lookback_run: fn(&crate::Context) -> i32,
}

include!(concat!(env!("OUT_DIR"), "/generated_function_metadata.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_context_wrappers.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_smoke_cases.rs"));

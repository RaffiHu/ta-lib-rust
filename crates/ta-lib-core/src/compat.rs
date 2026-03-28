use std::sync::{Mutex, OnceLock};

use crate::{CandleSettingType, Compatibility, Context, FuncUnstId, RangeType, RetCode};

static GLOBAL_CONTEXT: OnceLock<Mutex<Option<Context>>> = OnceLock::new();

fn runtime() -> &'static Mutex<Option<Context>> {
    GLOBAL_CONTEXT.get_or_init(|| Mutex::new(None))
}

fn with_context<T>(f: impl FnOnce(&Context) -> T) -> Result<T, RetCode> {
    let guard = runtime()
        .lock()
        .expect("global TA-Lib compatibility mutex poisoned");
    let context = guard.as_ref().ok_or(RetCode::LibNotInitialize)?;
    Ok(f(context))
}

fn with_context_mut<T>(f: impl FnOnce(&mut Context) -> T) -> Result<T, RetCode> {
    let mut guard = runtime()
        .lock()
        .expect("global TA-Lib compatibility mutex poisoned");
    let context = guard.as_mut().ok_or(RetCode::LibNotInitialize)?;
    Ok(f(context))
}

macro_rules! compat_cdl_noopt {
    ($lookback:ident, $func:ident) => {
        #[must_use]
        pub fn $lookback() -> i32 {
            with_context(Context::$lookback).unwrap_or(-1)
        }

        pub fn $func(
            start_idx: usize,
            end_idx: usize,
            in_open: &[f64],
            in_high: &[f64],
            in_low: &[f64],
            in_close: &[f64],
            out_beg_idx: &mut usize,
            out_nb_element: &mut usize,
            out_integer: &mut [i32],
        ) -> RetCode {
            with_context(|context| {
                context.$func(
                    start_idx,
                    end_idx,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    out_beg_idx,
                    out_nb_element,
                    out_integer,
                )
            })
            .unwrap_or(RetCode::LibNotInitialize)
        }
    };
}

/// Initializes the compatibility runtime.
pub fn initialize() -> RetCode {
    let mut guard = runtime()
        .lock()
        .expect("global TA-Lib compatibility mutex poisoned");
    *guard = Some(Context::default());
    RetCode::Success
}

/// Shuts down the compatibility runtime.
pub fn shutdown() -> RetCode {
    let mut guard = runtime()
        .lock()
        .expect("global TA-Lib compatibility mutex poisoned");
    if guard.is_none() {
        return RetCode::LibNotInitialize;
    }

    *guard = None;
    RetCode::Success
}

/// TA-Lib compatibility facade using process-global state.
pub struct Core;

impl Core {
    /// Sets an unstable period on the global compatibility context.
    pub fn set_unstable_period(id: FuncUnstId, unstable_period: u32) -> RetCode {
        with_context_mut(|context| context.set_unstable_period(id, unstable_period))
            .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Gets an unstable period from the global compatibility context.
    #[must_use]
    pub fn get_unstable_period(id: FuncUnstId) -> u32 {
        with_context(|context| context.get_unstable_period(id)).unwrap_or(0)
    }

    /// Sets compatibility mode on the global compatibility context.
    pub fn set_compatibility(value: Compatibility) -> RetCode {
        with_context_mut(|context| context.set_compatibility(value))
            .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Gets the current compatibility mode.
    #[must_use]
    pub fn get_compatibility() -> Compatibility {
        with_context(Context::get_compatibility).unwrap_or(Compatibility::Default)
    }

    /// Sets one candle setting on the global compatibility context.
    pub fn set_candle_settings(
        setting_type: CandleSettingType,
        range_type: RangeType,
        avg_period: i32,
        factor: f64,
    ) -> RetCode {
        with_context_mut(|context| {
            context.set_candle_settings(setting_type, range_type, avg_period, factor)
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Restores default candle settings on the global compatibility context.
    pub fn restore_candle_default_settings(setting_type: CandleSettingType) -> RetCode {
        with_context_mut(|context| context.restore_candle_default_settings(setting_type))
            .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ADD`.
    #[must_use]
    pub fn add_lookback() -> i32 {
        with_context(Context::add_lookback).unwrap_or(-1)
    }

    /// Compatibility `ADD`.
    pub fn add(
        start_idx: usize,
        end_idx: usize,
        in_real0: &[f64],
        in_real1: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.add(
                start_idx,
                end_idx,
                in_real0,
                in_real1,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SUB`.
    #[must_use]
    pub fn sub_lookback() -> i32 {
        with_context(Context::sub_lookback).unwrap_or(-1)
    }

    /// Compatibility `SUB`.
    pub fn sub(
        start_idx: usize,
        end_idx: usize,
        in_real0: &[f64],
        in_real1: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sub(
                start_idx,
                end_idx,
                in_real0,
                in_real1,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MULT`.
    #[must_use]
    pub fn mult_lookback() -> i32 {
        with_context(Context::mult_lookback).unwrap_or(-1)
    }

    /// Compatibility `MULT`.
    pub fn mult(
        start_idx: usize,
        end_idx: usize,
        in_real0: &[f64],
        in_real1: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mult(
                start_idx,
                end_idx,
                in_real0,
                in_real1,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `DIV`.
    #[must_use]
    pub fn div_lookback() -> i32 {
        with_context(Context::div_lookback).unwrap_or(-1)
    }

    /// Compatibility `DIV`.
    pub fn div(
        start_idx: usize,
        end_idx: usize,
        in_real0: &[f64],
        in_real1: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.div(
                start_idx,
                end_idx,
                in_real0,
                in_real1,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SMA`.
    #[must_use]
    pub fn sma_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.sma_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `SMA`.
    pub fn sma(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sma(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `EMA`.
    #[must_use]
    pub fn ema_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.ema_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `EMA`.
    pub fn ema(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ema(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `KAMA`.
    #[must_use]
    pub fn kama_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.kama_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `KAMA`.
    pub fn kama(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.kama(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `BBANDS`.
    #[must_use]
    pub fn bbands_lookback(
        opt_in_time_period: i32,
        opt_in_nb_dev_up: f64,
        opt_in_nb_dev_dn: f64,
        opt_in_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.bbands_lookback(
                opt_in_time_period,
                opt_in_nb_dev_up,
                opt_in_nb_dev_dn,
                opt_in_ma_type,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `BBANDS`.
    #[allow(clippy::too_many_arguments)]
    pub fn bbands(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        opt_in_nb_dev_up: f64,
        opt_in_nb_dev_dn: f64,
        opt_in_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real_upper_band: &mut [f64],
        out_real_middle_band: &mut [f64],
        out_real_lower_band: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.bbands(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                opt_in_nb_dev_up,
                opt_in_nb_dev_dn,
                opt_in_ma_type,
                out_beg_idx,
                out_nb_element,
                out_real_upper_band,
                out_real_middle_band,
                out_real_lower_band,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ACCBANDS`.
    #[must_use]
    pub fn accbands_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.accbands_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ACCBANDS`.
    #[allow(clippy::too_many_arguments)]
    pub fn accbands(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real_upper_band: &mut [f64],
        out_real_middle_band: &mut [f64],
        out_real_lower_band: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.accbands(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real_upper_band,
                out_real_middle_band,
                out_real_lower_band,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MAVP`.
    #[must_use]
    pub fn mavp_lookback(
        opt_in_min_period: i32,
        opt_in_max_period: i32,
        opt_in_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.mavp_lookback(opt_in_min_period, opt_in_max_period, opt_in_ma_type)
        })
        .unwrap_or(-1)
    }

    /// Compatibility `MAVP`.
    #[allow(clippy::too_many_arguments)]
    pub fn mavp(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        in_periods: &[f64],
        opt_in_min_period: i32,
        opt_in_max_period: i32,
        opt_in_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mavp(
                start_idx,
                end_idx,
                in_real,
                in_periods,
                opt_in_min_period,
                opt_in_max_period,
                opt_in_ma_type,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SAR`.
    #[must_use]
    pub fn sar_lookback(opt_in_acceleration: f64, opt_in_maximum: f64) -> i32 {
        with_context(|context| context.sar_lookback(opt_in_acceleration, opt_in_maximum))
            .unwrap_or(-1)
    }

    /// Compatibility `SAR`.
    pub fn sar(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_acceleration: f64,
        opt_in_maximum: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sar(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_acceleration,
                opt_in_maximum,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SAREXT`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn sarext_lookback(
        opt_in_start_value: f64,
        opt_in_offset_on_reverse: f64,
        opt_in_acceleration_init_long: f64,
        opt_in_acceleration_long: f64,
        opt_in_acceleration_max_long: f64,
        opt_in_acceleration_init_short: f64,
        opt_in_acceleration_short: f64,
        opt_in_acceleration_max_short: f64,
    ) -> i32 {
        with_context(|context| {
            context.sarext_lookback(
                opt_in_start_value,
                opt_in_offset_on_reverse,
                opt_in_acceleration_init_long,
                opt_in_acceleration_long,
                opt_in_acceleration_max_long,
                opt_in_acceleration_init_short,
                opt_in_acceleration_short,
                opt_in_acceleration_max_short,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `SAREXT`.
    #[allow(clippy::too_many_arguments)]
    pub fn sarext(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_start_value: f64,
        opt_in_offset_on_reverse: f64,
        opt_in_acceleration_init_long: f64,
        opt_in_acceleration_long: f64,
        opt_in_acceleration_max_long: f64,
        opt_in_acceleration_init_short: f64,
        opt_in_acceleration_short: f64,
        opt_in_acceleration_max_short: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sarext(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_start_value,
                opt_in_offset_on_reverse,
                opt_in_acceleration_init_long,
                opt_in_acceleration_long,
                opt_in_acceleration_max_long,
                opt_in_acceleration_init_short,
                opt_in_acceleration_short,
                opt_in_acceleration_max_short,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MAMA`.
    #[must_use]
    pub fn mama_lookback(opt_in_fast_limit: f64, opt_in_slow_limit: f64) -> i32 {
        with_context(|context| context.mama_lookback(opt_in_fast_limit, opt_in_slow_limit))
            .unwrap_or(-1)
    }

    /// Compatibility `MAMA`.
    pub fn mama(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_fast_limit: f64,
        opt_in_slow_limit: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_mama: &mut [f64],
        out_fama: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mama(
                start_idx,
                end_idx,
                in_real,
                opt_in_fast_limit,
                opt_in_slow_limit,
                out_beg_idx,
                out_nb_element,
                out_mama,
                out_fama,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_DCPERIOD`.
    #[must_use]
    pub fn htdcperiod_lookback() -> i32 {
        with_context(|context| context.ht_dc_period_lookback()).unwrap_or(-1)
    }

    /// Compatibility `HT_DCPERIOD`.
    pub fn htdcperiod(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ht_dc_period(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_DCPERIOD`.
    #[must_use]
    pub fn ht_dc_period_lookback() -> i32 {
        Self::htdcperiod_lookback()
    }

    /// Compatibility `HT_DCPERIOD`.
    pub fn ht_dc_period(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        Self::htdcperiod(
            start_idx,
            end_idx,
            in_real,
            out_beg_idx,
            out_nb_element,
            out_real,
        )
    }

    /// Lookback for `HT_DCPHASE`.
    #[must_use]
    pub fn htdcphase_lookback() -> i32 {
        with_context(|context| context.ht_dc_phase_lookback()).unwrap_or(-1)
    }

    /// Compatibility `HT_DCPHASE`.
    pub fn htdcphase(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ht_dc_phase(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_DCPHASE`.
    #[must_use]
    pub fn ht_dc_phase_lookback() -> i32 {
        Self::htdcphase_lookback()
    }

    /// Compatibility `HT_DCPHASE`.
    pub fn ht_dc_phase(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        Self::htdcphase(
            start_idx,
            end_idx,
            in_real,
            out_beg_idx,
            out_nb_element,
            out_real,
        )
    }

    /// Lookback for `HT_PHASOR`.
    #[must_use]
    pub fn htphasor_lookback() -> i32 {
        with_context(|context| context.ht_phasor_lookback()).unwrap_or(-1)
    }

    /// Compatibility `HT_PHASOR`.
    pub fn htphasor(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_in_phase: &mut [f64],
        out_quadrature: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ht_phasor(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_in_phase,
                out_quadrature,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_PHASOR`.
    #[must_use]
    pub fn ht_phasor_lookback() -> i32 {
        Self::htphasor_lookback()
    }

    /// Compatibility `HT_PHASOR`.
    pub fn ht_phasor(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_in_phase: &mut [f64],
        out_quadrature: &mut [f64],
    ) -> RetCode {
        Self::htphasor(
            start_idx,
            end_idx,
            in_real,
            out_beg_idx,
            out_nb_element,
            out_in_phase,
            out_quadrature,
        )
    }

    /// Lookback for `HT_SINE`.
    #[must_use]
    pub fn htsine_lookback() -> i32 {
        with_context(|context| context.ht_sine_lookback()).unwrap_or(-1)
    }

    /// Compatibility `HT_SINE`.
    pub fn htsine(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_sine: &mut [f64],
        out_lead_sine: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ht_sine(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_sine,
                out_lead_sine,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_SINE`.
    #[must_use]
    pub fn ht_sine_lookback() -> i32 {
        Self::htsine_lookback()
    }

    /// Compatibility `HT_SINE`.
    pub fn ht_sine(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_sine: &mut [f64],
        out_lead_sine: &mut [f64],
    ) -> RetCode {
        Self::htsine(
            start_idx,
            end_idx,
            in_real,
            out_beg_idx,
            out_nb_element,
            out_sine,
            out_lead_sine,
        )
    }

    /// Lookback for `HT_TRENDLINE`.
    #[must_use]
    pub fn httrendline_lookback() -> i32 {
        with_context(|context| context.ht_trendline_lookback()).unwrap_or(-1)
    }

    /// Compatibility `HT_TRENDLINE`.
    pub fn httrendline(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ht_trendline(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_TRENDLINE`.
    #[must_use]
    pub fn ht_trendline_lookback() -> i32 {
        Self::httrendline_lookback()
    }

    /// Compatibility `HT_TRENDLINE`.
    pub fn ht_trendline(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        Self::httrendline(
            start_idx,
            end_idx,
            in_real,
            out_beg_idx,
            out_nb_element,
            out_real,
        )
    }

    /// Lookback for `HT_TRENDMODE`.
    #[must_use]
    pub fn httrendmode_lookback() -> i32 {
        with_context(|context| context.ht_trend_mode_lookback()).unwrap_or(-1)
    }

    /// Compatibility `HT_TRENDMODE`.
    pub fn httrendmode(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.ht_trend_mode(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `HT_TRENDMODE`.
    #[must_use]
    pub fn ht_trend_mode_lookback() -> i32 {
        Self::httrendmode_lookback()
    }

    /// Compatibility `HT_TRENDMODE`.
    pub fn ht_trend_mode(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        Self::httrendmode(
            start_idx,
            end_idx,
            in_real,
            out_beg_idx,
            out_nb_element,
            out_integer,
        )
    }

    /// Lookback for `DEMA`.
    #[must_use]
    pub fn dema_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.dema_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `DEMA`.
    pub fn dema(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.dema(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TEMA`.
    #[must_use]
    pub fn tema_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.tema_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `TEMA`.
    pub fn tema(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.tema(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `T3`.
    #[must_use]
    pub fn t3_lookback(opt_in_time_period: i32, opt_in_v_factor: f64) -> i32 {
        with_context(|context| context.t3_lookback(opt_in_time_period, opt_in_v_factor))
            .unwrap_or(-1)
    }

    /// Compatibility `T3`.
    pub fn t3(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        opt_in_v_factor: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.t3(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                opt_in_v_factor,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TRIMA`.
    #[must_use]
    pub fn trima_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.trima_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `TRIMA`.
    pub fn trima(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.trima(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TRIX`.
    #[must_use]
    pub fn trix_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.trix_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `TRIX`.
    pub fn trix(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.trix(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `WMA`.
    #[must_use]
    pub fn wma_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.wma_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `WMA`.
    pub fn wma(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.wma(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MA`.
    #[must_use]
    pub fn ma_lookback(opt_in_time_period: i32, opt_in_ma_type: i32) -> i32 {
        with_context(|context| context.ma_lookback(opt_in_time_period, opt_in_ma_type))
            .unwrap_or(-1)
    }

    /// Compatibility `MA`.
    pub fn ma(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        opt_in_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ma(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                opt_in_ma_type,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `RSI`.
    #[must_use]
    pub fn rsi_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.rsi_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `RSI`.
    pub fn rsi(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.rsi(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ACOS`.
    #[must_use]
    pub fn acos_lookback() -> i32 {
        with_context(crate::Context::acos_lookback).unwrap_or(-1)
    }

    /// Compatibility `ACOS`.
    pub fn acos(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.acos(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ASIN`.
    #[must_use]
    pub fn asin_lookback() -> i32 {
        with_context(crate::Context::asin_lookback).unwrap_or(-1)
    }

    /// Compatibility `ASIN`.
    pub fn asin(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.asin(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ATAN`.
    #[must_use]
    pub fn atan_lookback() -> i32 {
        with_context(crate::Context::atan_lookback).unwrap_or(-1)
    }

    /// Compatibility `ATAN`.
    pub fn atan(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.atan(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CEIL`.
    #[must_use]
    pub fn ceil_lookback() -> i32 {
        with_context(crate::Context::ceil_lookback).unwrap_or(-1)
    }

    /// Compatibility `CEIL`.
    pub fn ceil(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ceil(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `COS`.
    #[must_use]
    pub fn cos_lookback() -> i32 {
        with_context(crate::Context::cos_lookback).unwrap_or(-1)
    }

    /// Compatibility `COS`.
    pub fn cos(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.cos(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `COSH`.
    #[must_use]
    pub fn cosh_lookback() -> i32 {
        with_context(crate::Context::cosh_lookback).unwrap_or(-1)
    }

    /// Compatibility `COSH`.
    pub fn cosh(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.cosh(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `EXP`.
    #[must_use]
    pub fn exp_lookback() -> i32 {
        with_context(crate::Context::exp_lookback).unwrap_or(-1)
    }

    /// Compatibility `EXP`.
    pub fn exp(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.exp(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `FLOOR`.
    #[must_use]
    pub fn floor_lookback() -> i32 {
        with_context(crate::Context::floor_lookback).unwrap_or(-1)
    }

    /// Compatibility `FLOOR`.
    pub fn floor(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.floor(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `LN`.
    #[must_use]
    pub fn ln_lookback() -> i32 {
        with_context(crate::Context::ln_lookback).unwrap_or(-1)
    }

    /// Compatibility `LN`.
    pub fn ln(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ln(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `LOG10`.
    #[must_use]
    pub fn log10_lookback() -> i32 {
        with_context(crate::Context::log10_lookback).unwrap_or(-1)
    }

    /// Compatibility `LOG10`.
    pub fn log10(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.log10(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SIN`.
    #[must_use]
    pub fn sin_lookback() -> i32 {
        with_context(crate::Context::sin_lookback).unwrap_or(-1)
    }

    /// Compatibility `SIN`.
    pub fn sin(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sin(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SINH`.
    #[must_use]
    pub fn sinh_lookback() -> i32 {
        with_context(crate::Context::sinh_lookback).unwrap_or(-1)
    }

    /// Compatibility `SINH`.
    pub fn sinh(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sinh(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SQRT`.
    #[must_use]
    pub fn sqrt_lookback() -> i32 {
        with_context(crate::Context::sqrt_lookback).unwrap_or(-1)
    }

    /// Compatibility `SQRT`.
    pub fn sqrt(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sqrt(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TAN`.
    #[must_use]
    pub fn tan_lookback() -> i32 {
        with_context(crate::Context::tan_lookback).unwrap_or(-1)
    }

    /// Compatibility `TAN`.
    pub fn tan(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.tan(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TANH`.
    #[must_use]
    pub fn tanh_lookback() -> i32 {
        with_context(crate::Context::tanh_lookback).unwrap_or(-1)
    }

    /// Compatibility `TANH`.
    pub fn tanh(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.tanh(
                start_idx,
                end_idx,
                in_real,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `SUM`.
    #[must_use]
    pub fn sum_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.sum_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `SUM`.
    pub fn sum(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.sum(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MAX`.
    #[must_use]
    pub fn max_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.max_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MAX`.
    pub fn max(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.max(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MIN`.
    #[must_use]
    pub fn min_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.min_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MIN`.
    pub fn min(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.min(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MAXINDEX`.
    #[must_use]
    pub fn max_index_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.max_index_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MAXINDEX`.
    pub fn max_index(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.max_index(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MININDEX`.
    #[must_use]
    pub fn min_index_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.min_index_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MININDEX`.
    pub fn min_index(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.min_index(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MINMAX`.
    #[must_use]
    pub fn min_max_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.min_max_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MINMAX`.
    pub fn min_max(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_min: &mut [f64],
        out_max: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.min_max(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_min,
                out_max,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MINMAXINDEX`.
    #[must_use]
    pub fn min_max_index_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.min_max_index_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MINMAXINDEX`.
    pub fn min_max_index(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_min_idx: &mut [i32],
        out_max_idx: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.min_max_index(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_min_idx,
                out_max_idx,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `AVGPRICE`.
    #[must_use]
    pub fn avg_price_lookback() -> i32 {
        with_context(crate::Context::avg_price_lookback).unwrap_or(-1)
    }

    /// Compatibility `AVGPRICE`.
    pub fn avg_price(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.avg_price(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MEDPRICE`.
    #[must_use]
    pub fn med_price_lookback() -> i32 {
        with_context(crate::Context::med_price_lookback).unwrap_or(-1)
    }

    /// Compatibility `MEDPRICE`.
    pub fn med_price(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.med_price(
                start_idx,
                end_idx,
                in_high,
                in_low,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TYPPRICE`.
    #[must_use]
    pub fn typ_price_lookback() -> i32 {
        with_context(crate::Context::typ_price_lookback).unwrap_or(-1)
    }

    /// Compatibility `TYPPRICE`.
    pub fn typ_price(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.typ_price(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `WCLPRICE`.
    #[must_use]
    pub fn wcl_price_lookback() -> i32 {
        with_context(crate::Context::wcl_price_lookback).unwrap_or(-1)
    }

    /// Compatibility `WCLPRICE`.
    pub fn wcl_price(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.wcl_price(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MIDPOINT`.
    #[must_use]
    pub fn mid_point_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.mid_point_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MIDPOINT`.
    pub fn mid_point(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mid_point(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MIDPRICE`.
    #[must_use]
    pub fn mid_price_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.mid_price_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MIDPRICE`.
    pub fn mid_price(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mid_price(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TRANGE`.
    #[must_use]
    pub fn true_range_lookback() -> i32 {
        with_context(crate::Context::true_range_lookback).unwrap_or(-1)
    }

    /// Compatibility `TRANGE`.
    pub fn true_range(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.true_range(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `AVGDEV`.
    #[must_use]
    pub fn avg_dev_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.avg_dev_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `AVGDEV`.
    pub fn avg_dev(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.avg_dev(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `BOP`.
    #[must_use]
    pub fn bop_lookback() -> i32 {
        with_context(crate::Context::bop_lookback).unwrap_or(-1)
    }

    /// Compatibility `BOP`.
    pub fn bop(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.bop(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `IMI`.
    #[must_use]
    pub fn imi_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.imi_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `IMI`.
    pub fn imi(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.imi(
                start_idx,
                end_idx,
                in_open,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MOM`.
    #[must_use]
    pub fn mom_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.mom_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MOM`.
    pub fn mom(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mom(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ROC`.
    #[must_use]
    pub fn roc_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.roc_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ROC`.
    pub fn roc(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.roc(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ROCP`.
    #[must_use]
    pub fn roc_p_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.roc_p_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ROCP`.
    pub fn roc_p(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.roc_p(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ROCR`.
    #[must_use]
    pub fn roc_r_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.roc_r_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ROCR`.
    pub fn roc_r(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.roc_r(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ROCR100`.
    #[must_use]
    pub fn roc_r100_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.roc_r100_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ROCR100`.
    pub fn roc_r100(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.roc_r100(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ATR`.
    #[must_use]
    pub fn atr_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.atr_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ATR`.
    pub fn atr(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.atr(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `NATR`.
    #[must_use]
    pub fn natr_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.natr_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `NATR`.
    pub fn natr(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.natr(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `AD`.
    #[must_use]
    pub fn ad_lookback() -> i32 {
        with_context(crate::Context::ad_lookback).unwrap_or(-1)
    }

    /// Compatibility `AD`.
    pub fn ad(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        in_volume: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ad(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                in_volume,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `OBV`.
    #[must_use]
    pub fn obv_lookback() -> i32 {
        with_context(crate::Context::obv_lookback).unwrap_or(-1)
    }

    /// Compatibility `OBV`.
    pub fn obv(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        in_volume: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.obv(
                start_idx,
                end_idx,
                in_real,
                in_volume,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `WILLR`.
    #[must_use]
    pub fn will_r_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.will_r_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `WILLR`.
    pub fn will_r(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.will_r(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `AROON`.
    #[must_use]
    pub fn aroon_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.aroon_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `AROON`.
    pub fn aroon(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_aroon_down: &mut [f64],
        out_aroon_up: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.aroon(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_aroon_down,
                out_aroon_up,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `AROONOSC`.
    #[must_use]
    pub fn aroon_osc_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.aroon_osc_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `AROONOSC`.
    pub fn aroon_osc(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.aroon_osc(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `VAR`.
    #[must_use]
    pub fn variance_lookback(opt_in_time_period: i32, opt_in_nb_dev: f64) -> i32 {
        with_context(|context| context.variance_lookback(opt_in_time_period, opt_in_nb_dev))
            .unwrap_or(-1)
    }

    /// Compatibility `VAR`.
    pub fn variance(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        opt_in_nb_dev: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.variance(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                opt_in_nb_dev,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `STDDEV`.
    #[must_use]
    pub fn std_dev_lookback(opt_in_time_period: i32, opt_in_nb_dev: f64) -> i32 {
        with_context(|context| context.std_dev_lookback(opt_in_time_period, opt_in_nb_dev))
            .unwrap_or(-1)
    }

    /// Compatibility `STDDEV`.
    pub fn std_dev(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        opt_in_nb_dev: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.std_dev(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                opt_in_nb_dev,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CORREL`.
    #[must_use]
    pub fn correl_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.correl_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `CORREL`.
    pub fn correl(
        start_idx: usize,
        end_idx: usize,
        in_real0: &[f64],
        in_real1: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.correl(
                start_idx,
                end_idx,
                in_real0,
                in_real1,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `BETA`.
    #[must_use]
    pub fn beta_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.beta_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `BETA`.
    pub fn beta(
        start_idx: usize,
        end_idx: usize,
        in_real0: &[f64],
        in_real1: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.beta(
                start_idx,
                end_idx,
                in_real0,
                in_real1,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `LINEARREG`.
    #[must_use]
    pub fn linearreg_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.linearreg_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `LINEARREG`.
    pub fn linearreg(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.linearreg(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `LINEARREG_ANGLE`.
    #[must_use]
    pub fn linearreg_angle_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.linearreg_angle_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `LINEARREG_ANGLE`.
    pub fn linearreg_angle(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.linearreg_angle(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `LINEARREG_INTERCEPT`.
    #[must_use]
    pub fn linearreg_intercept_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.linearreg_intercept_lookback(opt_in_time_period))
            .unwrap_or(-1)
    }

    /// Compatibility `LINEARREG_INTERCEPT`.
    pub fn linearreg_intercept(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.linearreg_intercept(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `LINEARREG_SLOPE`.
    #[must_use]
    pub fn linearreg_slope_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.linearreg_slope_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `LINEARREG_SLOPE`.
    pub fn linearreg_slope(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.linearreg_slope(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `TSF`.
    #[must_use]
    pub fn tsf_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.tsf_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `TSF`.
    pub fn tsf(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.tsf(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CCI`.
    #[must_use]
    pub fn cci_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.cci_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `CCI`.
    pub fn cci(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.cci(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CMO`.
    #[must_use]
    pub fn cmo_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.cmo_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `CMO`.
    pub fn cmo(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.cmo(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `STOCHF`.
    #[must_use]
    pub fn stochf_lookback(
        opt_in_fast_k_period: i32,
        opt_in_fast_d_period: i32,
        opt_in_fast_d_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.stochf_lookback(
                opt_in_fast_k_period,
                opt_in_fast_d_period,
                opt_in_fast_d_ma_type,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `STOCHF`.
    pub fn stochf(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_fast_k_period: i32,
        opt_in_fast_d_period: i32,
        opt_in_fast_d_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_fast_k: &mut [f64],
        out_fast_d: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.stochf(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_fast_k_period,
                opt_in_fast_d_period,
                opt_in_fast_d_ma_type,
                out_beg_idx,
                out_nb_element,
                out_fast_k,
                out_fast_d,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `STOCH`.
    #[must_use]
    pub fn stoch_lookback(
        opt_in_fast_k_period: i32,
        opt_in_slow_k_period: i32,
        opt_in_slow_k_ma_type: i32,
        opt_in_slow_d_period: i32,
        opt_in_slow_d_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.stoch_lookback(
                opt_in_fast_k_period,
                opt_in_slow_k_period,
                opt_in_slow_k_ma_type,
                opt_in_slow_d_period,
                opt_in_slow_d_ma_type,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `STOCH`.
    pub fn stoch(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_fast_k_period: i32,
        opt_in_slow_k_period: i32,
        opt_in_slow_k_ma_type: i32,
        opt_in_slow_d_period: i32,
        opt_in_slow_d_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_slow_k: &mut [f64],
        out_slow_d: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.stoch(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_fast_k_period,
                opt_in_slow_k_period,
                opt_in_slow_k_ma_type,
                opt_in_slow_d_period,
                opt_in_slow_d_ma_type,
                out_beg_idx,
                out_nb_element,
                out_slow_k,
                out_slow_d,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `STOCHRSI`.
    #[must_use]
    pub fn stochrsi_lookback(
        opt_in_time_period: i32,
        opt_in_fast_k_period: i32,
        opt_in_fast_d_period: i32,
        opt_in_fast_d_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.stochrsi_lookback(
                opt_in_time_period,
                opt_in_fast_k_period,
                opt_in_fast_d_period,
                opt_in_fast_d_ma_type,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `STOCHRSI`.
    pub fn stochrsi(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_time_period: i32,
        opt_in_fast_k_period: i32,
        opt_in_fast_d_period: i32,
        opt_in_fast_d_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_fast_k: &mut [f64],
        out_fast_d: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.stochrsi(
                start_idx,
                end_idx,
                in_real,
                opt_in_time_period,
                opt_in_fast_k_period,
                opt_in_fast_d_period,
                opt_in_fast_d_ma_type,
                out_beg_idx,
                out_nb_element,
                out_fast_k,
                out_fast_d,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ULTOSC`.
    #[must_use]
    pub fn ultosc_lookback(
        opt_in_time_period1: i32,
        opt_in_time_period2: i32,
        opt_in_time_period3: i32,
    ) -> i32 {
        with_context(|context| {
            context.ultosc_lookback(
                opt_in_time_period1,
                opt_in_time_period2,
                opt_in_time_period3,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `ULTOSC`.
    pub fn ultosc(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period1: i32,
        opt_in_time_period2: i32,
        opt_in_time_period3: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ultosc(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period1,
                opt_in_time_period2,
                opt_in_time_period3,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `PLUS_DM`.
    #[must_use]
    pub fn plus_dm_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.plus_dm_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `PLUS_DM`.
    pub fn plus_dm(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.plus_dm(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MINUS_DM`.
    #[must_use]
    pub fn minus_dm_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.minus_dm_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MINUS_DM`.
    pub fn minus_dm(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.minus_dm(
                start_idx,
                end_idx,
                in_high,
                in_low,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `PLUS_DI`.
    #[must_use]
    pub fn plus_di_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.plus_di_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `PLUS_DI`.
    pub fn plus_di(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.plus_di(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MINUS_DI`.
    #[must_use]
    pub fn minus_di_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.minus_di_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MINUS_DI`.
    pub fn minus_di(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.minus_di(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `DX`.
    #[must_use]
    pub fn dx_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.dx_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `DX`.
    pub fn dx(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.dx(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ADX`.
    #[must_use]
    pub fn adx_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.adx_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ADX`.
    pub fn adx(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.adx(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ADXR`.
    #[must_use]
    pub fn adxr_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.adxr_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `ADXR`.
    pub fn adxr(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.adxr(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `APO`.
    #[must_use]
    pub fn apo_lookback(
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        opt_in_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.apo_lookback(opt_in_fast_period, opt_in_slow_period, opt_in_ma_type)
        })
        .unwrap_or(-1)
    }

    /// Compatibility `APO`.
    pub fn apo(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        opt_in_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.apo(
                start_idx,
                end_idx,
                in_real,
                opt_in_fast_period,
                opt_in_slow_period,
                opt_in_ma_type,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `PPO`.
    #[must_use]
    pub fn ppo_lookback(
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        opt_in_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.ppo_lookback(opt_in_fast_period, opt_in_slow_period, opt_in_ma_type)
        })
        .unwrap_or(-1)
    }

    /// Compatibility `PPO`.
    pub fn ppo(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        opt_in_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ppo(
                start_idx,
                end_idx,
                in_real,
                opt_in_fast_period,
                opt_in_slow_period,
                opt_in_ma_type,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MACD`.
    #[must_use]
    pub fn macd_lookback(
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        opt_in_signal_period: i32,
    ) -> i32 {
        with_context(|context| {
            context.macd_lookback(opt_in_fast_period, opt_in_slow_period, opt_in_signal_period)
        })
        .unwrap_or(-1)
    }

    /// Compatibility `MACD`.
    pub fn macd(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        opt_in_signal_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_macd: &mut [f64],
        out_macd_signal: &mut [f64],
        out_macd_hist: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.macd(
                start_idx,
                end_idx,
                in_real,
                opt_in_fast_period,
                opt_in_slow_period,
                opt_in_signal_period,
                out_beg_idx,
                out_nb_element,
                out_macd,
                out_macd_signal,
                out_macd_hist,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MACDFIX`.
    #[must_use]
    pub fn macd_fix_lookback(opt_in_signal_period: i32) -> i32 {
        with_context(|context| context.macdfix_lookback(opt_in_signal_period)).unwrap_or(-1)
    }

    /// Lookback for `MACDFIX`.
    #[must_use]
    pub fn macdfix_lookback(opt_in_signal_period: i32) -> i32 {
        Self::macd_fix_lookback(opt_in_signal_period)
    }

    /// Compatibility `MACDFIX`.
    pub fn macd_fix(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_signal_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_macd: &mut [f64],
        out_macd_signal: &mut [f64],
        out_macd_hist: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.macdfix(
                start_idx,
                end_idx,
                in_real,
                opt_in_signal_period,
                out_beg_idx,
                out_nb_element,
                out_macd,
                out_macd_signal,
                out_macd_hist,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Compatibility `MACDFIX`.
    pub fn macdfix(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_signal_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_macd: &mut [f64],
        out_macd_signal: &mut [f64],
        out_macd_hist: &mut [f64],
    ) -> RetCode {
        Self::macd_fix(
            start_idx,
            end_idx,
            in_real,
            opt_in_signal_period,
            out_beg_idx,
            out_nb_element,
            out_macd,
            out_macd_signal,
            out_macd_hist,
        )
    }

    /// Lookback for `MACDEXT`.
    #[must_use]
    pub fn macdext_lookback(
        opt_in_fast_period: i32,
        opt_in_fast_ma_type: i32,
        opt_in_slow_period: i32,
        opt_in_slow_ma_type: i32,
        opt_in_signal_period: i32,
        opt_in_signal_ma_type: i32,
    ) -> i32 {
        with_context(|context| {
            context.macdext_lookback(
                opt_in_fast_period,
                opt_in_fast_ma_type,
                opt_in_slow_period,
                opt_in_slow_ma_type,
                opt_in_signal_period,
                opt_in_signal_ma_type,
            )
        })
        .unwrap_or(-1)
    }

    /// Compatibility `MACDEXT`.
    #[allow(clippy::too_many_arguments)]
    pub fn macdext(
        start_idx: usize,
        end_idx: usize,
        in_real: &[f64],
        opt_in_fast_period: i32,
        opt_in_fast_ma_type: i32,
        opt_in_slow_period: i32,
        opt_in_slow_ma_type: i32,
        opt_in_signal_period: i32,
        opt_in_signal_ma_type: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_macd: &mut [f64],
        out_macd_signal: &mut [f64],
        out_macd_hist: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.macdext(
                start_idx,
                end_idx,
                in_real,
                opt_in_fast_period,
                opt_in_fast_ma_type,
                opt_in_slow_period,
                opt_in_slow_ma_type,
                opt_in_signal_period,
                opt_in_signal_ma_type,
                out_beg_idx,
                out_nb_element,
                out_macd,
                out_macd_signal,
                out_macd_hist,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `MFI`.
    #[must_use]
    pub fn mfi_lookback(opt_in_time_period: i32) -> i32 {
        with_context(|context| context.mfi_lookback(opt_in_time_period)).unwrap_or(-1)
    }

    /// Compatibility `MFI`.
    pub fn mfi(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        in_volume: &[f64],
        opt_in_time_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.mfi(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                in_volume,
                opt_in_time_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `ADOSC`.
    #[must_use]
    pub fn ad_osc_lookback(opt_in_fast_period: i32, opt_in_slow_period: i32) -> i32 {
        with_context(|context| context.ad_osc_lookback(opt_in_fast_period, opt_in_slow_period))
            .unwrap_or(-1)
    }

    /// Compatibility `ADOSC`.
    pub fn ad_osc(
        start_idx: usize,
        end_idx: usize,
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        in_volume: &[f64],
        opt_in_fast_period: i32,
        opt_in_slow_period: i32,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_real: &mut [f64],
    ) -> RetCode {
        with_context(|context| {
            context.ad_osc(
                start_idx,
                end_idx,
                in_high,
                in_low,
                in_close,
                in_volume,
                opt_in_fast_period,
                opt_in_slow_period,
                out_beg_idx,
                out_nb_element,
                out_real,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLDOJI`.
    #[must_use]
    pub fn cdl_doji_lookback() -> i32 {
        with_context(Context::cdl_doji_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLDOJI`.
    pub fn cdl_doji(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_doji(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLDRAGONFLYDOJI`.
    #[must_use]
    pub fn cdl_dragonfly_doji_lookback() -> i32 {
        with_context(Context::cdl_dragonfly_doji_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLDRAGONFLYDOJI`.
    pub fn cdl_dragonfly_doji(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_dragonfly_doji(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLGRAVESTONEDOJI`.
    #[must_use]
    pub fn cdl_gravestone_doji_lookback() -> i32 {
        with_context(Context::cdl_gravestone_doji_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLGRAVESTONEDOJI`.
    pub fn cdl_gravestone_doji(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_gravestone_doji(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLSPINNINGTOP`.
    #[must_use]
    pub fn cdl_spinning_top_lookback() -> i32 {
        with_context(Context::cdl_spinning_top_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLSPINNINGTOP`.
    pub fn cdl_spinning_top(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_spinning_top(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLMARUBOZU`.
    #[must_use]
    pub fn cdl_marubozu_lookback() -> i32 {
        with_context(Context::cdl_marubozu_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLMARUBOZU`.
    pub fn cdl_marubozu(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_marubozu(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLLONGLINE`.
    #[must_use]
    pub fn cdl_long_line_lookback() -> i32 {
        with_context(Context::cdl_long_line_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLLONGLINE`.
    pub fn cdl_long_line(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_long_line(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLSHORTLINE`.
    #[must_use]
    pub fn cdl_short_line_lookback() -> i32 {
        with_context(Context::cdl_short_line_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLSHORTLINE`.
    pub fn cdl_short_line(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_short_line(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLHAMMER`.
    #[must_use]
    pub fn cdl_hammer_lookback() -> i32 {
        with_context(Context::cdl_hammer_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLHAMMER`.
    pub fn cdl_hammer(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_hammer(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLENGULFING`.
    #[must_use]
    pub fn cdl_engulfing_lookback() -> i32 {
        with_context(Context::cdl_engulfing_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLENGULFING`.
    pub fn cdl_engulfing(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_engulfing(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLHARAMI`.
    #[must_use]
    pub fn cdl_harami_lookback() -> i32 {
        with_context(Context::cdl_harami_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLHARAMI`.
    pub fn cdl_harami(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_harami(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLHARAMICROSS`.
    #[must_use]
    pub fn cdl_harami_cross_lookback() -> i32 {
        with_context(Context::cdl_harami_cross_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLHARAMICROSS`.
    pub fn cdl_harami_cross(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_harami_cross(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLBELTHOLD`.
    #[must_use]
    pub fn cdl_belt_hold_lookback() -> i32 {
        with_context(Context::cdl_belt_hold_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLBELTHOLD`.
    pub fn cdl_belt_hold(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_belt_hold(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLCLOSINGMARUBOZU`.
    #[must_use]
    pub fn cdl_closing_marubozu_lookback() -> i32 {
        with_context(Context::cdl_closing_marubozu_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLCLOSINGMARUBOZU`.
    pub fn cdl_closing_marubozu(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_closing_marubozu(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLHIGHWAVE`.
    #[must_use]
    pub fn cdl_high_wave_lookback() -> i32 {
        with_context(Context::cdl_high_wave_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLHIGHWAVE`.
    pub fn cdl_high_wave(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_high_wave(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLINVERTEDHAMMER`.
    #[must_use]
    pub fn cdl_inverted_hammer_lookback() -> i32 {
        with_context(Context::cdl_inverted_hammer_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLINVERTEDHAMMER`.
    pub fn cdl_inverted_hammer(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_inverted_hammer(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLSHOOTINGSTAR`.
    #[must_use]
    pub fn cdl_shooting_star_lookback() -> i32 {
        with_context(Context::cdl_shooting_star_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLSHOOTINGSTAR`.
    pub fn cdl_shooting_star(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_shooting_star(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLTAKURI`.
    #[must_use]
    pub fn cdl_takuri_lookback() -> i32 {
        with_context(Context::cdl_takuri_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLTAKURI`.
    pub fn cdl_takuri(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_takuri(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLCOUNTERATTACK`.
    #[must_use]
    pub fn cdl_counter_attack_lookback() -> i32 {
        with_context(Context::cdl_counter_attack_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLCOUNTERATTACK`.
    pub fn cdl_counter_attack(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_counter_attack(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLHOMINGPIGEON`.
    #[must_use]
    pub fn cdl_homing_pigeon_lookback() -> i32 {
        with_context(Context::cdl_homing_pigeon_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLHOMINGPIGEON`.
    pub fn cdl_homing_pigeon(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_homing_pigeon(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLINNECK`.
    #[must_use]
    pub fn cdl_in_neck_lookback() -> i32 {
        with_context(Context::cdl_in_neck_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLINNECK`.
    pub fn cdl_in_neck(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_in_neck(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLONNECK`.
    #[must_use]
    pub fn cdl_on_neck_lookback() -> i32 {
        with_context(Context::cdl_on_neck_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLONNECK`.
    pub fn cdl_on_neck(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_on_neck(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLTHRUSTING`.
    #[must_use]
    pub fn cdl_thrusting_lookback() -> i32 {
        with_context(Context::cdl_thrusting_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLTHRUSTING`.
    pub fn cdl_thrusting(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_thrusting(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLMATCHINGLOW`.
    #[must_use]
    pub fn cdl_matching_low_lookback() -> i32 {
        with_context(Context::cdl_matching_low_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLMATCHINGLOW`.
    pub fn cdl_matching_low(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_matching_low(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDL2CROWS`.
    #[must_use]
    pub fn cdl_2_crows_lookback() -> i32 {
        with_context(Context::cdl_2_crows_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDL2CROWS`.
    pub fn cdl_2_crows(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_2_crows(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDL3BLACKCROWS`.
    #[must_use]
    pub fn cdl_3_black_crows_lookback() -> i32 {
        with_context(Context::cdl_3_black_crows_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDL3BLACKCROWS`.
    pub fn cdl_3_black_crows(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_3_black_crows(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLDARKCLOUDCOVER`.
    #[must_use]
    pub fn cdl_dark_cloud_cover_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_dark_cloud_cover_lookback(opt_in_penetration))
            .unwrap_or(-1)
    }

    /// Compatibility `CDLDARKCLOUDCOVER`.
    #[allow(clippy::too_many_arguments)]
    pub fn cdl_dark_cloud_cover(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_dark_cloud_cover(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLPIERCING`.
    #[must_use]
    pub fn cdl_piercing_lookback() -> i32 {
        with_context(Context::cdl_piercing_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLPIERCING`.
    pub fn cdl_piercing(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_piercing(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLSEPARATINGLINES`.
    #[must_use]
    pub fn cdl_separating_lines_lookback() -> i32 {
        with_context(Context::cdl_separating_lines_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLSEPARATINGLINES`.
    pub fn cdl_separating_lines(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_separating_lines(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDLSTICKSANDWICH`.
    #[must_use]
    pub fn cdl_stick_sandwich_lookback() -> i32 {
        with_context(Context::cdl_stick_sandwich_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDLSTICKSANDWICH`.
    pub fn cdl_stick_sandwich(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_stick_sandwich(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDL3INSIDE`.
    #[must_use]
    pub fn cdl_3_inside_lookback() -> i32 {
        with_context(Context::cdl_3_inside_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDL3INSIDE`.
    pub fn cdl_3_inside(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_3_inside(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    /// Lookback for `CDL3OUTSIDE`.
    #[must_use]
    pub fn cdl_3_outside_lookback() -> i32 {
        with_context(Context::cdl_3_outside_lookback).unwrap_or(-1)
    }

    /// Compatibility `CDL3OUTSIDE`.
    pub fn cdl_3_outside(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_3_outside(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_hanging_man_lookback() -> i32 {
        with_context(Context::cdl_hanging_man_lookback).unwrap_or(-1)
    }

    pub fn cdl_hanging_man(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_hanging_man(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_doji_star_lookback() -> i32 {
        with_context(Context::cdl_doji_star_lookback).unwrap_or(-1)
    }

    pub fn cdl_doji_star(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_doji_star(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_evening_star_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_evening_star_lookback(opt_in_penetration)).unwrap_or(-1)
    }

    pub fn cdl_evening_star(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_evening_star(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_evening_doji_star_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_evening_doji_star_lookback(opt_in_penetration))
            .unwrap_or(-1)
    }

    pub fn cdl_evening_doji_star(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_evening_doji_star(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_morning_star_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_morning_star_lookback(opt_in_penetration)).unwrap_or(-1)
    }

    pub fn cdl_morning_star(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_morning_star(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_morning_doji_star_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_morning_doji_star_lookback(opt_in_penetration))
            .unwrap_or(-1)
    }

    pub fn cdl_morning_doji_star(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_morning_doji_star(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_long_legged_doji_lookback() -> i32 {
        with_context(Context::cdl_long_legged_doji_lookback).unwrap_or(-1)
    }

    pub fn cdl_long_legged_doji(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_long_legged_doji(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_rickshaw_man_lookback() -> i32 {
        with_context(Context::cdl_rickshaw_man_lookback).unwrap_or(-1)
    }

    pub fn cdl_rickshaw_man(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_rickshaw_man(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    compat_cdl_noopt!(cdl_tristar_lookback, cdl_tristar);
    compat_cdl_noopt!(cdl_unique_3_river_lookback, cdl_unique_3_river);
    compat_cdl_noopt!(cdl_breakaway_lookback, cdl_breakaway);
    compat_cdl_noopt!(cdl_tasuki_gap_lookback, cdl_tasuki_gap);
    compat_cdl_noopt!(cdl_upside_gap_2_crows_lookback, cdl_upside_gap_2_crows);
    compat_cdl_noopt!(cdl_kicking_lookback, cdl_kicking);
    compat_cdl_noopt!(cdl_kicking_by_length_lookback, cdl_kicking_by_length);
    compat_cdl_noopt!(cdl_rise_fall_3_methods_lookback, cdl_rise_fall_3_methods);
    compat_cdl_noopt!(cdl_identical_3_crows_lookback, cdl_identical_3_crows);
    compat_cdl_noopt!(cdl_3_white_soldiers_lookback, cdl_3_white_soldiers);
    compat_cdl_noopt!(cdl_3_line_strike_lookback, cdl_3_line_strike);
    compat_cdl_noopt!(cdl_advance_block_lookback, cdl_advance_block);
    compat_cdl_noopt!(cdl_stalled_pattern_lookback, cdl_stalled_pattern);
    compat_cdl_noopt!(cdl_gap_side_side_white_lookback, cdl_gap_side_side_white);
    compat_cdl_noopt!(cdl_3_stars_in_south_lookback, cdl_3_stars_in_south);
    compat_cdl_noopt!(cdl_conceal_babys_wall_lookback, cdl_conceal_babys_wall);
    compat_cdl_noopt!(cdl_hikkake_lookback, cdl_hikkake);
    compat_cdl_noopt!(cdl_hikkake_mod_lookback, cdl_hikkake_mod);
    compat_cdl_noopt!(cdl_ladder_bottom_lookback, cdl_ladder_bottom);
    compat_cdl_noopt!(cdl_x_side_gap_3_methods_lookback, cdl_x_side_gap_3_methods);

    #[must_use]
    pub fn cdl_mat_hold_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_mat_hold_lookback(opt_in_penetration)).unwrap_or(-1)
    }

    pub fn cdl_mat_hold(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_mat_hold(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }

    #[must_use]
    pub fn cdl_abandoned_baby_lookback(opt_in_penetration: f64) -> i32 {
        with_context(|context| context.cdl_abandoned_baby_lookback(opt_in_penetration))
            .unwrap_or(-1)
    }

    pub fn cdl_abandoned_baby(
        start_idx: usize,
        end_idx: usize,
        in_open: &[f64],
        in_high: &[f64],
        in_low: &[f64],
        in_close: &[f64],
        opt_in_penetration: f64,
        out_beg_idx: &mut usize,
        out_nb_element: &mut usize,
        out_integer: &mut [i32],
    ) -> RetCode {
        with_context(|context| {
            context.cdl_abandoned_baby(
                start_idx,
                end_idx,
                in_open,
                in_high,
                in_low,
                in_close,
                opt_in_penetration,
                out_beg_idx,
                out_nb_element,
                out_integer,
            )
        })
        .unwrap_or(RetCode::LibNotInitialize)
    }
}

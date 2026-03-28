use alloc::vec;

use crate::helpers::{
    INTEGER_DEFAULT, normalize_period, validate_input_len, validate_output_len, validate_range,
};
use crate::indicators::{ema, overlap_ops, sma};
use crate::{Context, MAType, RetCode};

fn normalize_ma_type(value: i32) -> Result<MAType, RetCode> {
    if value == INTEGER_DEFAULT {
        return Ok(MAType::Sma);
    }

    match value {
        0 => Ok(MAType::Sma),
        1 => Ok(MAType::Ema),
        2 => Ok(MAType::Wma),
        3 => Ok(MAType::Dema),
        4 => Ok(MAType::Tema),
        5 => Ok(MAType::Trima),
        6 => Ok(MAType::Kama),
        7 => Ok(MAType::Mama),
        8 => Ok(MAType::T3),
        _ => Err(RetCode::BadParam),
    }
}

fn ma_lookback(context: &Context, period: i32, ma_type: MAType) -> Result<i32, RetCode> {
    match ma_type {
        MAType::Sma => Ok(sma::lookback(period)),
        MAType::Ema => Ok(ema::lookback(context, period)),
        MAType::Wma => Ok(overlap_ops::wma_lookback(period)),
        MAType::Trima => Ok(overlap_ops::trima_lookback(period)),
        MAType::Kama => Ok(overlap_ops::kama_lookback(context, period)),
        MAType::T3 => Ok(overlap_ops::t3_lookback(
            context,
            period,
            crate::REAL_DEFAULT,
        )),
        _ => Err(RetCode::NotSupported),
    }
}

fn ma_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    period: i32,
    ma_type: MAType,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    match ma_type {
        MAType::Sma => sma::run(
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Ema => ema::run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Wma => overlap_ops::wma_run(
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Trima => overlap_ops::trima_run(
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Kama => overlap_ops::kama_run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::T3 => overlap_ops::t3_run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            crate::REAL_DEFAULT,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        _ => RetCode::NotSupported,
    }
}

pub(crate) fn apo_lookback(
    context: &Context,
    opt_in_fast_period: i32,
    opt_in_slow_period: i32,
    opt_in_ma_type: i32,
) -> i32 {
    let Ok(fast) = normalize_period(opt_in_fast_period, 12, 2, 100_000) else {
        return -1;
    };
    let Ok(slow) = normalize_period(opt_in_slow_period, 26, 2, 100_000) else {
        return -1;
    };
    let Ok(ma_type) = normalize_ma_type(opt_in_ma_type) else {
        return -1;
    };
    ma_lookback(context, fast.max(slow) as i32, ma_type).unwrap_or(-1)
}

pub(crate) fn apo_run(
    context: &Context,
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
    price_oscillator_run(
        context,
        start_idx,
        end_idx,
        in_real,
        opt_in_fast_period,
        opt_in_slow_period,
        opt_in_ma_type,
        out_beg_idx,
        out_nb_element,
        out_real,
        false,
    )
}

pub(crate) fn ppo_lookback(
    context: &Context,
    opt_in_fast_period: i32,
    opt_in_slow_period: i32,
    opt_in_ma_type: i32,
) -> i32 {
    apo_lookback(
        context,
        opt_in_fast_period,
        opt_in_slow_period,
        opt_in_ma_type,
    )
}

pub(crate) fn ppo_run(
    context: &Context,
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
    price_oscillator_run(
        context,
        start_idx,
        end_idx,
        in_real,
        opt_in_fast_period,
        opt_in_slow_period,
        opt_in_ma_type,
        out_beg_idx,
        out_nb_element,
        out_real,
        true,
    )
}

fn price_oscillator_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_fast_period: i32,
    opt_in_slow_period: i32,
    opt_in_ma_type: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
    percentage: bool,
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let Ok(mut fast) = normalize_period(opt_in_fast_period, 12, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(mut slow) = normalize_period(opt_in_slow_period, 26, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(ma_type) = normalize_ma_type(opt_in_ma_type) else {
        return RetCode::BadParam;
    };
    if slow < fast {
        core::mem::swap(&mut fast, &mut slow);
    }

    let fast_lookback = match ma_lookback(context, fast as i32, ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let slow_lookback = match ma_lookback(context, slow as i32, ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let lookback = fast_lookback.max(slow_lookback);
    let adjusted_start = start_idx.max(lookback as usize);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }
    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut fast_buf = vec![0.0; end_idx + 1];
    let mut slow_buf = vec![0.0; end_idx + 1];
    let mut fast_beg = 0usize;
    let mut fast_nb = 0usize;
    let mut slow_beg = 0usize;
    let mut slow_nb = 0usize;
    let ret = ma_run(
        context,
        0,
        end_idx,
        in_real,
        fast as i32,
        ma_type,
        &mut fast_beg,
        &mut fast_nb,
        &mut fast_buf,
    );
    if ret != RetCode::Success {
        return ret;
    }
    let ret = ma_run(
        context,
        0,
        end_idx,
        in_real,
        slow as i32,
        ma_type,
        &mut slow_beg,
        &mut slow_nb,
        &mut slow_buf,
    );
    if ret != RetCode::Success {
        return ret;
    }
    if fast_beg != fast_lookback as usize || slow_beg != slow_lookback as usize {
        return RetCode::InternalError;
    }

    for i in 0..needed {
        let today = adjusted_start + i;
        let fast_val = fast_buf[today - fast_beg];
        let slow_val = slow_buf[today - slow_beg];
        out_real[i] = if percentage {
            if slow_val != 0.0 {
                ((fast_val - slow_val) / slow_val) * 100.0
            } else {
                0.0
            }
        } else {
            fast_val - slow_val
        };
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn macd_lookback(
    context: &Context,
    opt_in_fast_period: i32,
    opt_in_slow_period: i32,
    opt_in_signal_period: i32,
) -> i32 {
    let Ok(mut fast) = normalize_period(opt_in_fast_period, 12, 2, 100_000) else {
        return -1;
    };
    let Ok(mut slow) = normalize_period(opt_in_slow_period, 26, 2, 100_000) else {
        return -1;
    };
    let Ok(signal) = normalize_period(opt_in_signal_period, 9, 1, 100_000) else {
        return -1;
    };
    if slow < fast {
        core::mem::swap(&mut fast, &mut slow);
    }
    ema::lookback(context, slow as i32) + ema::lookback(context, signal as i32)
}

pub(crate) fn macd_run(
    context: &Context,
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
    let Ok(mut fast) = normalize_period(opt_in_fast_period, 12, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(mut slow) = normalize_period(opt_in_slow_period, 26, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(signal) = normalize_period(opt_in_signal_period, 9, 1, 100_000) else {
        return RetCode::BadParam;
    };
    if slow < fast {
        core::mem::swap(&mut fast, &mut slow);
    }
    macd_internal(
        context,
        start_idx,
        end_idx,
        in_real,
        slow,
        fast,
        signal,
        2.0 / (slow as f64 + 1.0),
        2.0 / (fast as f64 + 1.0),
        out_beg_idx,
        out_nb_element,
        out_macd,
        out_macd_signal,
        out_macd_hist,
    )
}

pub(crate) fn macdfix_lookback(context: &Context, opt_in_signal_period: i32) -> i32 {
    let Ok(signal) = normalize_period(opt_in_signal_period, 9, 1, 100_000) else {
        return -1;
    };
    ema::lookback(context, 26) + ema::lookback(context, signal as i32)
}

pub(crate) fn macdfix_run(
    context: &Context,
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
    let Ok(signal) = normalize_period(opt_in_signal_period, 9, 1, 100_000) else {
        return RetCode::BadParam;
    };
    macd_internal(
        context,
        start_idx,
        end_idx,
        in_real,
        26,
        12,
        signal,
        0.075,
        0.15,
        out_beg_idx,
        out_nb_element,
        out_macd,
        out_macd_signal,
        out_macd_hist,
    )
}

pub(crate) fn macdext_lookback(
    context: &Context,
    opt_in_fast_period: i32,
    opt_in_fast_ma_type: i32,
    opt_in_slow_period: i32,
    opt_in_slow_ma_type: i32,
    opt_in_signal_period: i32,
    opt_in_signal_ma_type: i32,
) -> i32 {
    let Ok(mut fast) = normalize_period(opt_in_fast_period, 12, 2, 100_000) else {
        return -1;
    };
    let Ok(mut slow) = normalize_period(opt_in_slow_period, 26, 2, 100_000) else {
        return -1;
    };
    let Ok(signal) = normalize_period(opt_in_signal_period, 9, 1, 100_000) else {
        return -1;
    };
    let Ok(fast_ma_type) = normalize_ma_type(opt_in_fast_ma_type) else {
        return -1;
    };
    let Ok(slow_ma_type) = normalize_ma_type(opt_in_slow_ma_type) else {
        return -1;
    };
    let Ok(signal_ma_type) = normalize_ma_type(opt_in_signal_ma_type) else {
        return -1;
    };
    if slow < fast {
        core::mem::swap(&mut fast, &mut slow);
    }

    let fast_lb = overlap_ops::ma_lookback_dispatch(context, fast as i32, fast_ma_type as i32);
    let slow_lb = overlap_ops::ma_lookback_dispatch(context, slow as i32, slow_ma_type as i32);
    let signal_lb =
        overlap_ops::ma_lookback_dispatch(context, signal as i32, signal_ma_type as i32);
    match (fast_lb, slow_lb, signal_lb) {
        (Ok(fast_lb), Ok(slow_lb), Ok(signal_lb)) => fast_lb.max(slow_lb) + signal_lb,
        (Err(ret_code), _, _) | (_, Err(ret_code), _) | (_, _, Err(ret_code)) => match ret_code {
            RetCode::NotSupported => -1,
            _ => -1,
        },
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn macdext_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let Ok(mut fast) = normalize_period(opt_in_fast_period, 12, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(mut slow) = normalize_period(opt_in_slow_period, 26, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(signal) = normalize_period(opt_in_signal_period, 9, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let mut fast_ma_type = match normalize_ma_type(opt_in_fast_ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let mut slow_ma_type = match normalize_ma_type(opt_in_slow_ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let signal_ma_type = match normalize_ma_type(opt_in_signal_ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    if slow < fast {
        core::mem::swap(&mut fast, &mut slow);
        core::mem::swap(&mut fast_ma_type, &mut slow_ma_type);
    }

    let fast_lb = match overlap_ops::ma_lookback_dispatch(context, fast as i32, fast_ma_type as i32)
    {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };
    let slow_lb = match overlap_ops::ma_lookback_dispatch(context, slow as i32, slow_ma_type as i32)
    {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };
    let signal_lb =
        match overlap_ops::ma_lookback_dispatch(context, signal as i32, signal_ma_type as i32) {
            Ok(value) => value as usize,
            Err(ret_code) => return ret_code,
        };
    let lookback_total = fast_lb.max(slow_lb) + signal_lb;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_macd, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_macd_signal, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_macd_hist, needed) {
        return ret_code;
    }

    let temp_start = adjusted_start - signal_lb;
    let temp_needed = end_idx - temp_start + 1;
    let mut slow_buf = vec![0.0; temp_needed];
    let mut fast_buf = vec![0.0; temp_needed];
    let mut slow_beg = 0usize;
    let mut fast_beg = 0usize;
    let mut slow_nb = 0usize;
    let mut fast_nb = 0usize;

    let ret_code = overlap_ops::ma_run_dispatch(
        context,
        temp_start,
        end_idx,
        in_real,
        slow as i32,
        slow_ma_type as i32,
        &mut slow_beg,
        &mut slow_nb,
        &mut slow_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    let ret_code = overlap_ops::ma_run_dispatch(
        context,
        temp_start,
        end_idx,
        in_real,
        fast as i32,
        fast_ma_type as i32,
        &mut fast_beg,
        &mut fast_nb,
        &mut fast_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if slow_beg != temp_start
        || fast_beg != temp_start
        || slow_nb != temp_needed
        || fast_nb != temp_needed
    {
        return RetCode::InternalError;
    }

    let mut diff = vec![0.0; temp_needed];
    for i in 0..temp_needed {
        diff[i] = fast_buf[i] - slow_buf[i];
    }

    let mut signal_beg = 0usize;
    let mut signal_nb = 0usize;
    let ret_code = overlap_ops::ma_run_dispatch(
        context,
        signal_lb,
        temp_needed - 1,
        &diff,
        signal as i32,
        signal_ma_type as i32,
        &mut signal_beg,
        &mut signal_nb,
        out_macd_signal,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if signal_beg != signal_lb || signal_nb != needed {
        return RetCode::InternalError;
    }

    for i in 0..needed {
        out_macd[i] = diff[i + signal_lb];
        out_macd_hist[i] = out_macd[i] - out_macd_signal[i];
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

fn macd_internal(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    slow_period: usize,
    fast_period: usize,
    signal_period: usize,
    slow_k: f64,
    fast_k: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_macd: &mut [f64],
    out_macd_signal: &mut [f64],
    out_macd_hist: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let lookback_signal = ema::lookback(context, signal_period as i32) as usize;
    let lookback_total = lookback_signal + ema::lookback(context, slow_period as i32) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }
    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_macd, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_macd_signal, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_macd_hist, needed) {
        return ret_code;
    }

    let temp_start = adjusted_start - lookback_signal;
    let temp_needed = end_idx - temp_start + 1;
    let mut slow_buf = vec![0.0; temp_needed];
    let mut fast_buf = vec![0.0; temp_needed];
    let mut slow_beg = 0usize;
    let mut fast_beg = 0usize;
    let mut slow_nb = 0usize;
    let mut fast_nb = 0usize;
    let ret = ema::run_with_k(
        context,
        temp_start,
        end_idx,
        in_real,
        slow_period,
        slow_k,
        &mut slow_beg,
        &mut slow_nb,
        &mut slow_buf,
    );
    if ret != RetCode::Success {
        return ret;
    }
    let ret = ema::run_with_k(
        context,
        temp_start,
        end_idx,
        in_real,
        fast_period,
        fast_k,
        &mut fast_beg,
        &mut fast_nb,
        &mut fast_buf,
    );
    if ret != RetCode::Success {
        return ret;
    }
    if slow_beg != temp_start
        || fast_beg != temp_start
        || slow_nb != temp_needed
        || fast_nb != temp_needed
    {
        return RetCode::InternalError;
    }

    let mut diff = vec![0.0; temp_needed];
    for i in 0..temp_needed {
        diff[i] = fast_buf[i] - slow_buf[i];
        out_macd[i.saturating_sub(lookback_signal)] =
            if i >= lookback_signal { diff[i] } else { 0.0 };
    }

    let mut signal_buf = vec![0.0; needed];
    let mut signal_beg = 0usize;
    let mut signal_nb = 0usize;
    let ret = ema::run(
        context,
        0,
        temp_needed - 1,
        &diff,
        signal_period as i32,
        &mut signal_beg,
        &mut signal_nb,
        &mut signal_buf,
    );
    if ret != RetCode::Success {
        return ret;
    }
    if signal_beg != lookback_signal || signal_nb != needed {
        return RetCode::InternalError;
    }

    for i in 0..needed {
        out_macd[i] = diff[i + lookback_signal];
        out_macd_signal[i] = signal_buf[i];
        out_macd_hist[i] = out_macd[i] - out_macd_signal[i];
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

use crate::helpers::{
    INTEGER_DEFAULT, normalize_period, validate_input_len, validate_output_len, validate_range,
};
use crate::indicators::{ema, sma};
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
        _ => RetCode::NotSupported,
    }
}

fn validate_hlc(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
) -> Result<(), RetCode> {
    validate_range(start_idx, end_idx)?;
    for input in [in_high, in_low, in_close] {
        validate_input_len(input, end_idx)?;
    }
    Ok(())
}

fn fast_k_values(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    period: usize,
    out: &mut [f64],
) {
    for (out_idx, today) in (start_idx..=end_idx).enumerate() {
        let trailing_idx = today - (period - 1);
        let mut highest = in_high[trailing_idx];
        let mut lowest = in_low[trailing_idx];
        for i in trailing_idx + 1..=today {
            let high = in_high[i];
            let low = in_low[i];
            if high >= highest {
                highest = high;
            }
            if low <= lowest {
                lowest = low;
            }
        }

        let diff = highest - lowest;
        out[out_idx] = if diff != 0.0 {
            ((in_close[today] - lowest) / diff) * 100.0
        } else {
            0.0
        };
    }
}

pub(crate) fn ultosc_lookback(
    opt_in_time_period1: i32,
    opt_in_time_period2: i32,
    opt_in_time_period3: i32,
) -> i32 {
    let Ok(period1) = normalize_period(opt_in_time_period1, 7, 1, 100_000) else {
        return -1;
    };
    let Ok(period2) = normalize_period(opt_in_time_period2, 14, 1, 100_000) else {
        return -1;
    };
    let Ok(period3) = normalize_period(opt_in_time_period3, 28, 1, 100_000) else {
        return -1;
    };

    period1.max(period2).max(period3) as i32
}

pub(crate) fn ultosc_run(
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(p1) = normalize_period(opt_in_time_period1, 7, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(p2) = normalize_period(opt_in_time_period2, 14, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(p3) = normalize_period(opt_in_time_period3, 28, 1, 100_000) else {
        return RetCode::BadParam;
    };

    let mut periods = [p1, p2, p3];
    periods.sort_unstable();
    let [short_period, mid_period, long_period] = periods;

    let adjusted_start = start_idx.max(long_period);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let mut bp1 = 0.0;
        let mut tr1 = 0.0;
        for day in today - short_period + 1..=today {
            let true_low = in_low[day].min(in_close[day - 1]);
            let true_high = in_high[day].max(in_close[day - 1]);
            bp1 += in_close[day] - true_low;
            tr1 += true_high - true_low;
        }

        let mut bp2 = 0.0;
        let mut tr2 = 0.0;
        for day in today - mid_period + 1..=today {
            let true_low = in_low[day].min(in_close[day - 1]);
            let true_high = in_high[day].max(in_close[day - 1]);
            bp2 += in_close[day] - true_low;
            tr2 += true_high - true_low;
        }

        let mut bp3 = 0.0;
        let mut tr3 = 0.0;
        for day in today - long_period + 1..=today {
            let true_low = in_low[day].min(in_close[day - 1]);
            let true_high = in_high[day].max(in_close[day - 1]);
            bp3 += in_close[day] - true_low;
            tr3 += true_high - true_low;
        }

        let avg1 = if tr1 != 0.0 { bp1 / tr1 } else { 0.0 };
        let avg2 = if tr2 != 0.0 { bp2 / tr2 } else { 0.0 };
        let avg3 = if tr3 != 0.0 { bp3 / tr3 } else { 0.0 };
        out_real[out_idx] = 100.0 * ((4.0 * avg1) + (2.0 * avg2) + avg3) / 7.0;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn stochf_lookback(
    context: &Context,
    opt_in_fast_k_period: i32,
    opt_in_fast_d_period: i32,
    opt_in_fast_d_ma_type: i32,
) -> i32 {
    let Ok(fast_k_period) = normalize_period(opt_in_fast_k_period, 5, 1, 100_000) else {
        return -1;
    };
    let Ok(fast_d_period) = normalize_period(opt_in_fast_d_period, 3, 1, 100_000) else {
        return -1;
    };
    let Ok(ma_type) = normalize_ma_type(opt_in_fast_d_ma_type) else {
        return -1;
    };
    let Ok(ma_lookback) = ma_lookback(context, fast_d_period as i32, ma_type) else {
        return -1;
    };

    fast_k_period as i32 - 1 + ma_lookback
}

pub(crate) fn stochf_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(fast_k_period) = normalize_period(opt_in_fast_k_period, 5, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(fast_d_period) = normalize_period(opt_in_fast_d_period, 3, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(ma_type) = normalize_ma_type(opt_in_fast_d_ma_type) else {
        return RetCode::BadParam;
    };
    let fast_d_lookback = match ma_lookback(context, fast_d_period as i32, ma_type) {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };

    let lookback_total = fast_k_period - 1 + fast_d_lookback;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_fast_k, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_fast_d, needed) {
        return ret_code;
    }

    let fast_k_start = adjusted_start - fast_d_lookback;
    let mut fast_k_buffer = vec![0.0; end_idx - fast_k_start + 1];
    fast_k_values(
        fast_k_start,
        end_idx,
        in_high,
        in_low,
        in_close,
        fast_k_period,
        &mut fast_k_buffer,
    );

    let mut fast_d_beg = 0usize;
    let mut fast_d_nb = 0usize;
    let ret_code = ma_run(
        context,
        fast_d_lookback,
        fast_k_buffer.len() - 1,
        &fast_k_buffer,
        fast_d_period as i32,
        ma_type,
        &mut fast_d_beg,
        &mut fast_d_nb,
        out_fast_d,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if fast_d_beg != fast_d_lookback || fast_d_nb != needed {
        return RetCode::InternalError;
    }

    out_fast_k[..needed].copy_from_slice(&fast_k_buffer[fast_d_lookback..fast_d_lookback + needed]);
    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn stoch_lookback(
    context: &Context,
    opt_in_fast_k_period: i32,
    opt_in_slow_k_period: i32,
    opt_in_slow_k_ma_type: i32,
    opt_in_slow_d_period: i32,
    opt_in_slow_d_ma_type: i32,
) -> i32 {
    let Ok(fast_k_period) = normalize_period(opt_in_fast_k_period, 5, 1, 100_000) else {
        return -1;
    };
    let Ok(slow_k_period) = normalize_period(opt_in_slow_k_period, 3, 1, 100_000) else {
        return -1;
    };
    let Ok(slow_d_period) = normalize_period(opt_in_slow_d_period, 3, 1, 100_000) else {
        return -1;
    };
    let Ok(slow_k_ma_type) = normalize_ma_type(opt_in_slow_k_ma_type) else {
        return -1;
    };
    let Ok(slow_d_ma_type) = normalize_ma_type(opt_in_slow_d_ma_type) else {
        return -1;
    };
    let Ok(slow_k_lookback) = ma_lookback(context, slow_k_period as i32, slow_k_ma_type) else {
        return -1;
    };
    let Ok(slow_d_lookback) = ma_lookback(context, slow_d_period as i32, slow_d_ma_type) else {
        return -1;
    };

    fast_k_period as i32 - 1 + slow_k_lookback + slow_d_lookback
}

pub(crate) fn stoch_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(fast_k_period) = normalize_period(opt_in_fast_k_period, 5, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(slow_k_period) = normalize_period(opt_in_slow_k_period, 3, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(slow_d_period) = normalize_period(opt_in_slow_d_period, 3, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(slow_k_ma_type) = normalize_ma_type(opt_in_slow_k_ma_type) else {
        return RetCode::BadParam;
    };
    let Ok(slow_d_ma_type) = normalize_ma_type(opt_in_slow_d_ma_type) else {
        return RetCode::BadParam;
    };
    let slow_k_lookback = match ma_lookback(context, slow_k_period as i32, slow_k_ma_type) {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };
    let slow_d_lookback = match ma_lookback(context, slow_d_period as i32, slow_d_ma_type) {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };

    let lookback_total = fast_k_period - 1 + slow_k_lookback + slow_d_lookback;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_slow_k, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_slow_d, needed) {
        return ret_code;
    }

    let fast_k_start = adjusted_start - slow_k_lookback - slow_d_lookback;
    let mut fast_k_buffer = vec![0.0; end_idx - fast_k_start + 1];
    fast_k_values(
        fast_k_start,
        end_idx,
        in_high,
        in_low,
        in_close,
        fast_k_period,
        &mut fast_k_buffer,
    );

    let mut slow_k_buffer = vec![0.0; fast_k_buffer.len() - slow_k_lookback];
    let mut slow_k_beg = 0usize;
    let mut slow_k_nb = 0usize;
    let ret_code = ma_run(
        context,
        slow_k_lookback,
        fast_k_buffer.len() - 1,
        &fast_k_buffer,
        slow_k_period as i32,
        slow_k_ma_type,
        &mut slow_k_beg,
        &mut slow_k_nb,
        &mut slow_k_buffer,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if slow_k_beg != slow_k_lookback {
        return RetCode::InternalError;
    }

    let mut slow_d_beg = 0usize;
    let mut slow_d_nb = 0usize;
    let ret_code = ma_run(
        context,
        slow_d_lookback,
        slow_k_nb - 1,
        &slow_k_buffer[..slow_k_nb],
        slow_d_period as i32,
        slow_d_ma_type,
        &mut slow_d_beg,
        &mut slow_d_nb,
        out_slow_d,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if slow_d_beg != slow_d_lookback || slow_d_nb != needed {
        return RetCode::InternalError;
    }

    out_slow_k[..needed].copy_from_slice(&slow_k_buffer[slow_d_lookback..slow_d_lookback + needed]);
    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn stochrsi_lookback(
    context: &Context,
    opt_in_time_period: i32,
    opt_in_fast_k_period: i32,
    opt_in_fast_d_period: i32,
    opt_in_fast_d_ma_type: i32,
) -> i32 {
    let rsi_lb = crate::indicators::rsi::lookback(context, opt_in_time_period);
    let stochf_lb = stochf_lookback(
        context,
        opt_in_fast_k_period,
        opt_in_fast_d_period,
        opt_in_fast_d_ma_type,
    );
    if rsi_lb < 0 || stochf_lb < 0 {
        -1
    } else {
        rsi_lb + stochf_lb
    }
}

pub(crate) fn stochrsi_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let lookback_total = stochrsi_lookback(
        context,
        opt_in_time_period,
        opt_in_fast_k_period,
        opt_in_fast_d_period,
        opt_in_fast_d_ma_type,
    );
    if lookback_total < 0 {
        return RetCode::BadParam;
    }
    let adjusted_start = start_idx.max(lookback_total as usize);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_fast_k, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_fast_d, needed) {
        return ret_code;
    }

    let stochf_lb = stochf_lookback(
        context,
        opt_in_fast_k_period,
        opt_in_fast_d_period,
        opt_in_fast_d_ma_type,
    ) as usize;
    let rsi_start = adjusted_start - stochf_lb;
    let mut rsi_buffer = vec![0.0; end_idx - rsi_start + 1];
    let mut rsi_beg = 0usize;
    let mut rsi_nb = 0usize;
    let ret_code = crate::indicators::rsi::run(
        context,
        rsi_start,
        end_idx,
        in_real,
        opt_in_time_period,
        &mut rsi_beg,
        &mut rsi_nb,
        &mut rsi_buffer,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if rsi_beg != rsi_start {
        return RetCode::InternalError;
    }

    let mut stoch_beg = 0usize;
    let mut stoch_nb = 0usize;
    let ret_code = stochf_run(
        context,
        stochf_lb,
        rsi_nb - 1,
        &rsi_buffer[..rsi_nb],
        &rsi_buffer[..rsi_nb],
        &rsi_buffer[..rsi_nb],
        opt_in_fast_k_period,
        opt_in_fast_d_period,
        opt_in_fast_d_ma_type,
        &mut stoch_beg,
        &mut stoch_nb,
        out_fast_k,
        out_fast_d,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if stoch_beg != stochf_lb || stoch_nb != needed {
        return RetCode::InternalError;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

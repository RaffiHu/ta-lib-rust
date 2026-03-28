use crate::helpers::{
    is_zero, normalize_period, validate_input_len, validate_output_len, validate_range,
};
use crate::{Context, FuncUnstId, RetCode};

fn dm_diff(prev_high: f64, prev_low: f64, high: f64, low: f64) -> (f64, f64) {
    let diff_p = high - prev_high;
    let diff_m = prev_low - low;

    if diff_m > 0.0 && diff_p < diff_m {
        (0.0, diff_m)
    } else if diff_p > 0.0 && diff_p > diff_m {
        (diff_p, 0.0)
    } else {
        (0.0, 0.0)
    }
}

fn true_range(high: f64, low: f64, prev_close: f64) -> f64 {
    let high_low = high - low;
    let high_close = (high - prev_close).abs();
    let low_close = (low - prev_close).abs();
    high_low.max(high_close).max(low_close)
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

fn validate_hl(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
) -> Result<(), RetCode> {
    validate_range(start_idx, end_idx)?;
    for input in [in_high, in_low] {
        validate_input_len(input, end_idx)?;
    }
    Ok(())
}

pub(crate) fn plus_dm_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return -1;
    };

    if period > 1 {
        period as i32 + context.get_unstable_period(FuncUnstId::PlusDM) as i32 - 1
    } else {
        1
    }
}

pub(crate) fn minus_dm_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return -1;
    };

    if period > 1 {
        period as i32 + context.get_unstable_period(FuncUnstId::MinusDM) as i32 - 1
    } else {
        1
    }
}

fn dm_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    opt_in_time_period: i32,
    unstable_id: FuncUnstId,
    plus: bool,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hl(start_idx, end_idx, in_high, in_low) {
        return ret_code;
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return RetCode::BadParam;
    };

    let unstable = context.get_unstable_period(unstable_id) as usize;
    let lookback = if period > 1 { period + unstable - 1 } else { 1 };
    let adjusted_start = start_idx.max(lookback);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if period <= 1 {
        let mut today = adjusted_start - 1;
        let mut prev_high = in_high[today];
        let mut prev_low = in_low[today];

        for out in &mut out_real[..needed] {
            today += 1;
            let high = in_high[today];
            let low = in_low[today];
            let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
            *out = if plus { plus_dm } else { minus_dm };
            prev_high = high;
            prev_low = low;
        }

        *out_beg_idx = adjusted_start;
        *out_nb_element = needed;
        return RetCode::Success;
    }

    let mut today = adjusted_start - lookback;
    let mut prev_high = in_high[today];
    let mut prev_low = in_low[today];
    let mut prev_dm = 0.0;

    for _ in 0..(period - 1) {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_dm += if plus { plus_dm } else { minus_dm };
        prev_high = high;
        prev_low = low;
    }

    for _ in 0..unstable {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        let current_dm = if plus { plus_dm } else { minus_dm };
        prev_dm = prev_dm - (prev_dm / period as f64) + current_dm;
        prev_high = high;
        prev_low = low;
    }

    out_real[0] = prev_dm;
    let mut out_idx = 1;
    while today < end_idx {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        let current_dm = if plus { plus_dm } else { minus_dm };
        prev_dm = prev_dm - (prev_dm / period as f64) + current_dm;
        prev_high = high;
        prev_low = low;
        out_real[out_idx] = prev_dm;
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn plus_dm_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    dm_run(
        context,
        start_idx,
        end_idx,
        in_high,
        in_low,
        opt_in_time_period,
        FuncUnstId::PlusDM,
        true,
        out_beg_idx,
        out_nb_element,
        out_real,
    )
}

pub(crate) fn minus_dm_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    dm_run(
        context,
        start_idx,
        end_idx,
        in_high,
        in_low,
        opt_in_time_period,
        FuncUnstId::MinusDM,
        false,
        out_beg_idx,
        out_nb_element,
        out_real,
    )
}

pub(crate) fn plus_di_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return -1;
    };

    if period > 1 {
        period as i32 + context.get_unstable_period(FuncUnstId::PlusDI) as i32
    } else {
        1
    }
}

pub(crate) fn minus_di_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return -1;
    };

    if period > 1 {
        period as i32 + context.get_unstable_period(FuncUnstId::MinusDI) as i32
    } else {
        1
    }
}

fn di_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    opt_in_time_period: i32,
    unstable_id: FuncUnstId,
    plus: bool,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return RetCode::BadParam;
    };

    let unstable = context.get_unstable_period(unstable_id) as usize;
    let lookback = if period > 1 { period + unstable } else { 1 };
    let adjusted_start = start_idx.max(lookback);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if period <= 1 {
        let mut today = adjusted_start - 1;
        let mut prev_high = in_high[today];
        let mut prev_low = in_low[today];
        let mut prev_close = in_close[today];

        for out in &mut out_real[..needed] {
            today += 1;
            let high = in_high[today];
            let low = in_low[today];
            let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
            let tr = true_range(high, low, prev_close);
            let dm = if plus { plus_dm } else { minus_dm };
            *out = if is_zero(tr) { 0.0 } else { dm / tr };
            prev_high = high;
            prev_low = low;
            prev_close = in_close[today];
        }

        *out_beg_idx = adjusted_start;
        *out_nb_element = needed;
        return RetCode::Success;
    }

    let mut today = adjusted_start - lookback;
    let mut prev_high = in_high[today];
    let mut prev_low = in_low[today];
    let mut prev_close = in_close[today];
    let mut prev_dm = 0.0;
    let mut prev_tr = 0.0;

    for _ in 0..(period - 1) {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_dm += if plus { plus_dm } else { minus_dm };
        prev_tr += true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];
    }

    for _ in 0..=unstable {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        let dm = if plus { plus_dm } else { minus_dm };
        prev_dm = prev_dm - (prev_dm / period as f64) + dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];
    }

    out_real[0] = if is_zero(prev_tr) {
        0.0
    } else {
        100.0 * (prev_dm / prev_tr)
    };

    let mut out_idx = 1;
    while today < end_idx {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        let dm = if plus { plus_dm } else { minus_dm };
        prev_dm = prev_dm - (prev_dm / period as f64) + dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];

        out_real[out_idx] = if is_zero(prev_tr) {
            0.0
        } else {
            100.0 * (prev_dm / prev_tr)
        };
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn plus_di_run(
    context: &Context,
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
    di_run(
        context,
        start_idx,
        end_idx,
        in_high,
        in_low,
        in_close,
        opt_in_time_period,
        FuncUnstId::PlusDI,
        true,
        out_beg_idx,
        out_nb_element,
        out_real,
    )
}

pub(crate) fn minus_di_run(
    context: &Context,
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
    di_run(
        context,
        start_idx,
        end_idx,
        in_high,
        in_low,
        in_close,
        opt_in_time_period,
        FuncUnstId::MinusDI,
        false,
        out_beg_idx,
        out_nb_element,
        out_real,
    )
}

pub(crate) fn dx_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32 + context.get_unstable_period(FuncUnstId::Dx) as i32
}

pub(crate) fn dx_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let unstable = context.get_unstable_period(FuncUnstId::Dx) as usize;
    let lookback = period + unstable;
    let adjusted_start = start_idx.max(lookback);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut today = adjusted_start - lookback;
    let mut prev_high = in_high[today];
    let mut prev_low = in_low[today];
    let mut prev_close = in_close[today];
    let mut prev_minus_dm = 0.0;
    let mut prev_plus_dm = 0.0;
    let mut prev_tr = 0.0;

    for _ in 0..(period - 1) {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr += true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];
    }

    let mut prev_dx = 0.0;
    for offset in 0..=unstable {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm -= prev_minus_dm / period as f64;
        prev_plus_dm -= prev_plus_dm / period as f64;
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];

        let current_dx = if is_zero(prev_tr) {
            if offset == 0 { 0.0 } else { prev_dx }
        } else {
            let minus_di = 100.0 * (prev_minus_dm / prev_tr);
            let plus_di = 100.0 * (prev_plus_dm / prev_tr);
            let sum_di = minus_di + plus_di;
            if is_zero(sum_di) {
                if offset == 0 { 0.0 } else { prev_dx }
            } else {
                100.0 * ((minus_di - plus_di).abs() / sum_di)
            }
        };

        if offset == unstable {
            out_real[0] = current_dx;
        }
        prev_dx = current_dx;
    }

    let mut out_idx = 1;
    while today < end_idx {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm -= prev_minus_dm / period as f64;
        prev_plus_dm -= prev_plus_dm / period as f64;
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];

        let current_dx = if is_zero(prev_tr) {
            prev_dx
        } else {
            let minus_di = 100.0 * (prev_minus_dm / prev_tr);
            let plus_di = 100.0 * (prev_plus_dm / prev_tr);
            let sum_di = minus_di + plus_di;
            if is_zero(sum_di) {
                prev_dx
            } else {
                100.0 * ((minus_di - plus_di).abs() / sum_di)
            }
        };
        out_real[out_idx] = current_dx;
        prev_dx = current_dx;
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn adx_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    (2 * period) as i32 + context.get_unstable_period(FuncUnstId::Adx) as i32 - 1
}

pub(crate) fn adx_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let unstable = context.get_unstable_period(FuncUnstId::Adx) as usize;
    let lookback = (2 * period) + unstable - 1;
    let adjusted_start = start_idx.max(lookback);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut today = adjusted_start - lookback;
    let mut prev_high = in_high[today];
    let mut prev_low = in_low[today];
    let mut prev_close = in_close[today];
    let mut prev_minus_dm = 0.0;
    let mut prev_plus_dm = 0.0;
    let mut prev_tr = 0.0;

    for _ in 0..(period - 1) {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr += true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];
    }

    let mut sum_dx = 0.0;
    for _ in 0..period {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm -= prev_minus_dm / period as f64;
        prev_plus_dm -= prev_plus_dm / period as f64;
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];

        if !is_zero(prev_tr) {
            let minus_di = 100.0 * (prev_minus_dm / prev_tr);
            let plus_di = 100.0 * (prev_plus_dm / prev_tr);
            let sum_di = minus_di + plus_di;
            if !is_zero(sum_di) {
                sum_dx += 100.0 * ((minus_di - plus_di).abs() / sum_di);
            }
        }
    }

    let mut prev_adx = sum_dx / period as f64;

    for _ in 0..unstable {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm -= prev_minus_dm / period as f64;
        prev_plus_dm -= prev_plus_dm / period as f64;
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];

        if !is_zero(prev_tr) {
            let minus_di = 100.0 * (prev_minus_dm / prev_tr);
            let plus_di = 100.0 * (prev_plus_dm / prev_tr);
            let sum_di = minus_di + plus_di;
            if !is_zero(sum_di) {
                let dx = 100.0 * ((minus_di - plus_di).abs() / sum_di);
                prev_adx = ((prev_adx * (period as f64 - 1.0)) + dx) / period as f64;
            }
        }
    }

    out_real[0] = prev_adx;
    let mut out_idx = 1;
    while today < end_idx {
        today += 1;
        let high = in_high[today];
        let low = in_low[today];
        let (plus_dm, minus_dm) = dm_diff(prev_high, prev_low, high, low);
        prev_minus_dm -= prev_minus_dm / period as f64;
        prev_plus_dm -= prev_plus_dm / period as f64;
        prev_minus_dm += minus_dm;
        prev_plus_dm += plus_dm;
        prev_tr = prev_tr - (prev_tr / period as f64) + true_range(high, low, prev_close);
        prev_high = high;
        prev_low = low;
        prev_close = in_close[today];

        if !is_zero(prev_tr) {
            let minus_di = 100.0 * (prev_minus_dm / prev_tr);
            let plus_di = 100.0 * (prev_plus_dm / prev_tr);
            let sum_di = minus_di + plus_di;
            if !is_zero(sum_di) {
                let dx = 100.0 * ((minus_di - plus_di).abs() / sum_di);
                prev_adx = ((prev_adx * (period as f64 - 1.0)) + dx) / period as f64;
            }
        }

        out_real[out_idx] = prev_adx;
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn adxr_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32 + adx_lookback(context, period as i32) - 1
}

pub(crate) fn adxr_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_hlc(start_idx, end_idx, in_high, in_low, in_close) {
        return ret_code;
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let lookback = adxr_lookback(context, period as i32);
    if lookback < 0 {
        return RetCode::BadParam;
    }
    let adjusted_start = start_idx.max(lookback as usize);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let adx_start = adjusted_start - (period - 1);
    let mut adx_buf = vec![0.0; end_idx - adx_start + 1];
    let mut adx_beg_idx = 0usize;
    let mut adx_nb_element = 0usize;
    let ret_code = adx_run(
        context,
        adx_start,
        end_idx,
        in_high,
        in_low,
        in_close,
        period as i32,
        &mut adx_beg_idx,
        &mut adx_nb_element,
        &mut adx_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if adx_beg_idx != adx_start || adx_nb_element < needed + (period - 1) {
        return RetCode::InternalError;
    }

    for idx in 0..needed {
        out_real[idx] = (adx_buf[idx + (period - 1)] + adx_buf[idx]) / 2.0;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

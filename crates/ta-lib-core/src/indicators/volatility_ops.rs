use crate::helpers::{
    is_zero, normalize_period, validate_input_len, validate_output_len, validate_range,
};
use crate::{Context, FuncUnstId, RetCode};

pub(crate) fn atr_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return -1;
    };

    period as i32 + context.get_unstable_period(FuncUnstId::Atr) as i32
}

pub(crate) fn atr_run(
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

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_high, in_low, in_close] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return RetCode::BadParam;
    };

    let unstable_period = context.get_unstable_period(FuncUnstId::Atr) as usize;
    let lookback_total = period + unstable_period;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if period <= 1 {
        return crate::indicators::range_ops::trange_run(
            adjusted_start,
            end_idx,
            in_high,
            in_low,
            in_close,
            out_beg_idx,
            out_nb_element,
            out_real,
        );
    }

    let tr_start = adjusted_start - lookback_total + 1;
    let mut tr_buffer = vec![0.0; end_idx - tr_start + 1];
    let mut tr_beg_idx = 0usize;
    let mut tr_nb_element = 0usize;
    let ret_code = crate::indicators::range_ops::trange_run(
        tr_start,
        end_idx,
        in_high,
        in_low,
        in_close,
        &mut tr_beg_idx,
        &mut tr_nb_element,
        &mut tr_buffer,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }

    let mut prev_atr = tr_buffer[..period].iter().sum::<f64>() / period as f64;
    let mut tr_idx = period;
    for _ in 0..unstable_period {
        prev_atr = ((prev_atr * (period as f64 - 1.0)) + tr_buffer[tr_idx]) / period as f64;
        tr_idx += 1;
    }

    out_real[0] = prev_atr;
    for out_idx in 1..needed {
        prev_atr = ((prev_atr * (period as f64 - 1.0)) + tr_buffer[tr_idx]) / period as f64;
        tr_idx += 1;
        out_real[out_idx] = prev_atr;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn natr_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return -1;
    };

    period as i32 + context.get_unstable_period(FuncUnstId::Natr) as i32
}

pub(crate) fn natr_run(
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

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_high, in_low, in_close] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 1, 100_000) else {
        return RetCode::BadParam;
    };

    let unstable_period = context.get_unstable_period(FuncUnstId::Natr) as usize;
    let lookback_total = period + unstable_period;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if period <= 1 {
        let ret_code = crate::indicators::range_ops::trange_run(
            adjusted_start,
            end_idx,
            in_high,
            in_low,
            in_close,
            out_beg_idx,
            out_nb_element,
            out_real,
        );
        if ret_code != RetCode::Success {
            return ret_code;
        }
        for (offset, value) in out_real[..*out_nb_element].iter_mut().enumerate() {
            let close = in_close[*out_beg_idx + offset];
            *value = if !is_zero(close) {
                (*value / close) * 100.0
            } else {
                0.0
            };
        }
        return RetCode::Success;
    }

    let tr_start = adjusted_start - lookback_total + 1;
    let mut tr_buffer = vec![0.0; end_idx - tr_start + 1];
    let mut tr_beg_idx = 0usize;
    let mut tr_nb_element = 0usize;
    let ret_code = crate::indicators::range_ops::trange_run(
        tr_start,
        end_idx,
        in_high,
        in_low,
        in_close,
        &mut tr_beg_idx,
        &mut tr_nb_element,
        &mut tr_buffer,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }

    let mut prev_atr = tr_buffer[..period].iter().sum::<f64>() / period as f64;
    let mut tr_idx = period;
    for _ in 0..unstable_period {
        prev_atr = ((prev_atr * (period as f64 - 1.0)) + tr_buffer[tr_idx]) / period as f64;
        tr_idx += 1;
    }

    let mut today = adjusted_start;
    out_real[0] = if !is_zero(in_close[today]) {
        (prev_atr / in_close[today]) * 100.0
    } else {
        0.0
    };

    for out_idx in 1..needed {
        prev_atr = ((prev_atr * (period as f64 - 1.0)) + tr_buffer[tr_idx]) / period as f64;
        tr_idx += 1;
        today += 1;
        out_real[out_idx] = if !is_zero(in_close[today]) {
            (prev_atr / in_close[today]) * 100.0
        } else {
            0.0
        };
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

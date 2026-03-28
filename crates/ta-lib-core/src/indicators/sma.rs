use crate::RetCode;
use crate::helpers::{normalize_period, validate_input_len, validate_output_len, validate_range};

pub(crate) fn lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1
}

pub(crate) fn run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
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
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let lookback_total = period - 1;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut period_total = 0.0;
    let mut trailing_idx = adjusted_start - lookback_total;
    let mut today = trailing_idx;
    let mut remaining = period;

    while remaining > 1 {
        period_total += in_real[today];
        today += 1;
        remaining -= 1;
    }

    let mut out_idx = 0;
    while today <= end_idx {
        period_total += in_real[today];
        out_real[out_idx] = period_total / period as f64;
        period_total -= in_real[trailing_idx];

        out_idx += 1;
        trailing_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

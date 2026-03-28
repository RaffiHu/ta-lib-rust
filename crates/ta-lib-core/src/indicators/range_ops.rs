use crate::RetCode;
use crate::helpers::{normalize_period, validate_input_len, validate_output_len, validate_range};

pub(crate) fn midpoint_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1
}

pub(crate) fn midpoint_run(
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

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let trailing_idx = today - (period - 1);
        let mut lowest = in_real[trailing_idx];
        let mut highest = lowest;

        for &value in &in_real[trailing_idx + 1..=today] {
            if value < lowest {
                lowest = value;
            }
            if value > highest {
                highest = value;
            }
        }

        out_real[out_idx] = (highest + lowest) / 2.0;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn midprice_lookback(opt_in_time_period: i32) -> i32 {
    midpoint_lookback(opt_in_time_period)
}

pub(crate) fn midprice_run(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
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
    for input in [in_high, in_low] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let trailing_idx = today - (period - 1);
        let mut highest = in_high[trailing_idx];
        let mut lowest = in_low[trailing_idx];

        for &value in &in_high[trailing_idx + 1..=today] {
            if value > highest {
                highest = value;
            }
        }
        for &value in &in_low[trailing_idx + 1..=today] {
            if value < lowest {
                lowest = value;
            }
        }

        out_real[out_idx] = (highest + lowest) / 2.0;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn trange_lookback() -> i32 {
    1
}

pub(crate) fn trange_run(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
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

    let adjusted_start = start_idx.max(1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let high = in_high[today];
        let low = in_low[today];
        let prev_close = in_close[today - 1];
        let high_low = high - low;
        let high_close = (high - prev_close).abs();
        let low_close = (low - prev_close).abs();

        out_real[out_idx] = high_low.max(high_close).max(low_close);
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn avgdev_lookback(opt_in_time_period: i32) -> i32 {
    midpoint_lookback(opt_in_time_period)
}

pub(crate) fn avgdev_run(
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

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let period_f64 = period as f64;
    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let trailing_idx = today - (period - 1);
        let window = &in_real[trailing_idx..=today];
        let mean = window.iter().sum::<f64>() / period_f64;
        let avg_dev = window.iter().map(|value| (value - mean).abs()).sum::<f64>() / period_f64;
        out_real[out_idx] = avg_dev;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

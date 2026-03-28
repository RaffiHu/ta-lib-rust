use crate::RetCode;
use crate::helpers::{normalize_period, validate_input_len, validate_output_len, validate_range};

#[derive(Clone, Copy)]
pub(crate) enum ExtremumMode {
    Max,
    Min,
}

pub(crate) fn sum_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1
}

pub(crate) fn sum_run(
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

    let trailing_idx = adjusted_start - lookback_total;
    let mut i = trailing_idx;
    let mut period_total = 0.0;
    while i < adjusted_start {
        period_total += in_real[i];
        i += 1;
    }

    let mut out_idx = 0usize;
    while i <= end_idx {
        period_total += in_real[i];
        let temp_real = period_total;
        period_total -= in_real[i - lookback_total];
        out_real[out_idx] = temp_real;
        out_idx += 1;
        i += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn extrema_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1
}

pub(crate) fn extrema_values_run(
    mode: ExtremumMode,
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

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut out_idx = 0usize;
    let mut today = adjusted_start;
    let mut trailing_idx = adjusted_start - (period - 1);
    let mut best_idx: Option<usize> = None;
    let mut best = 0.0;

    while today <= end_idx {
        let tmp = in_real[today];

        if best_idx.is_none_or(|index| index < trailing_idx) {
            best_idx = Some(trailing_idx);
            best = in_real[trailing_idx];
            let mut i = trailing_idx + 1;
            while i <= today {
                let candidate = in_real[i];
                if better(mode, candidate, best) {
                    best_idx = Some(i);
                    best = candidate;
                }
                i += 1;
            }
        } else if better_or_equal(mode, tmp, best) {
            best_idx = Some(today);
            best = tmp;
        }

        out_real[out_idx] = best;
        out_idx += 1;
        trailing_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn extrema_indices_run(
    mode: ExtremumMode,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
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

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    if out_integer.len() < end_idx - adjusted_start + 1 {
        return RetCode::BadParam;
    }

    let mut out_idx = 0usize;
    let mut today = adjusted_start;
    let mut trailing_idx = adjusted_start - (period - 1);
    let mut best_idx: Option<usize> = None;
    let mut best = 0.0;

    while today <= end_idx {
        let tmp = in_real[today];

        if best_idx.is_none_or(|index| index < trailing_idx) {
            best_idx = Some(trailing_idx);
            best = in_real[trailing_idx];
            let mut i = trailing_idx + 1;
            while i <= today {
                let candidate = in_real[i];
                if better(mode, candidate, best) {
                    best_idx = Some(i);
                    best = candidate;
                }
                i += 1;
            }
        } else if better_or_equal(mode, tmp, best) {
            best_idx = Some(today);
            best = tmp;
        }

        out_integer[out_idx] = i32::try_from(best_idx.expect("best index")).unwrap_or(i32::MAX);
        out_idx += 1;
        trailing_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn minmax_values_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_min: &mut [f64],
    out_max: &mut [f64],
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

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_min, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_max, needed) {
        return ret_code;
    }

    let mut out_idx = 0usize;
    let mut today = adjusted_start;
    let mut trailing_idx = adjusted_start - (period - 1);
    let mut highest_idx: Option<usize> = None;
    let mut highest = 0.0;
    let mut lowest_idx: Option<usize> = None;
    let mut lowest = 0.0;

    while today <= end_idx {
        let tmp = in_real[today];

        if highest_idx.is_none_or(|index| index < trailing_idx) {
            highest_idx = Some(trailing_idx);
            highest = in_real[trailing_idx];
            let mut i = trailing_idx + 1;
            while i <= today {
                let candidate = in_real[i];
                if candidate > highest {
                    highest_idx = Some(i);
                    highest = candidate;
                }
                i += 1;
            }
        } else if tmp >= highest {
            highest_idx = Some(today);
            highest = tmp;
        }

        if lowest_idx.is_none_or(|index| index < trailing_idx) {
            lowest_idx = Some(trailing_idx);
            lowest = in_real[trailing_idx];
            let mut i = trailing_idx + 1;
            while i <= today {
                let candidate = in_real[i];
                if candidate < lowest {
                    lowest_idx = Some(i);
                    lowest = candidate;
                }
                i += 1;
            }
        } else if tmp <= lowest {
            lowest_idx = Some(today);
            lowest = tmp;
        }

        out_min[out_idx] = lowest;
        out_max[out_idx] = highest;
        out_idx += 1;
        trailing_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn minmax_indices_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_min_idx: &mut [i32],
    out_max_idx: &mut [i32],
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

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if out_min_idx.len() < needed || out_max_idx.len() < needed {
        return RetCode::BadParam;
    }

    let mut out_idx = 0usize;
    let mut today = adjusted_start;
    let mut trailing_idx = adjusted_start - (period - 1);
    let mut highest_idx: Option<usize> = None;
    let mut highest = 0.0;
    let mut lowest_idx: Option<usize> = None;
    let mut lowest = 0.0;

    while today <= end_idx {
        let tmp = in_real[today];

        if highest_idx.is_none_or(|index| index < trailing_idx) {
            highest_idx = Some(trailing_idx);
            highest = in_real[trailing_idx];
            let mut i = trailing_idx + 1;
            while i <= today {
                let candidate = in_real[i];
                if candidate > highest {
                    highest_idx = Some(i);
                    highest = candidate;
                }
                i += 1;
            }
        } else if tmp >= highest {
            highest_idx = Some(today);
            highest = tmp;
        }

        if lowest_idx.is_none_or(|index| index < trailing_idx) {
            lowest_idx = Some(trailing_idx);
            lowest = in_real[trailing_idx];
            let mut i = trailing_idx + 1;
            while i <= today {
                let candidate = in_real[i];
                if candidate < lowest {
                    lowest_idx = Some(i);
                    lowest = candidate;
                }
                i += 1;
            }
        } else if tmp <= lowest {
            lowest_idx = Some(today);
            lowest = tmp;
        }

        out_min_idx[out_idx] = i32::try_from(lowest_idx.expect("lowest index")).unwrap_or(i32::MAX);
        out_max_idx[out_idx] =
            i32::try_from(highest_idx.expect("highest index")).unwrap_or(i32::MAX);
        out_idx += 1;
        trailing_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

fn better(mode: ExtremumMode, lhs: f64, rhs: f64) -> bool {
    match mode {
        ExtremumMode::Max => lhs > rhs,
        ExtremumMode::Min => lhs < rhs,
    }
}

fn better_or_equal(mode: ExtremumMode, lhs: f64, rhs: f64) -> bool {
    match mode {
        ExtremumMode::Max => lhs >= rhs,
        ExtremumMode::Min => lhs <= rhs,
    }
}

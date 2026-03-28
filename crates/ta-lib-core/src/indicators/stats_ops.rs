use crate::RetCode;
use crate::helpers::{
    REAL_DEFAULT, is_zero, normalize_period, validate_input_len, validate_output_len,
    validate_range,
};

#[derive(Clone, Copy)]
struct LinearRegressionCoefficients {
    slope: f64,
    intercept: f64,
}

fn normalize_nb_dev(value: f64) -> Result<f64, RetCode> {
    if value == REAL_DEFAULT {
        Ok(1.0)
    } else if !(-3.0e37..=3.0e37).contains(&value) {
        Err(RetCode::BadParam)
    } else {
        Ok(value)
    }
}

pub(crate) fn var_lookback(opt_in_time_period: i32, opt_in_nb_dev: f64) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 5, 1, 100_000) else {
        return -1;
    };
    if normalize_nb_dev(opt_in_nb_dev).is_err() {
        return -1;
    }
    period as i32 - 1
}

pub(crate) fn var_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    opt_in_nb_dev: f64,
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

    let Ok(period) = normalize_period(opt_in_time_period, 5, 1, 100_000) else {
        return RetCode::BadParam;
    };
    if normalize_nb_dev(opt_in_nb_dev).is_err() {
        return RetCode::BadParam;
    }

    let adjusted_start = start_idx.max(period - 1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut sum = 0.0;
    let mut sum_sq = 0.0;
    let trailing_idx = adjusted_start - (period - 1);
    for &value in &in_real[trailing_idx..adjusted_start] {
        sum += value;
        sum_sq += value * value;
    }

    let period_f64 = period as f64;
    let mut i = adjusted_start;
    let mut out_idx = 0usize;
    while i <= end_idx {
        let value = in_real[i];
        sum += value;
        sum_sq += value * value;

        let mean = sum / period_f64;
        let variance = (sum_sq / period_f64) - (mean * mean);
        out_real[out_idx] = variance.max(0.0);
        out_idx += 1;

        let trailing = in_real[i - (period - 1)];
        sum -= trailing;
        sum_sq -= trailing * trailing;
        i += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn stddev_lookback(opt_in_time_period: i32, opt_in_nb_dev: f64) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 5, 2, 100_000) else {
        return -1;
    };
    if normalize_nb_dev(opt_in_nb_dev).is_err() {
        return -1;
    }
    period as i32 - 1
}

pub(crate) fn stddev_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    opt_in_nb_dev: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    let Ok(nb_dev) = normalize_nb_dev(opt_in_nb_dev) else {
        return RetCode::BadParam;
    };
    let ret_code = var_run(
        start_idx,
        end_idx,
        in_real,
        opt_in_time_period,
        nb_dev,
        out_beg_idx,
        out_nb_element,
        out_real,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }

    for value in &mut out_real[..*out_nb_element] {
        if *value > 0.0 && !is_zero(*value) {
            *value = value.sqrt() * nb_dev;
        } else {
            *value = 0.0;
        }
    }

    RetCode::Success
}

pub(crate) fn correl_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 1, 100_000) else {
        return -1;
    };
    period as i32 - 1
}

pub(crate) fn correl_run(
    start_idx: usize,
    end_idx: usize,
    in_real0: &[f64],
    in_real1: &[f64],
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
    for input in [in_real0, in_real1] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 30, 1, 100_000) else {
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

    let trailing_idx = adjusted_start - (period - 1);
    let mut sum_xy = 0.0;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;
    for i in trailing_idx..=adjusted_start {
        let x = in_real0[i];
        let y = in_real1[i];
        sum_xy += x * y;
        sum_x += x;
        sum_y += y;
        sum_x2 += x * x;
        sum_y2 += y * y;
    }

    let period_f64 = period as f64;
    let mut today = adjusted_start + 1;
    let mut trailing = trailing_idx;
    let mut out_idx = 0usize;
    loop {
        let denominator =
            (sum_x2 - ((sum_x * sum_x) / period_f64)) * (sum_y2 - ((sum_y * sum_y) / period_f64));
        out_real[out_idx] = if denominator > 0.0 && !is_zero(denominator) {
            (sum_xy - ((sum_x * sum_y) / period_f64)) / denominator.sqrt()
        } else {
            0.0
        };
        out_idx += 1;

        if today > end_idx {
            break;
        }

        let tx = in_real0[trailing];
        let ty = in_real1[trailing];
        sum_x -= tx;
        sum_x2 -= tx * tx;
        sum_xy -= tx * ty;
        sum_y -= ty;
        sum_y2 -= ty * ty;

        let x = in_real0[today];
        let y = in_real1[today];
        sum_x += x;
        sum_x2 += x * x;
        sum_xy += x * y;
        sum_y += y;
        sum_y2 += y * y;

        trailing += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn beta_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 5, 1, 100_000) else {
        return -1;
    };
    period as i32
}

pub(crate) fn beta_run(
    start_idx: usize,
    end_idx: usize,
    in_real0: &[f64],
    in_real1: &[f64],
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
    for input in [in_real0, in_real1] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 5, 1, 100_000) else {
        return RetCode::BadParam;
    };
    let adjusted_start = start_idx.max(period);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut trailing_idx = adjusted_start - period;
    let mut last_price_x = in_real0[trailing_idx];
    let mut last_price_y = in_real1[trailing_idx];
    let mut trailing_last_price_x = last_price_x;
    let mut trailing_last_price_y = last_price_y;

    let mut s_xx = 0.0;
    let mut s_xy = 0.0;
    let mut s_x = 0.0;
    let mut s_y = 0.0;

    trailing_idx += 1;
    let mut i = trailing_idx;
    while i < adjusted_start {
        let tmp_x = in_real0[i];
        let x = if !is_zero(last_price_x) {
            (tmp_x - last_price_x) / last_price_x
        } else {
            0.0
        };
        last_price_x = tmp_x;

        let tmp_y = in_real1[i];
        let y = if !is_zero(last_price_y) {
            (tmp_y - last_price_y) / last_price_y
        } else {
            0.0
        };
        last_price_y = tmp_y;

        s_xx += x * x;
        s_xy += x * y;
        s_x += x;
        s_y += y;
        i += 1;
    }

    let n = period as f64;
    let mut out_idx = 0usize;
    while i <= end_idx {
        let tmp_x = in_real0[i];
        let x = if !is_zero(last_price_x) {
            (tmp_x - last_price_x) / last_price_x
        } else {
            0.0
        };
        last_price_x = tmp_x;

        let tmp_y = in_real1[i];
        let y = if !is_zero(last_price_y) {
            (tmp_y - last_price_y) / last_price_y
        } else {
            0.0
        };
        last_price_y = tmp_y;

        s_xx += x * x;
        s_xy += x * y;
        s_x += x;
        s_y += y;

        let trailing_tmp_x = in_real0[trailing_idx];
        let trailing_x = if !is_zero(trailing_last_price_x) {
            (trailing_tmp_x - trailing_last_price_x) / trailing_last_price_x
        } else {
            0.0
        };
        trailing_last_price_x = trailing_tmp_x;

        let trailing_tmp_y = in_real1[trailing_idx];
        let trailing_y = if !is_zero(trailing_last_price_y) {
            (trailing_tmp_y - trailing_last_price_y) / trailing_last_price_y
        } else {
            0.0
        };
        trailing_last_price_y = trailing_tmp_y;
        trailing_idx += 1;

        let denominator = (n * s_xx) - (s_x * s_x);
        out_real[out_idx] = if !is_zero(denominator) {
            ((n * s_xy) - (s_x * s_y)) / denominator
        } else {
            0.0
        };
        out_idx += 1;

        s_xx -= trailing_x * trailing_x;
        s_xy -= trailing_x * trailing_y;
        s_x -= trailing_x;
        s_y -= trailing_y;

        i += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

fn linearreg_coefficients(
    window: &[f64],
    sum_x: f64,
    divisor: f64,
) -> LinearRegressionCoefficients {
    let mut sum_xy = 0.0;
    let mut sum_y = 0.0;
    let last_x = window.len().saturating_sub(1);

    for (index, value) in window.iter().enumerate() {
        let x = (last_x - index) as f64;
        sum_y += *value;
        sum_xy += x * *value;
    }

    let period = window.len() as f64;
    let slope = ((period * sum_xy) - (sum_x * sum_y)) / divisor;
    let intercept = (sum_y - (slope * sum_x)) / period;

    LinearRegressionCoefficients { slope, intercept }
}

fn linearreg_run_impl<F>(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
    map_output: F,
) -> RetCode
where
    F: Fn(LinearRegressionCoefficients, usize) -> f64,
{
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
    let lookback_total = period - 1;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let period_f64 = period as f64;
    let sum_x = period_f64 * (period_f64 - 1.0) * 0.5;
    let sum_x_sqr = period_f64 * (period_f64 - 1.0) * ((2.0 * period_f64) - 1.0) / 6.0;
    let divisor = (sum_x * sum_x) - (period_f64 * sum_x_sqr);

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let window_start = today + 1 - period;
        let coeffs = linearreg_coefficients(&in_real[window_start..=today], sum_x, divisor);
        out_real[out_idx] = map_output(coeffs, period);
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn linearreg_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };
    period as i32 - 1
}

pub(crate) fn linearreg_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    linearreg_run_impl(
        start_idx,
        end_idx,
        in_real,
        opt_in_time_period,
        out_beg_idx,
        out_nb_element,
        out_real,
        |coeffs, period| coeffs.intercept + (coeffs.slope * (period as f64 - 1.0)),
    )
}

pub(crate) fn linearreg_angle_lookback(opt_in_time_period: i32) -> i32 {
    linearreg_lookback(opt_in_time_period)
}

pub(crate) fn linearreg_angle_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    linearreg_run_impl(
        start_idx,
        end_idx,
        in_real,
        opt_in_time_period,
        out_beg_idx,
        out_nb_element,
        out_real,
        |coeffs, _| coeffs.slope.atan().to_degrees(),
    )
}

pub(crate) fn linearreg_intercept_lookback(opt_in_time_period: i32) -> i32 {
    linearreg_lookback(opt_in_time_period)
}

pub(crate) fn linearreg_intercept_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    linearreg_run_impl(
        start_idx,
        end_idx,
        in_real,
        opt_in_time_period,
        out_beg_idx,
        out_nb_element,
        out_real,
        |coeffs, _| coeffs.intercept,
    )
}

pub(crate) fn linearreg_slope_lookback(opt_in_time_period: i32) -> i32 {
    linearreg_lookback(opt_in_time_period)
}

pub(crate) fn linearreg_slope_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    linearreg_run_impl(
        start_idx,
        end_idx,
        in_real,
        opt_in_time_period,
        out_beg_idx,
        out_nb_element,
        out_real,
        |coeffs, _| coeffs.slope,
    )
}

pub(crate) fn tsf_lookback(opt_in_time_period: i32) -> i32 {
    linearreg_lookback(opt_in_time_period)
}

pub(crate) fn tsf_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    linearreg_run_impl(
        start_idx,
        end_idx,
        in_real,
        opt_in_time_period,
        out_beg_idx,
        out_nb_element,
        out_real,
        |coeffs, period| coeffs.intercept + (coeffs.slope * period as f64),
    )
}

use alloc::vec::Vec;

use crate::helpers::{
    INTEGER_DEFAULT, REAL_DEFAULT, is_zero, normalize_period, validate_input_len,
    validate_output_len, validate_range,
};
use crate::indicators::{ema, sma, stats_ops};
use crate::{Context, MAType, RetCode};

pub(crate) fn normalize_ma_type(value: i32) -> Result<MAType, RetCode> {
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

pub(crate) fn ma_lookback_dispatch(
    context: &Context,
    opt_in_time_period: i32,
    opt_in_ma_type: i32,
) -> Result<i32, RetCode> {
    let period = if opt_in_time_period == INTEGER_DEFAULT {
        30
    } else {
        opt_in_time_period
    };

    if period < 1 || period > 100_000 {
        return Err(RetCode::BadParam);
    }
    if period <= 1 {
        return Ok(0);
    }

    let ma_type = normalize_ma_type(opt_in_ma_type)?;
    match ma_type {
        MAType::Sma => Ok(sma::lookback(period)),
        MAType::Ema => Ok(ema::lookback(context, period)),
        MAType::Wma => Ok(wma_lookback(period)),
        MAType::Dema => Ok(dema_lookback(context, period)),
        MAType::Tema => Ok(tema_lookback(context, period)),
        MAType::Trima => Ok(trima_lookback(period)),
        MAType::Kama => Ok(kama_lookback(context, period)),
        MAType::Mama => Ok(mama_lookback(context, REAL_DEFAULT, REAL_DEFAULT)),
        MAType::T3 => Ok(t3_lookback(context, period, REAL_DEFAULT)),
    }
}

pub(crate) fn ma_run_dispatch(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    opt_in_ma_type: i32,
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

    let period = if opt_in_time_period == INTEGER_DEFAULT {
        30
    } else {
        opt_in_time_period
    };
    if period < 1 || period > 100_000 {
        return RetCode::BadParam;
    }

    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if period <= 1 {
        out_real[..needed].copy_from_slice(&in_real[start_idx..=end_idx]);
        *out_beg_idx = start_idx;
        *out_nb_element = needed;
        return RetCode::Success;
    }

    let ma_type = match normalize_ma_type(opt_in_ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
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
        MAType::Wma => wma_run(
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Dema => dema_run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Tema => tema_run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Trima => trima_run(
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Kama => kama_run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
        MAType::Mama => {
            let mut out_fama = vec![0.0; needed];
            mama_run(
                context,
                start_idx,
                end_idx,
                in_real,
                REAL_DEFAULT,
                REAL_DEFAULT,
                out_beg_idx,
                out_nb_element,
                out_real,
                &mut out_fama,
            )
        }
        MAType::T3 => t3_run(
            context,
            start_idx,
            end_idx,
            in_real,
            period,
            REAL_DEFAULT,
            out_beg_idx,
            out_nb_element,
            out_real,
        ),
    }
}

fn bbands_normalize_nb_dev(value: f64) -> Result<f64, RetCode> {
    if value == REAL_DEFAULT {
        Ok(2.0)
    } else if !(-3.0e37..=3.0e37).contains(&value) {
        Err(RetCode::BadParam)
    } else {
        Ok(value)
    }
}

fn stddev_using_precalc_ma(
    in_real: &[f64],
    middle_band: &[f64],
    out_beg_idx: usize,
    out_nb_element: usize,
    opt_in_time_period: usize,
    out_stddev: &mut [f64],
) {
    for out_idx in 0..out_nb_element {
        let today = out_beg_idx + out_idx;
        let window_start = today + 1 - opt_in_time_period;
        let average = middle_band[out_idx];
        let mut sum = 0.0;
        for value in &in_real[window_start..=today] {
            let delta = *value - average;
            sum += delta * delta;
        }
        let variance = sum / opt_in_time_period as f64;
        out_stddev[out_idx] = if variance > 0.0 && !is_zero(variance) {
            variance.sqrt()
        } else {
            0.0
        };
    }
}

pub(crate) fn bbands_lookback(
    context: &Context,
    opt_in_time_period: i32,
    opt_in_nb_dev_up: f64,
    opt_in_nb_dev_dn: f64,
    opt_in_ma_type: i32,
) -> i32 {
    if bbands_normalize_nb_dev(opt_in_nb_dev_up).is_err()
        || bbands_normalize_nb_dev(opt_in_nb_dev_dn).is_err()
    {
        return -1;
    }

    match ma_lookback_dispatch(context, opt_in_time_period, opt_in_ma_type) {
        Ok(value) => value,
        Err(_) => -1,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn bbands_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let nb_dev_up = match bbands_normalize_nb_dev(opt_in_nb_dev_up) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let nb_dev_dn = match bbands_normalize_nb_dev(opt_in_nb_dev_dn) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let ma_type = match normalize_ma_type(opt_in_ma_type) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let period = if opt_in_time_period == INTEGER_DEFAULT {
        5
    } else {
        opt_in_time_period
    };
    if !(2..=100_000).contains(&period) {
        return RetCode::BadParam;
    }

    let lookback_total = match ma_lookback_dispatch(context, period, opt_in_ma_type) {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real_upper_band, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_real_middle_band, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_real_lower_band, needed) {
        return ret_code;
    }

    let ret_code = ma_run_dispatch(
        context,
        start_idx,
        end_idx,
        in_real,
        period,
        opt_in_ma_type,
        out_beg_idx,
        out_nb_element,
        out_real_middle_band,
    );
    if ret_code != RetCode::Success || *out_nb_element == 0 {
        *out_nb_element = 0;
        return ret_code;
    }

    let mut stddev = vec![0.0; *out_nb_element];
    if ma_type == MAType::Sma {
        stddev_using_precalc_ma(
            in_real,
            &out_real_middle_band[..*out_nb_element],
            *out_beg_idx,
            *out_nb_element,
            period as usize,
            &mut stddev,
        );
    } else {
        let ret_code = stats_ops::stddev_run(
            *out_beg_idx,
            end_idx,
            in_real,
            period,
            1.0,
            out_beg_idx,
            out_nb_element,
            &mut stddev,
        );
        if ret_code != RetCode::Success {
            *out_nb_element = 0;
            return ret_code;
        }
    }

    for idx in 0..*out_nb_element {
        let middle = out_real_middle_band[idx];
        let deviation = stddev[idx];
        out_real_upper_band[idx] = middle + (deviation * nb_dev_up);
        out_real_lower_band[idx] = middle - (deviation * nb_dev_dn);
    }

    RetCode::Success
}

pub(crate) fn accbands_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 20, 2, 100_000) else {
        return -1;
    };
    sma::lookback(period as i32)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn accbands_run(
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

    let Ok(period) = normalize_period(opt_in_time_period, 20, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let lookback_total = sma::lookback(period as i32) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let output_size = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real_upper_band, output_size) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_real_middle_band, output_size) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_real_lower_band, output_size) {
        return ret_code;
    }

    let buffer_start = adjusted_start - lookback_total;
    let buffer_size = output_size + lookback_total;
    let mut upper_input = vec![0.0; buffer_size];
    let mut lower_input = vec![0.0; buffer_size];

    for (idx, today) in (buffer_start..=end_idx).enumerate() {
        let sum = in_high[today] + in_low[today];
        if !is_zero(sum) {
            let temp = 4.0 * (in_high[today] - in_low[today]) / sum;
            upper_input[idx] = in_high[today] * (1.0 + temp);
            lower_input[idx] = in_low[today] * (1.0 - temp);
        } else {
            upper_input[idx] = in_high[today];
            lower_input[idx] = in_low[today];
        }
    }

    let mut dummy_beg = 0usize;
    let mut dummy_nb = 0usize;
    let ret_code = sma::run(
        adjusted_start,
        end_idx,
        in_close,
        period as i32,
        &mut dummy_beg,
        &mut dummy_nb,
        out_real_middle_band,
    );
    if ret_code != RetCode::Success || dummy_nb != output_size {
        *out_nb_element = 0;
        return ret_code;
    }

    let ret_code = sma::run(
        0,
        buffer_size - 1,
        &upper_input,
        period as i32,
        &mut dummy_beg,
        &mut dummy_nb,
        out_real_upper_band,
    );
    if ret_code != RetCode::Success || dummy_nb != output_size {
        *out_nb_element = 0;
        return ret_code;
    }

    let ret_code = sma::run(
        0,
        buffer_size - 1,
        &lower_input,
        period as i32,
        &mut dummy_beg,
        &mut dummy_nb,
        out_real_lower_band,
    );
    if ret_code != RetCode::Success || dummy_nb != output_size {
        *out_nb_element = 0;
        return ret_code;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = output_size;
    RetCode::Success
}

pub(crate) fn mavp_lookback(
    context: &Context,
    opt_in_min_period: i32,
    opt_in_max_period: i32,
    opt_in_ma_type: i32,
) -> i32 {
    let Ok(_min_period) = normalize_period(opt_in_min_period, 2, 2, 100_000) else {
        return -1;
    };
    let Ok(max_period) = normalize_period(opt_in_max_period, 30, 2, 100_000) else {
        return -1;
    };
    match ma_lookback_dispatch(context, max_period as i32, opt_in_ma_type) {
        Ok(value) => value,
        Err(_) => -1,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn mavp_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_real, in_periods] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(min_period) = normalize_period(opt_in_min_period, 2, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(max_period) = normalize_period(opt_in_max_period, 30, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let lookback_total = match ma_lookback_dispatch(context, max_period as i32, opt_in_ma_type) {
        Ok(value) => value as usize,
        Err(ret_code) => return ret_code,
    };
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let output_size = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, output_size) {
        return ret_code;
    }

    let mut local_periods = vec![0usize; output_size];
    for (idx, period_value) in in_periods[adjusted_start..=end_idx].iter().enumerate() {
        let mut period = *period_value as i32;
        if period < min_period as i32 {
            period = min_period as i32;
        } else if period > max_period as i32 {
            period = max_period as i32;
        }
        local_periods[idx] = period as usize;
    }

    let mut local_output = vec![0.0; output_size];
    for i in 0..output_size {
        let cur_period = local_periods[i];
        if cur_period == 0 {
            continue;
        }

        let mut local_beg = 0usize;
        let mut local_nb = 0usize;
        let ret_code = ma_run_dispatch(
            context,
            adjusted_start,
            end_idx,
            in_real,
            cur_period as i32,
            opt_in_ma_type,
            &mut local_beg,
            &mut local_nb,
            &mut local_output,
        );
        if ret_code != RetCode::Success {
            *out_nb_element = 0;
            return ret_code;
        }
        if local_beg != adjusted_start || local_nb != output_size {
            return RetCode::InternalError;
        }

        out_real[i] = local_output[i];
        for j in i + 1..output_size {
            if local_periods[j] == cur_period {
                local_periods[j] = 0;
                out_real[j] = local_output[j];
            }
        }
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = output_size;
    RetCode::Success
}

#[derive(Clone, Copy)]
struct SarConfig {
    start_value: f64,
    offset_on_reverse: f64,
    acceleration_init_long: f64,
    acceleration_long: f64,
    acceleration_max_long: f64,
    acceleration_init_short: f64,
    acceleration_short: f64,
    acceleration_max_short: f64,
    extended: bool,
}

fn normalize_nonnegative_real(value: f64, default_value: f64) -> Result<f64, RetCode> {
    if value == REAL_DEFAULT {
        Ok(default_value)
    } else if !(0.0..=3.0e37).contains(&value) {
        Err(RetCode::BadParam)
    } else {
        Ok(value)
    }
}

fn normalize_any_real(value: f64, default_value: f64) -> Result<f64, RetCode> {
    if value == REAL_DEFAULT {
        Ok(default_value)
    } else if !(-3.0e37..=3.0e37).contains(&value) {
        Err(RetCode::BadParam)
    } else {
        Ok(value)
    }
}

fn initial_sar_direction(in_high: &[f64], in_low: &[f64], start_idx: usize) -> bool {
    let plus_dm = in_high[start_idx] - in_high[start_idx - 1];
    let minus_dm = in_low[start_idx - 1] - in_low[start_idx];
    !(minus_dm > 0.0 && minus_dm > plus_dm)
}

fn sar_run_internal(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
    mut config: SarConfig,
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

    let adjusted_start = start_idx.max(1);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if config.acceleration_init_long > config.acceleration_max_long {
        config.acceleration_init_long = config.acceleration_max_long;
    }
    if config.acceleration_long > config.acceleration_max_long {
        config.acceleration_long = config.acceleration_max_long;
    }
    if config.acceleration_init_short > config.acceleration_max_short {
        config.acceleration_init_short = config.acceleration_max_short;
    }
    if config.acceleration_short > config.acceleration_max_short {
        config.acceleration_short = config.acceleration_max_short;
    }

    let mut is_long = if is_zero(config.start_value) {
        initial_sar_direction(in_high, in_low, adjusted_start)
    } else {
        config.start_value > 0.0
    };

    let mut today_idx = adjusted_start;
    let mut new_high = in_high[today_idx - 1];
    let mut new_low = in_low[today_idx - 1];

    let mut ep;
    let mut sar;
    if is_zero(config.start_value) {
        if is_long {
            ep = in_high[today_idx];
            sar = new_low;
        } else {
            ep = in_low[today_idx];
            sar = new_high;
        }
    } else if config.start_value > 0.0 {
        ep = in_high[today_idx];
        sar = config.start_value;
    } else {
        ep = in_low[today_idx];
        sar = config.start_value.abs();
    }

    let mut af_long = config.acceleration_init_long;
    let mut af_short = config.acceleration_init_short;

    new_low = in_low[today_idx];
    new_high = in_high[today_idx];

    let mut out_idx = 0usize;
    while today_idx <= end_idx {
        let prev_low = new_low;
        let prev_high = new_high;
        new_low = in_low[today_idx];
        new_high = in_high[today_idx];
        today_idx += 1;

        if is_long {
            if new_low <= sar {
                is_long = false;
                sar = ep;
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
                let mut output = sar;
                if config.extended && !is_zero(config.offset_on_reverse) {
                    output += output * config.offset_on_reverse;
                }
                out_real[out_idx] = if config.extended { -output } else { output };
                out_idx += 1;

                af_short = config.acceleration_init_short;
                ep = new_low;
                sar = sar + af_short * (ep - sar);
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
            } else {
                out_real[out_idx] = sar;
                out_idx += 1;

                if new_high > ep {
                    ep = new_high;
                    af_long += config.acceleration_long;
                    if af_long > config.acceleration_max_long {
                        af_long = config.acceleration_max_long;
                    }
                }
                sar = sar + af_long * (ep - sar);
                if sar > prev_low {
                    sar = prev_low;
                }
                if sar > new_low {
                    sar = new_low;
                }
            }
        } else if new_high >= sar {
            is_long = true;
            sar = ep;
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
            let mut output = sar;
            if config.extended && !is_zero(config.offset_on_reverse) {
                output -= output * config.offset_on_reverse;
            }
            out_real[out_idx] = output;
            out_idx += 1;

            af_long = config.acceleration_init_long;
            ep = new_high;
            sar = sar + af_long * (ep - sar);
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
        } else {
            out_real[out_idx] = if config.extended { -sar } else { sar };
            out_idx += 1;

            if new_low < ep {
                ep = new_low;
                af_short += config.acceleration_short;
                if af_short > config.acceleration_max_short {
                    af_short = config.acceleration_max_short;
                }
            }
            sar = sar + af_short * (ep - sar);
            if sar < prev_high {
                sar = prev_high;
            }
            if sar < new_high {
                sar = new_high;
            }
        }
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn sar_lookback(opt_in_acceleration: f64, opt_in_maximum: f64) -> i32 {
    if normalize_nonnegative_real(opt_in_acceleration, 0.02).is_err()
        || normalize_nonnegative_real(opt_in_maximum, 0.2).is_err()
    {
        return -1;
    }
    1
}

pub(crate) fn sar_run(
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
    let opt_in_acceleration = match normalize_nonnegative_real(opt_in_acceleration, 0.02) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let opt_in_maximum = match normalize_nonnegative_real(opt_in_maximum, 0.2) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };

    sar_run_internal(
        start_idx,
        end_idx,
        in_high,
        in_low,
        out_beg_idx,
        out_nb_element,
        out_real,
        SarConfig {
            start_value: 0.0,
            offset_on_reverse: 0.0,
            acceleration_init_long: opt_in_acceleration,
            acceleration_long: opt_in_acceleration,
            acceleration_max_long: opt_in_maximum,
            acceleration_init_short: opt_in_acceleration,
            acceleration_short: opt_in_acceleration,
            acceleration_max_short: opt_in_maximum,
            extended: false,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sarext_lookback(
    opt_in_start_value: f64,
    opt_in_offset_on_reverse: f64,
    opt_in_acceleration_init_long: f64,
    opt_in_acceleration_long: f64,
    opt_in_acceleration_max_long: f64,
    opt_in_acceleration_init_short: f64,
    opt_in_acceleration_short: f64,
    opt_in_acceleration_max_short: f64,
) -> i32 {
    if normalize_any_real(opt_in_start_value, 0.0).is_err()
        || normalize_nonnegative_real(opt_in_offset_on_reverse, 0.0).is_err()
        || normalize_nonnegative_real(opt_in_acceleration_init_long, 0.02).is_err()
        || normalize_nonnegative_real(opt_in_acceleration_long, 0.02).is_err()
        || normalize_nonnegative_real(opt_in_acceleration_max_long, 0.2).is_err()
        || normalize_nonnegative_real(opt_in_acceleration_init_short, 0.02).is_err()
        || normalize_nonnegative_real(opt_in_acceleration_short, 0.02).is_err()
        || normalize_nonnegative_real(opt_in_acceleration_max_short, 0.2).is_err()
    {
        return -1;
    }
    1
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn sarext_run(
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
    let start_value = match normalize_any_real(opt_in_start_value, 0.0) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let offset_on_reverse = match normalize_nonnegative_real(opt_in_offset_on_reverse, 0.0) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let acceleration_init_long =
        match normalize_nonnegative_real(opt_in_acceleration_init_long, 0.02) {
            Ok(value) => value,
            Err(ret_code) => return ret_code,
        };
    let acceleration_long = match normalize_nonnegative_real(opt_in_acceleration_long, 0.02) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let acceleration_max_long = match normalize_nonnegative_real(opt_in_acceleration_max_long, 0.2)
    {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let acceleration_init_short =
        match normalize_nonnegative_real(opt_in_acceleration_init_short, 0.02) {
            Ok(value) => value,
            Err(ret_code) => return ret_code,
        };
    let acceleration_short = match normalize_nonnegative_real(opt_in_acceleration_short, 0.02) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let acceleration_max_short =
        match normalize_nonnegative_real(opt_in_acceleration_max_short, 0.2) {
            Ok(value) => value,
            Err(ret_code) => return ret_code,
        };

    sar_run_internal(
        start_idx,
        end_idx,
        in_high,
        in_low,
        out_beg_idx,
        out_nb_element,
        out_real,
        SarConfig {
            start_value,
            offset_on_reverse,
            acceleration_init_long,
            acceleration_long,
            acceleration_max_long,
            acceleration_init_short,
            acceleration_short,
            acceleration_max_short,
            extended: true,
        },
    )
}

pub(crate) fn kama_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    period as i32 + context.get_unstable_period(crate::FuncUnstId::Kama) as i32
}

pub(crate) fn kama_run(
    context: &Context,
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
    let unstable = context.get_unstable_period(crate::FuncUnstId::Kama) as usize;
    let lookback_total = period + unstable;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let const_max = 2.0 / 31.0;
    let const_diff = (2.0 / 3.0) - const_max;

    let mut sum_roc1 = 0.0;
    let mut today = adjusted_start - lookback_total;
    let mut trailing_idx = today;
    for _ in 0..period {
        sum_roc1 += (in_real[today] - in_real[today + 1]).abs();
        today += 1;
    }

    let mut prev_kama = in_real[today - 1];
    let mut temp_real = in_real[today];
    let mut temp_real2 = in_real[trailing_idx];
    trailing_idx += 1;
    let mut period_roc = temp_real - temp_real2;
    let mut trailing_value = temp_real2;

    if sum_roc1 <= period_roc || is_zero(sum_roc1) {
        temp_real = 1.0;
    } else {
        temp_real = (period_roc / sum_roc1).abs();
    }
    temp_real = (temp_real * const_diff) + const_max;
    temp_real *= temp_real;
    prev_kama = ((in_real[today] - prev_kama) * temp_real) + prev_kama;
    today += 1;

    while today <= adjusted_start {
        temp_real = in_real[today];
        temp_real2 = in_real[trailing_idx];
        trailing_idx += 1;
        period_roc = temp_real - temp_real2;
        sum_roc1 -= (trailing_value - temp_real2).abs();
        sum_roc1 += (temp_real - in_real[today - 1]).abs();
        trailing_value = temp_real2;

        if sum_roc1 <= period_roc || is_zero(sum_roc1) {
            temp_real = 1.0;
        } else {
            temp_real = (period_roc / sum_roc1).abs();
        }
        temp_real = (temp_real * const_diff) + const_max;
        temp_real *= temp_real;
        prev_kama = ((in_real[today] - prev_kama) * temp_real) + prev_kama;
        today += 1;
    }

    out_real[0] = prev_kama;
    let mut out_idx = 1usize;
    while today <= end_idx {
        temp_real = in_real[today];
        temp_real2 = in_real[trailing_idx];
        trailing_idx += 1;
        period_roc = temp_real - temp_real2;
        sum_roc1 -= (trailing_value - temp_real2).abs();
        sum_roc1 += (temp_real - in_real[today - 1]).abs();
        trailing_value = temp_real2;

        if sum_roc1 <= period_roc || is_zero(sum_roc1) {
            temp_real = 1.0;
        } else {
            temp_real = (period_roc / sum_roc1).abs();
        }
        temp_real = (temp_real * const_diff) + const_max;
        temp_real *= temp_real;
        prev_kama = ((in_real[today] - prev_kama) * temp_real) + prev_kama;
        out_real[out_idx] = prev_kama;
        out_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

fn normalize_vfactor(value: f64) -> Result<f64, RetCode> {
    let normalized = if value == REAL_DEFAULT { 0.7 } else { value };
    if !(0.0..=1.0).contains(&normalized) {
        return Err(RetCode::BadParam);
    }
    Ok(normalized)
}

pub(crate) fn t3_lookback(context: &Context, opt_in_time_period: i32, opt_in_v_factor: f64) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 5, 2, 100_000) else {
        return -1;
    };
    if normalize_vfactor(opt_in_v_factor).is_err() {
        return -1;
    }

    (6 * (period as i32 - 1)) + context.get_unstable_period(crate::FuncUnstId::T3) as i32
}

pub(crate) fn t3_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    opt_in_time_period: i32,
    opt_in_v_factor: f64,
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

    let Ok(period) = normalize_period(opt_in_time_period, 5, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let opt_in_v_factor = match normalize_vfactor(opt_in_v_factor) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };

    let unstable = context.get_unstable_period(crate::FuncUnstId::T3) as usize;
    let lookback_total = (6 * (period - 1)) + unstable;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    *out_beg_idx = adjusted_start;
    let mut today = adjusted_start - lookback_total;
    let k = 2.0 / (period as f64 + 1.0);
    let one_minus_k = 1.0 - k;

    let mut temp_real = in_real[today];
    today += 1;
    for _ in 0..(period - 1) {
        temp_real += in_real[today];
        today += 1;
    }
    let mut e1 = temp_real / period as f64;

    temp_real = e1;
    for _ in 0..(period - 1) {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        temp_real += e1;
        today += 1;
    }
    let mut e2 = temp_real / period as f64;

    temp_real = e2;
    for _ in 0..(period - 1) {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        e2 = (k * e1) + (one_minus_k * e2);
        temp_real += e2;
        today += 1;
    }
    let mut e3 = temp_real / period as f64;

    temp_real = e3;
    for _ in 0..(period - 1) {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        e2 = (k * e1) + (one_minus_k * e2);
        e3 = (k * e2) + (one_minus_k * e3);
        temp_real += e3;
        today += 1;
    }
    let mut e4 = temp_real / period as f64;

    temp_real = e4;
    for _ in 0..(period - 1) {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        e2 = (k * e1) + (one_minus_k * e2);
        e3 = (k * e2) + (one_minus_k * e3);
        e4 = (k * e3) + (one_minus_k * e4);
        temp_real += e4;
        today += 1;
    }
    let mut e5 = temp_real / period as f64;

    temp_real = e5;
    for _ in 0..(period - 1) {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        e2 = (k * e1) + (one_minus_k * e2);
        e3 = (k * e2) + (one_minus_k * e3);
        e4 = (k * e3) + (one_minus_k * e4);
        e5 = (k * e4) + (one_minus_k * e5);
        temp_real += e5;
        today += 1;
    }
    let mut e6 = temp_real / period as f64;

    while today <= adjusted_start {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        e2 = (k * e1) + (one_minus_k * e2);
        e3 = (k * e2) + (one_minus_k * e3);
        e4 = (k * e3) + (one_minus_k * e4);
        e5 = (k * e4) + (one_minus_k * e5);
        e6 = (k * e5) + (one_minus_k * e6);
        today += 1;
    }

    let temp_real = opt_in_v_factor * opt_in_v_factor;
    let c1 = -(temp_real * opt_in_v_factor);
    let c2 = 3.0 * (temp_real - c1);
    let c3 = -6.0 * temp_real - 3.0 * (opt_in_v_factor - c1);
    let c4 = 1.0 + 3.0 * opt_in_v_factor - c1 + 3.0 * temp_real;

    let mut out_idx = 0usize;
    out_real[out_idx] = (c1 * e6) + (c2 * e5) + (c3 * e4) + (c4 * e3);
    out_idx += 1;

    while today <= end_idx {
        e1 = (k * in_real[today]) + (one_minus_k * e1);
        e2 = (k * e1) + (one_minus_k * e2);
        e3 = (k * e2) + (one_minus_k * e3);
        e4 = (k * e3) + (one_minus_k * e4);
        e5 = (k * e4) + (one_minus_k * e5);
        e6 = (k * e5) + (one_minus_k * e6);
        out_real[out_idx] = (c1 * e6) + (c2 * e5) + (c3 * e4) + (c4 * e3);
        out_idx += 1;
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn wma_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    (period - 1) as i32
}

pub(crate) fn wma_run(
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

    let divider = ((period * (period + 1)) / 2) as f64;
    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let window_start = today + 1 - period;
        let mut weighted_sum = 0.0;
        for (weight, value) in (1..=period).zip(&in_real[window_start..=today]) {
            weighted_sum += *value * weight as f64;
        }
        out_real[out_idx] = weighted_sum / divider;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn trima_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    (period - 1) as i32
}

pub(crate) fn trima_run(
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

    let weights = triangular_weights(period);
    let divider: f64 = weights.iter().map(|weight| *weight as f64).sum();
    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let window_start = today + 1 - period;
        let mut weighted_sum = 0.0;
        for (weight, value) in weights.iter().zip(&in_real[window_start..=today]) {
            weighted_sum += *value * *weight as f64;
        }
        out_real[out_idx] = weighted_sum / divider;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

fn triangular_weights(period: usize) -> Vec<usize> {
    let mut weights = Vec::with_capacity(period);
    let midpoint = period / 2;

    for index in 0..period {
        let weight = if period % 2 == 0 {
            if index < midpoint {
                index + 1
            } else {
                period - index
            }
        } else {
            let peak = midpoint + 1;
            if index < midpoint {
                index + 1
            } else {
                peak - (index - midpoint)
            }
        };
        weights.push(weight);
    }

    weights
}

#[derive(Debug, Clone)]
struct HilbertState {
    odd: [f64; 3],
    even: [f64; 3],
    prev_odd: f64,
    prev_even: f64,
    prev_input_odd: f64,
    prev_input_even: f64,
}

impl HilbertState {
    fn new() -> Self {
        Self {
            odd: [0.0; 3],
            even: [0.0; 3],
            prev_odd: 0.0,
            prev_even: 0.0,
            prev_input_odd: 0.0,
            prev_input_even: 0.0,
        }
    }

    fn transform_even(
        &mut self,
        input: f64,
        hilbert_idx: usize,
        adjusted_prev_period: f64,
        a: f64,
        b: f64,
    ) -> f64 {
        let hilbert_temp_real = a * input;
        let mut value = -self.even[hilbert_idx];
        self.even[hilbert_idx] = hilbert_temp_real;
        value += hilbert_temp_real;
        value -= self.prev_even;
        self.prev_even = b * self.prev_input_even;
        value += self.prev_even;
        self.prev_input_even = input;
        value * adjusted_prev_period
    }

    fn transform_odd(
        &mut self,
        input: f64,
        hilbert_idx: usize,
        adjusted_prev_period: f64,
        a: f64,
        b: f64,
    ) -> f64 {
        let hilbert_temp_real = a * input;
        let mut value = -self.odd[hilbert_idx];
        self.odd[hilbert_idx] = hilbert_temp_real;
        value += hilbert_temp_real;
        value -= self.prev_odd;
        self.prev_odd = b * self.prev_input_odd;
        value += self.prev_odd;
        self.prev_input_odd = input;
        value * adjusted_prev_period
    }
}

fn normalize_mama_limit(value: f64, default_value: f64) -> Result<f64, RetCode> {
    let normalized = if value == REAL_DEFAULT {
        default_value
    } else {
        value
    };
    if !(0.01..=0.99).contains(&normalized) {
        return Err(RetCode::BadParam);
    }
    Ok(normalized)
}

pub(crate) fn mama_lookback(
    context: &Context,
    opt_in_fast_limit: f64,
    opt_in_slow_limit: f64,
) -> i32 {
    if normalize_mama_limit(opt_in_fast_limit, 0.5).is_err()
        || normalize_mama_limit(opt_in_slow_limit, 0.05).is_err()
    {
        return -1;
    }

    32 + context.get_unstable_period(crate::FuncUnstId::Mama) as i32
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn mama_run(
    context: &Context,
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let opt_in_fast_limit = match normalize_mama_limit(opt_in_fast_limit, 0.5) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };
    let opt_in_slow_limit = match normalize_mama_limit(opt_in_slow_limit, 0.05) {
        Ok(value) => value,
        Err(ret_code) => return ret_code,
    };

    let lookback_total = 32 + context.get_unstable_period(crate::FuncUnstId::Mama) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_mama, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_fama, needed) {
        return ret_code;
    }

    let a = 0.0962;
    let b = 0.5769;
    let rad2deg = 180.0 / (4.0 * 1.0f64.atan());

    *out_beg_idx = adjusted_start;

    let mut trailing_wma_idx = adjusted_start - lookback_total;
    let mut today = trailing_wma_idx;
    let mut temp_real = in_real[today];
    today += 1;
    let mut period_wma_sub = temp_real;
    let mut period_wma_sum = temp_real;
    temp_real = in_real[today];
    today += 1;
    period_wma_sub += temp_real;
    period_wma_sum += temp_real * 2.0;
    temp_real = in_real[today];
    today += 1;
    period_wma_sub += temp_real;
    period_wma_sum += temp_real * 3.0;
    let mut trailing_wma_value = 0.0;

    let do_price_wma = |new_price: f64,
                        period_wma_sub: &mut f64,
                        period_wma_sum: &mut f64,
                        trailing_wma_value: &mut f64,
                        trailing_wma_idx: &mut usize|
     -> f64 {
        *period_wma_sub += new_price;
        *period_wma_sub -= *trailing_wma_value;
        *period_wma_sum += new_price * 4.0;
        *trailing_wma_value = in_real[*trailing_wma_idx];
        *trailing_wma_idx += 1;
        let smoothed_value = *period_wma_sum * 0.1;
        *period_wma_sum -= *period_wma_sub;
        smoothed_value
    };

    for _ in 0..9 {
        temp_real = in_real[today];
        today += 1;
        let _ = do_price_wma(
            temp_real,
            &mut period_wma_sub,
            &mut period_wma_sum,
            &mut trailing_wma_value,
            &mut trailing_wma_idx,
        );
    }

    let mut hilbert_idx = 0usize;
    let mut detrender = HilbertState::new();
    let mut q1 = HilbertState::new();
    let mut j_i = HilbertState::new();
    let mut j_q = HilbertState::new();

    let mut period = 0.0;
    let mut out_idx = 0usize;
    let mut prev_i2 = 0.0;
    let mut prev_q2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut mama = 0.0;
    let mut fama = 0.0;
    let mut i1_for_odd_prev3 = 0.0;
    let mut i1_for_even_prev3 = 0.0;
    let mut i1_for_odd_prev2 = 0.0;
    let mut i1_for_even_prev2 = 0.0;
    let mut prev_phase = 0.0;

    while today <= end_idx {
        let adjusted_prev_period = (0.075 * period) + 0.54;
        let today_value = in_real[today];
        let smoothed_value = do_price_wma(
            today_value,
            &mut period_wma_sub,
            &mut period_wma_sum,
            &mut trailing_wma_value,
            &mut trailing_wma_idx,
        );

        let (q2, i2, alpha_phase) = if today % 2 == 0 {
            let detrender_value =
                detrender.transform_even(smoothed_value, hilbert_idx, adjusted_prev_period, a, b);
            let q1_value =
                q1.transform_even(detrender_value, hilbert_idx, adjusted_prev_period, a, b);
            let ji_value =
                j_i.transform_even(i1_for_even_prev3, hilbert_idx, adjusted_prev_period, a, b);
            let jq_value = j_q.transform_even(q1_value, hilbert_idx, adjusted_prev_period, a, b);
            hilbert_idx += 1;
            if hilbert_idx == 3 {
                hilbert_idx = 0;
            }

            let q2 = (0.2 * (q1_value + ji_value)) + (0.8 * prev_q2);
            let i2 = (0.2 * (i1_for_even_prev3 - jq_value)) + (0.8 * prev_i2);
            i1_for_odd_prev3 = i1_for_odd_prev2;
            i1_for_odd_prev2 = detrender_value;
            let alpha_phase = if i1_for_even_prev3 != 0.0 {
                (q1_value / i1_for_even_prev3).atan() * rad2deg
            } else {
                0.0
            };
            (q2, i2, alpha_phase)
        } else {
            let detrender_value =
                detrender.transform_odd(smoothed_value, hilbert_idx, adjusted_prev_period, a, b);
            let q1_value =
                q1.transform_odd(detrender_value, hilbert_idx, adjusted_prev_period, a, b);
            let ji_value =
                j_i.transform_odd(i1_for_odd_prev3, hilbert_idx, adjusted_prev_period, a, b);
            let jq_value = j_q.transform_odd(q1_value, hilbert_idx, adjusted_prev_period, a, b);

            let q2 = (0.2 * (q1_value + ji_value)) + (0.8 * prev_q2);
            let i2 = (0.2 * (i1_for_odd_prev3 - jq_value)) + (0.8 * prev_i2);
            i1_for_even_prev3 = i1_for_even_prev2;
            i1_for_even_prev2 = detrender_value;
            let alpha_phase = if i1_for_odd_prev3 != 0.0 {
                (q1_value / i1_for_odd_prev3).atan() * rad2deg
            } else {
                0.0
            };
            (q2, i2, alpha_phase)
        };

        let mut delta_phase = prev_phase - alpha_phase;
        prev_phase = alpha_phase;
        if delta_phase < 1.0 {
            delta_phase = 1.0;
        }

        let mut alpha = if delta_phase > 1.0 {
            let alpha = opt_in_fast_limit / delta_phase;
            if alpha < opt_in_slow_limit {
                opt_in_slow_limit
            } else {
                alpha
            }
        } else {
            opt_in_fast_limit
        };

        mama = (alpha * today_value) + ((1.0 - alpha) * mama);
        alpha *= 0.5;
        fama = (alpha * mama) + ((1.0 - alpha) * fama);
        if today >= adjusted_start {
            out_mama[out_idx] = mama;
            out_fama[out_idx] = fama;
            out_idx += 1;
        }

        re = (0.2 * ((i2 * prev_i2) + (q2 * prev_q2))) + (0.8 * re);
        im = (0.2 * ((i2 * prev_q2) - (q2 * prev_i2))) + (0.8 * im);
        prev_q2 = q2;
        prev_i2 = i2;
        let previous_period = period;
        if im != 0.0 && re != 0.0 {
            period = 360.0 / ((im / re).atan() * rad2deg);
        }
        let mut temp_real2 = 1.5 * previous_period;
        if period > temp_real2 {
            period = temp_real2;
        }
        temp_real2 = 0.67 * previous_period;
        if period < temp_real2 {
            period = temp_real2;
        }
        period = period.clamp(6.0, 50.0);
        period = (0.2 * period) + (0.8 * previous_period);
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn dema_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    ema::lookback(context, period as i32) * 2
}

pub(crate) fn dema_run(
    context: &Context,
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
    let ema_lookback = ema::lookback(context, period as i32) as usize;
    let lookback_total = ema_lookback * 2;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut first_buf = vec![0.0; end_idx + 1];
    let mut first_beg = 0usize;
    let mut first_nb = 0usize;
    let ret_code = ema::run(
        context,
        0,
        end_idx,
        in_real,
        period as i32,
        &mut first_beg,
        &mut first_nb,
        &mut first_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if first_nb == 0 {
        return RetCode::Success;
    }

    let mut second_buf = vec![0.0; first_nb];
    let mut second_beg = 0usize;
    let mut second_nb = 0usize;
    let ret_code = ema::run(
        context,
        0,
        first_nb - 1,
        &first_buf[..first_nb],
        period as i32,
        &mut second_beg,
        &mut second_nb,
        &mut second_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    let second_original_beg = first_beg + second_beg;
    if second_original_beg > adjusted_start {
        return RetCode::InternalError;
    }
    let first_offset = adjusted_start.saturating_sub(first_beg);
    let second_offset = adjusted_start - second_original_beg;
    if first_offset + needed > first_nb || second_offset + needed > second_nb {
        return RetCode::InternalError;
    }

    for idx in 0..needed {
        out_real[idx] = (2.0 * first_buf[first_offset + idx]) - second_buf[second_offset + idx];
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn tema_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    ema::lookback(context, period as i32) * 3
}

pub(crate) fn tema_run(
    context: &Context,
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
    let ema_lookback = ema::lookback(context, period as i32) as usize;
    let lookback_total = ema_lookback * 3;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut first_buf = vec![0.0; end_idx + 1];
    let mut first_beg = 0usize;
    let mut first_nb = 0usize;
    let ret_code = ema::run(
        context,
        0,
        end_idx,
        in_real,
        period as i32,
        &mut first_beg,
        &mut first_nb,
        &mut first_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if first_nb == 0 {
        return RetCode::Success;
    }

    let mut second_buf = vec![0.0; first_nb];
    let mut second_beg = 0usize;
    let mut second_nb = 0usize;
    let ret_code = ema::run(
        context,
        0,
        first_nb - 1,
        &first_buf[..first_nb],
        period as i32,
        &mut second_beg,
        &mut second_nb,
        &mut second_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if second_nb == 0 {
        return RetCode::InternalError;
    }

    let mut third_buf = vec![0.0; second_nb];
    let mut third_beg = 0usize;
    let mut third_nb = 0usize;
    let ret_code = ema::run(
        context,
        0,
        second_nb - 1,
        &second_buf[..second_nb],
        period as i32,
        &mut third_beg,
        &mut third_nb,
        &mut third_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    let second_original_beg = first_beg + second_beg;
    let third_original_beg = second_original_beg + third_beg;
    if third_original_beg > adjusted_start {
        return RetCode::InternalError;
    }
    let first_offset = adjusted_start.saturating_sub(first_beg);
    let second_offset = adjusted_start.saturating_sub(second_original_beg);
    let third_offset = adjusted_start - third_original_beg;
    if first_offset + needed > first_nb
        || second_offset + needed > second_nb
        || third_offset + needed > third_nb
    {
        return RetCode::InternalError;
    }

    for idx in 0..needed {
        out_real[idx] = (3.0 * first_buf[first_offset + idx])
            - (3.0 * second_buf[second_offset + idx])
            + third_buf[third_offset + idx];
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn trix_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let period = if opt_in_time_period == INTEGER_DEFAULT {
        30
    } else {
        opt_in_time_period
    };
    if !(1..=100_000).contains(&period) {
        return -1;
    }

    let ema_lookback = if period <= 1 {
        0
    } else {
        ema::lookback(context, period)
    };
    (ema_lookback * 3) + 1
}

pub(crate) fn trix_run(
    context: &Context,
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

    let period = if opt_in_time_period == INTEGER_DEFAULT {
        30
    } else {
        opt_in_time_period
    };
    if !(1..=100_000).contains(&period) {
        return RetCode::BadParam;
    }

    let ema_lookback = if period <= 1 {
        0usize
    } else {
        ema::lookback(context, period) as usize
    };
    let lookback_total = (ema_lookback * 3) + 1;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    if period <= 1 {
        for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
            let prev = in_real[today - 1];
            out_real[out_idx] = if prev != 0.0 {
                ((in_real[today] / prev) - 1.0) * 100.0
            } else {
                0.0
            };
        }
        *out_beg_idx = adjusted_start;
        *out_nb_element = needed;
        return RetCode::Success;
    }

    let triple_start = adjusted_start - 1;
    let first_start = triple_start - (ema_lookback * 3);

    let mut first_buf = vec![0.0; end_idx - first_start + 1];
    let mut first_beg = 0usize;
    let mut first_nb = 0usize;
    let ret_code = ema::run(
        context,
        first_start,
        end_idx,
        in_real,
        period,
        &mut first_beg,
        &mut first_nb,
        &mut first_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }

    let mut second_buf = vec![0.0; first_nb.saturating_sub(ema_lookback)];
    let mut second_beg = 0usize;
    let mut second_nb = 0usize;
    let ret_code = ema::run(
        context,
        ema_lookback,
        first_nb - 1,
        &first_buf[..first_nb],
        period,
        &mut second_beg,
        &mut second_nb,
        &mut second_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }

    let mut third_buf = vec![0.0; second_nb.saturating_sub(ema_lookback)];
    let mut third_beg = 0usize;
    let mut third_nb = 0usize;
    let ret_code = ema::run(
        context,
        ema_lookback,
        second_nb - 1,
        &second_buf[..second_nb],
        period,
        &mut third_beg,
        &mut third_nb,
        &mut third_buf,
    );
    if ret_code != RetCode::Success {
        return ret_code;
    }
    if third_beg != ema_lookback || third_nb < needed + 1 {
        return RetCode::InternalError;
    }

    for idx in 0..needed {
        let prev = third_buf[idx];
        let curr = third_buf[idx + 1];
        out_real[idx] = if prev != 0.0 {
            ((curr / prev) - 1.0) * 100.0
        } else {
            0.0
        };
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

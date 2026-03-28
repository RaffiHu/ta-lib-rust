use crate::helpers::{
    is_zero, normalize_period, validate_input_len, validate_output_len, validate_range,
};
use crate::{Compatibility, Context, FuncUnstId, RetCode};

#[derive(Clone, Copy)]
pub(crate) enum RocMode {
    Mom,
    Roc,
    RocP,
    RocR,
    RocR100,
}

pub(crate) fn bop_lookback() -> i32 {
    0
}

pub(crate) fn imi_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32 + context.get_unstable_period(FuncUnstId::Imi) as i32 - 1
}

pub(crate) fn imi_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
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
    for input in [in_open, in_close] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let lookback_total = period + context.get_unstable_period(FuncUnstId::Imi) as usize - 1;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let trailing = today - lookback_total;
        let mut up_sum = 0.0;
        let mut down_sum = 0.0;
        for i in trailing..=today {
            let close = in_close[i];
            let open = in_open[i];
            if close > open {
                up_sum += close - open;
            } else {
                down_sum += open - close;
            }
        }

        let total = up_sum + down_sum;
        out_real[out_idx] = if is_zero(total) {
            0.0
        } else {
            100.0 * (up_sum / total)
        };
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn bop_run(
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
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_open, in_high, in_low, in_close] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, today) in (start_idx..=end_idx).enumerate() {
        let spread = in_high[today] - in_low[today];
        out_real[out_idx] = if spread < 0.0 || is_zero(spread) {
            0.0
        } else {
            (in_close[today] - in_open[today]) / spread
        };
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn roc_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 10, 1, 100_000) else {
        return -1;
    };

    period as i32
}

pub(crate) fn roc_run(
    mode: RocMode,
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

    let Ok(period) = normalize_period(opt_in_time_period, 10, 1, 100_000) else {
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

    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let prev = in_real[today - period];
        let current = in_real[today];
        out_real[out_idx] = match mode {
            RocMode::Mom => current - prev,
            RocMode::Roc => {
                if prev != 0.0 {
                    ((current / prev) - 1.0) * 100.0
                } else {
                    0.0
                }
            }
            RocMode::RocP => {
                if prev != 0.0 {
                    (current - prev) / prev
                } else {
                    0.0
                }
            }
            RocMode::RocR => {
                if prev != 0.0 {
                    current / prev
                } else {
                    0.0
                }
            }
            RocMode::RocR100 => {
                if prev != 0.0 {
                    (current / prev) * 100.0
                } else {
                    0.0
                }
            }
        };
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn willr_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1
}

pub(crate) fn willr_run(
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

        let diff = (highest - lowest) / -100.0;
        out_real[out_idx] = if diff != 0.0 {
            (highest - in_close[today]) / diff
        } else {
            0.0
        };
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn cci_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1
}

pub(crate) fn cci_run(
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

    let mut out_idx = 0usize;
    for today in adjusted_start..=end_idx {
        let trailing_idx = today - (period - 1);
        let mut sum_tp = 0.0;
        let mut last_value = 0.0;
        for i in trailing_idx..=today {
            let tp = (in_high[i] + in_low[i] + in_close[i]) / 3.0;
            sum_tp += tp;
            if i == today {
                last_value = tp;
            }
        }

        let average = sum_tp / period as f64;
        let mut mean_dev = 0.0;
        for i in trailing_idx..=today {
            let tp = (in_high[i] + in_low[i] + in_close[i]) / 3.0;
            mean_dev += (tp - average).abs();
        }

        let numerator = last_value - average;
        out_real[out_idx] = if numerator != 0.0 && mean_dev != 0.0 {
            numerator / (0.015 * (mean_dev / period as f64))
        } else {
            0.0
        };
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn aroon_lookback(opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    period as i32
}

pub(crate) fn aroon_run(
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
    let adjusted_start = start_idx.max(period);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_aroon_down, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_aroon_up, needed) {
        return ret_code;
    }

    let factor = 100.0 / period as f64;
    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let trailing_idx = today - period;
        let mut lowest_idx = trailing_idx;
        let mut highest_idx = trailing_idx;
        let mut lowest = in_low[lowest_idx];
        let mut highest = in_high[highest_idx];

        for i in trailing_idx + 1..=today {
            let low = in_low[i];
            if low <= lowest {
                lowest = low;
                lowest_idx = i;
            }

            let high = in_high[i];
            if high >= highest {
                highest = high;
                highest_idx = i;
            }
        }

        out_aroon_up[out_idx] = factor * (period - (today - highest_idx)) as f64;
        out_aroon_down[out_idx] = factor * (period - (today - lowest_idx)) as f64;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn aroon_osc_lookback(opt_in_time_period: i32) -> i32 {
    aroon_lookback(opt_in_time_period)
}

pub(crate) fn aroon_osc_run(
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
    let adjusted_start = start_idx.max(period);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let factor = 100.0 / period as f64;
    for (out_idx, today) in (adjusted_start..=end_idx).enumerate() {
        let trailing_idx = today - period;
        let mut lowest_idx = trailing_idx;
        let mut highest_idx = trailing_idx;
        let mut lowest = in_low[lowest_idx];
        let mut highest = in_high[highest_idx];

        for i in trailing_idx + 1..=today {
            let low = in_low[i];
            if low <= lowest {
                lowest = low;
                lowest_idx = i;
            }

            let high = in_high[i];
            if high >= highest {
                highest = high;
                highest_idx = i;
            }
        }

        out_real[out_idx] = factor * (highest_idx as f64 - lowest_idx as f64);
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn cmo_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    let mut ret_value = period as i32 + context.get_unstable_period(FuncUnstId::Cmo) as i32;
    if context.get_compatibility() == Compatibility::Metastock {
        ret_value -= 1;
    }
    ret_value
}

pub(crate) fn cmo_run(
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

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let lookback_total = cmo_lookback(context, period as i32);
    if lookback_total < 0 {
        return RetCode::BadParam;
    }
    let lookback_total = lookback_total as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut out_idx = 0usize;
    let unstable_period = context.get_unstable_period(FuncUnstId::Cmo) as usize;
    let mut today = adjusted_start - lookback_total;
    let mut prev_value = in_real[today];
    let mut prev_gain;
    let mut prev_loss;

    if unstable_period == 0 && context.get_compatibility() == Compatibility::Metastock {
        let save_prev_value = prev_value;
        prev_gain = 0.0;
        prev_loss = 0.0;

        for _ in 0..period {
            let temp_value1 = in_real[today];
            today += 1;
            let temp_value2 = temp_value1 - prev_value;
            prev_value = temp_value1;
            if temp_value2 < 0.0 {
                prev_loss -= temp_value2;
            } else {
                prev_gain += temp_value2;
            }
        }

        let avg_loss = prev_loss / period as f64;
        let avg_gain = prev_gain / period as f64;
        let sum = avg_gain + avg_loss;
        out_real[out_idx] = if !is_zero(sum) {
            100.0 * ((avg_gain - avg_loss) / sum)
        } else {
            0.0
        };
        out_idx += 1;

        if today > end_idx {
            *out_beg_idx = adjusted_start;
            *out_nb_element = out_idx;
            return RetCode::Success;
        }

        today -= period;
        prev_value = save_prev_value;
    }

    prev_gain = 0.0;
    prev_loss = 0.0;
    today += 1;
    for _ in 0..period {
        let temp_value1 = in_real[today];
        today += 1;
        let temp_value2 = temp_value1 - prev_value;
        prev_value = temp_value1;
        if temp_value2 < 0.0 {
            prev_loss -= temp_value2;
        } else {
            prev_gain += temp_value2;
        }
    }

    prev_loss /= period as f64;
    prev_gain /= period as f64;

    if today > adjusted_start {
        let sum = prev_gain + prev_loss;
        out_real[out_idx] = if !is_zero(sum) {
            100.0 * ((prev_gain - prev_loss) / sum)
        } else {
            0.0
        };
        out_idx += 1;
    } else {
        while today < adjusted_start {
            let temp_value1 = in_real[today];
            let temp_value2 = temp_value1 - prev_value;
            prev_value = temp_value1;

            prev_loss *= period as f64 - 1.0;
            prev_gain *= period as f64 - 1.0;
            if temp_value2 < 0.0 {
                prev_loss -= temp_value2;
            } else {
                prev_gain += temp_value2;
            }

            prev_loss /= period as f64;
            prev_gain /= period as f64;
            today += 1;
        }
    }

    while today <= end_idx {
        let temp_value1 = in_real[today];
        today += 1;
        let temp_value2 = temp_value1 - prev_value;
        prev_value = temp_value1;

        prev_loss *= period as f64 - 1.0;
        prev_gain *= period as f64 - 1.0;
        if temp_value2 < 0.0 {
            prev_loss -= temp_value2;
        } else {
            prev_gain += temp_value2;
        }

        prev_loss /= period as f64;
        prev_gain /= period as f64;
        let sum = prev_gain + prev_loss;
        out_real[out_idx] = if !is_zero(sum) {
            100.0 * ((prev_gain - prev_loss) / sum)
        } else {
            0.0
        };
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

use crate::helpers::{
    is_zero, normalize_period, validate_input_len, validate_output_len, validate_range,
};
use crate::{Compatibility, Context, FuncUnstId, RetCode};

pub(crate) fn lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };

    let mut ret_value = period as i32 + context.get_unstable_period(FuncUnstId::Rsi) as i32;
    if context.get_compatibility() == Compatibility::Metastock {
        ret_value -= 1;
    }

    ret_value
}

pub(crate) fn run(
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

    let lookback_total = lookback(context, period as i32);
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
    let unstable_period = context.get_unstable_period(FuncUnstId::Rsi) as usize;
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

        let temp_value1 = prev_loss / period as f64;
        let temp_value2 = prev_gain / period as f64;
        let sum = temp_value2 + temp_value1;
        out_real[out_idx] = if !is_zero(sum) {
            100.0 * (temp_value2 / sum)
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
            100.0 * (prev_gain / sum)
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
            100.0 * (prev_gain / sum)
        } else {
            0.0
        };
        out_idx += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

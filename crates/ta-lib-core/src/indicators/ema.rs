use crate::helpers::{
    normalize_period, per_to_k, validate_input_len, validate_output_len, validate_range,
};
use crate::{Compatibility, Context, FuncUnstId, RetCode};

pub(crate) fn lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return -1;
    };

    period as i32 - 1 + context.get_unstable_period(FuncUnstId::Ema) as i32
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
    let Ok(period) = normalize_period(opt_in_time_period, 30, 2, 100_000) else {
        return RetCode::BadParam;
    };

    run_with_k(
        context,
        start_idx,
        end_idx,
        in_real,
        period,
        per_to_k(period),
        out_beg_idx,
        out_nb_element,
        out_real,
    )
}

pub(crate) fn run_with_k(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    period: usize,
    opt_in_k_1: f64,
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

    let lookback_total = period - 1 + context.get_unstable_period(FuncUnstId::Ema) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut today;
    let mut prev_ma;
    if context.get_compatibility() == Compatibility::Default {
        today = adjusted_start - lookback_total;
        let mut temp_real = 0.0;
        for _ in 0..period {
            temp_real += in_real[today];
            today += 1;
        }
        prev_ma = temp_real / period as f64;
    } else {
        prev_ma = in_real[0];
        today = 1;
    }

    while today <= adjusted_start {
        prev_ma = ((in_real[today] - prev_ma) * opt_in_k_1) + prev_ma;
        today += 1;
    }

    out_real[0] = prev_ma;
    let mut out_idx = 1;

    while today <= end_idx {
        prev_ma = ((in_real[today] - prev_ma) * opt_in_k_1) + prev_ma;
        out_real[out_idx] = prev_ma;
        out_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

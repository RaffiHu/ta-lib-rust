use crate::helpers::{
    normalize_period, per_to_k, validate_input_len, validate_output_len, validate_range,
};
use crate::{Context, FuncUnstId, RetCode};

pub(crate) fn ad_lookback() -> i32 {
    0
}

pub(crate) fn ad_run(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    in_volume: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_high, in_low, in_close, in_volume] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut ad = 0.0;
    for (out_idx, today) in (start_idx..=end_idx).enumerate() {
        let high = in_high[today];
        let low = in_low[today];
        let tmp = high - low;
        let close = in_close[today];
        if tmp > 0.0 {
            ad += (((close - low) - (high - close)) / tmp) * in_volume[today];
        }
        out_real[out_idx] = ad;
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn obv_lookback() -> i32 {
    0
}

pub(crate) fn obv_run(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    in_volume: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_real, in_volume] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut prev_obv = in_volume[start_idx];
    let mut prev_real = in_real[start_idx];
    for (out_idx, today) in (start_idx..=end_idx).enumerate() {
        let current = in_real[today];
        if current > prev_real {
            prev_obv += in_volume[today];
        } else if current < prev_real {
            prev_obv -= in_volume[today];
        }
        out_real[out_idx] = prev_obv;
        prev_real = current;
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn mfi_lookback(context: &Context, opt_in_time_period: i32) -> i32 {
    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return -1;
    };
    period as i32 + context.get_unstable_period(FuncUnstId::Mfi) as i32
}

pub(crate) fn mfi_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    in_volume: &[f64],
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
    for input in [in_high, in_low, in_close, in_volume] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(period) = normalize_period(opt_in_time_period, 14, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let unstable_period = context.get_unstable_period(FuncUnstId::Mfi) as usize;
    let lookback_total = period + unstable_period;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let mut positive = vec![0.0; period];
    let mut negative = vec![0.0; period];
    let mut pos_sum = 0.0;
    let mut neg_sum = 0.0;

    let mut today = adjusted_start - lookback_total;
    let mut prev_value = typical_price(in_high, in_low, in_close, today);
    today += 1;

    for idx in 0..period {
        let typical = typical_price(in_high, in_low, in_close, today);
        let delta = typical - prev_value;
        prev_value = typical;
        let money_flow = typical * in_volume[today];
        if delta < 0.0 {
            negative[idx] = money_flow;
            neg_sum += money_flow;
        } else if delta > 0.0 {
            positive[idx] = money_flow;
            pos_sum += money_flow;
        }
        today += 1;
    }

    let mut ring_idx = 0usize;
    let mut out_idx = 0usize;
    if today > adjusted_start {
        let total = pos_sum + neg_sum;
        out_real[out_idx] = if total < 1.0 {
            0.0
        } else {
            100.0 * (pos_sum / total)
        };
        out_idx += 1;
    } else {
        while today < adjusted_start {
            pos_sum -= positive[ring_idx];
            neg_sum -= negative[ring_idx];

            let typical = typical_price(in_high, in_low, in_close, today);
            let delta = typical - prev_value;
            prev_value = typical;
            let money_flow = typical * in_volume[today];
            if delta < 0.0 {
                positive[ring_idx] = 0.0;
                negative[ring_idx] = money_flow;
                neg_sum += money_flow;
            } else if delta > 0.0 {
                positive[ring_idx] = money_flow;
                negative[ring_idx] = 0.0;
                pos_sum += money_flow;
            } else {
                positive[ring_idx] = 0.0;
                negative[ring_idx] = 0.0;
            }

            ring_idx = (ring_idx + 1) % period;
            today += 1;
        }
    }

    while today <= end_idx {
        pos_sum -= positive[ring_idx];
        neg_sum -= negative[ring_idx];

        let typical = typical_price(in_high, in_low, in_close, today);
        let delta = typical - prev_value;
        prev_value = typical;
        let money_flow = typical * in_volume[today];
        if delta < 0.0 {
            positive[ring_idx] = 0.0;
            negative[ring_idx] = money_flow;
            neg_sum += money_flow;
        } else if delta > 0.0 {
            positive[ring_idx] = money_flow;
            negative[ring_idx] = 0.0;
            pos_sum += money_flow;
        } else {
            positive[ring_idx] = 0.0;
            negative[ring_idx] = 0.0;
        }

        let total = pos_sum + neg_sum;
        out_real[out_idx] = if total < 1.0 {
            0.0
        } else {
            100.0 * (pos_sum / total)
        };
        out_idx += 1;
        ring_idx = (ring_idx + 1) % period;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn adosc_lookback(
    context: &Context,
    opt_in_fast_period: i32,
    opt_in_slow_period: i32,
) -> i32 {
    let Ok(fast) = normalize_period(opt_in_fast_period, 3, 2, 100_000) else {
        return -1;
    };
    let Ok(slow) = normalize_period(opt_in_slow_period, 10, 2, 100_000) else {
        return -1;
    };

    fast.max(slow) as i32 - 1 + context.get_unstable_period(FuncUnstId::Ema) as i32
}

pub(crate) fn adosc_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    in_volume: &[f64],
    opt_in_fast_period: i32,
    opt_in_slow_period: i32,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    for input in [in_high, in_low, in_close, in_volume] {
        if let Err(ret_code) = validate_input_len(input, end_idx) {
            return ret_code;
        }
    }

    let Ok(fast) = normalize_period(opt_in_fast_period, 3, 2, 100_000) else {
        return RetCode::BadParam;
    };
    let Ok(slow) = normalize_period(opt_in_slow_period, 10, 2, 100_000) else {
        return RetCode::BadParam;
    };

    let slowest = fast.max(slow);
    let lookback_total = slowest - 1 + context.get_unstable_period(FuncUnstId::Ema) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    let fast_k = per_to_k(fast);
    let slow_k = per_to_k(slow);
    let one_minus_fast_k = 1.0 - fast_k;
    let one_minus_slow_k = 1.0 - slow_k;

    let mut today = adjusted_start - lookback_total;
    let mut ad = 0.0;
    update_ad(today, in_high, in_low, in_close, in_volume, &mut ad);
    today += 1;
    let mut fast_ema = ad;
    let mut slow_ema = ad;

    while today < adjusted_start {
        update_ad(today, in_high, in_low, in_close, in_volume, &mut ad);
        fast_ema = (fast_k * ad) + (one_minus_fast_k * fast_ema);
        slow_ema = (slow_k * ad) + (one_minus_slow_k * slow_ema);
        today += 1;
    }

    let mut out_idx = 0usize;
    while today <= end_idx {
        update_ad(today, in_high, in_low, in_close, in_volume, &mut ad);
        fast_ema = (fast_k * ad) + (one_minus_fast_k * fast_ema);
        slow_ema = (slow_k * ad) + (one_minus_slow_k * slow_ema);
        out_real[out_idx] = fast_ema - slow_ema;
        out_idx += 1;
        today += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = out_idx;
    RetCode::Success
}

fn typical_price(in_high: &[f64], in_low: &[f64], in_close: &[f64], idx: usize) -> f64 {
    (in_high[idx] + in_low[idx] + in_close[idx]) / 3.0
}

fn update_ad(
    today: usize,
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    in_volume: &[f64],
    ad: &mut f64,
) {
    let high = in_high[today];
    let low = in_low[today];
    let tmp = high - low;
    let close = in_close[today];
    if tmp > 0.0 {
        *ad += (((close - low) - (high - close)) / tmp) * in_volume[today];
    }
}

use crate::helpers::{validate_input_len, validate_output_len, validate_range};
use crate::{CandleSettingType, Context, RangeType, RetCode};

fn setting_avg_period(context: &Context, setting_type: CandleSettingType) -> usize {
    context
        .candle_setting(setting_type)
        .map(|setting| setting.avg_period.max(0) as usize)
        .unwrap_or(0)
}

fn candle_color(in_open: &[f64], in_close: &[f64], index: usize) -> i32 {
    if in_close[index] >= in_open[index] {
        1
    } else {
        -1
    }
}

fn real_body(in_open: &[f64], in_close: &[f64], index: usize) -> f64 {
    (in_close[index] - in_open[index]).abs()
}

fn upper_shadow(in_open: &[f64], in_high: &[f64], in_close: &[f64], index: usize) -> f64 {
    in_high[index] - in_close[index].max(in_open[index])
}

fn lower_shadow(in_open: &[f64], in_low: &[f64], in_close: &[f64], index: usize) -> f64 {
    in_close[index].min(in_open[index]) - in_low[index]
}

fn high_low_range(in_high: &[f64], in_low: &[f64], index: usize) -> f64 {
    in_high[index] - in_low[index]
}

fn real_body_gap_up(in_open: &[f64], in_close: &[f64], index: usize, previous: usize) -> bool {
    in_open[index].min(in_close[index]) > in_open[previous].max(in_close[previous])
}

fn real_body_gap_down(in_open: &[f64], in_close: &[f64], index: usize, previous: usize) -> bool {
    in_open[index].max(in_close[index]) < in_open[previous].min(in_close[previous])
}

fn candle_gap_up(in_high: &[f64], in_low: &[f64], index: usize, previous: usize) -> bool {
    in_low[index] > in_high[previous]
}

fn candle_gap_down(in_high: &[f64], in_low: &[f64], index: usize, previous: usize) -> bool {
    in_high[index] < in_low[previous]
}

fn candle_range(
    context: &Context,
    setting_type: CandleSettingType,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    index: usize,
) -> f64 {
    let Some(setting) = context.candle_setting(setting_type) else {
        return 0.0;
    };

    match setting.range_type {
        RangeType::RealBody => real_body(in_open, in_close, index),
        RangeType::HighLow => high_low_range(in_high, in_low, index),
        RangeType::Shadows => {
            upper_shadow(in_open, in_high, in_close, index)
                + lower_shadow(in_open, in_low, in_close, index)
        }
    }
}

fn candle_average(
    context: &Context,
    setting_type: CandleSettingType,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    index: usize,
) -> f64 {
    let Some(setting) = context.candle_setting(setting_type) else {
        return 0.0;
    };
    let avg_period = setting.avg_period.max(0) as usize;
    let range_factor = if matches!(setting.range_type, RangeType::Shadows) {
        2.0
    } else {
        1.0
    };

    let base = if avg_period == 0 {
        candle_range(
            context,
            setting_type,
            in_open,
            in_high,
            in_low,
            in_close,
            index,
        )
    } else {
        let start = index - avg_period;
        let mut total = 0.0;
        for period_index in start..index {
            total += candle_range(
                context,
                setting_type,
                in_open,
                in_high,
                in_low,
                in_close,
                period_index,
            );
        }
        total / avg_period as f64
    };

    setting.factor * base / range_factor
}

fn validate_ohlc(
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
) -> Result<(), RetCode> {
    validate_range(start_idx, end_idx)?;
    validate_input_len(in_open, end_idx)?;
    validate_input_len(in_high, end_idx)?;
    validate_input_len(in_low, end_idx)?;
    validate_input_len(in_close, end_idx)?;
    Ok(())
}

fn run_pattern(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    lookback: usize,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
    eval: impl Fn(&Context, &[f64], &[f64], &[f64], &[f64], usize) -> i32,
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_ohlc(start_idx, end_idx, in_open, in_high, in_low, in_close) {
        return ret_code;
    }

    let adjusted_start = start_idx.max(lookback);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_integer, needed) {
        return ret_code;
    }

    for (out_idx, index) in (adjusted_start..=end_idx).enumerate() {
        out_integer[out_idx] = eval(context, in_open, in_high, in_low, in_close, index);
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn cdl_doji_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyDoji) as i32
}

pub(crate) fn cdl_doji_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_doji_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                <= candle_average(
                    context,
                    CandleSettingType::BodyDoji,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_dragonfly_doji_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyDoji).max(setting_avg_period(
        context,
        CandleSettingType::ShadowVeryShort,
    )) as i32
}

pub(crate) fn cdl_dragonfly_doji_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_dragonfly_doji_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let very_short = candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index,
            );
            if real_body(in_open, in_close, index)
                <= candle_average(
                    context,
                    CandleSettingType::BodyDoji,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index) < very_short
                && lower_shadow(in_open, in_low, in_close, index) > very_short
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_gravestone_doji_lookback(context: &Context) -> i32 {
    cdl_dragonfly_doji_lookback(context)
}

pub(crate) fn cdl_gravestone_doji_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_gravestone_doji_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let very_short = candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index,
            );
            if real_body(in_open, in_close, index)
                <= candle_average(
                    context,
                    CandleSettingType::BodyDoji,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && lower_shadow(in_open, in_low, in_close, index) < very_short
                && upper_shadow(in_open, in_high, in_close, index) > very_short
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_spinning_top_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyShort) as i32
}

pub(crate) fn cdl_spinning_top_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_spinning_top_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let body = real_body(in_open, in_close, index);
            if body
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index) > body
                && lower_shadow(in_open, in_low, in_close, index) > body
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_marubozu_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong).max(setting_avg_period(
        context,
        CandleSettingType::ShadowVeryShort,
    )) as i32
}

pub(crate) fn cdl_marubozu_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_marubozu_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_long_line_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong)
        .max(setting_avg_period(context, CandleSettingType::ShadowShort)) as i32
}

pub(crate) fn cdl_long_line_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_long_line_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_short_line_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::ShadowShort)) as i32
}

pub(crate) fn cdl_short_line_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_short_line_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_hammer_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::ShadowLong))
        .max(setting_avg_period(
            context,
            CandleSettingType::ShadowVeryShort,
        ))
        .max(setting_avg_period(context, CandleSettingType::Near));
    lookback as i32 + 1
}

pub(crate) fn cdl_hammer_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_hammer_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && lower_shadow(in_open, in_low, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_close[index].min(in_open[index])
                    <= in_low[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_engulfing_lookback(_context: &Context) -> i32 {
    2
}

pub(crate) fn cdl_engulfing_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_engulfing_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |_, in_open, _in_high, _in_low, in_close, index| {
            let current_color = candle_color(in_open, in_close, index);
            let previous_color = candle_color(in_open, in_close, index - 1);

            let bullish = current_color == 1
                && previous_color == -1
                && ((in_close[index] >= in_open[index - 1]
                    && in_open[index] < in_close[index - 1])
                    || (in_close[index] > in_open[index - 1]
                        && in_open[index] <= in_close[index - 1]));
            let bearish = current_color == -1
                && previous_color == 1
                && ((in_open[index] >= in_close[index - 1]
                    && in_close[index] < in_open[index - 1])
                    || (in_open[index] > in_close[index - 1]
                        && in_close[index] <= in_open[index - 1]));

            if bullish || bearish {
                if in_open[index] != in_close[index - 1] && in_close[index] != in_open[index - 1] {
                    current_color * 100
                } else {
                    current_color * 80
                }
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_harami_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 1
}

pub(crate) fn cdl_harami_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_harami_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let prev_max = in_close[index - 1].max(in_open[index - 1]);
            let prev_min = in_close[index - 1].min(in_open[index - 1]);
            let current_max = in_close[index].max(in_open[index]);
            let current_min = in_close[index].min(in_open[index]);

            if real_body(in_open, in_close, index - 1)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 1,
                )
                && real_body(in_open, in_close, index)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                if current_max < prev_max && current_min > prev_min {
                    -candle_color(in_open, in_close, index - 1) * 100
                } else if current_max <= prev_max && current_min >= prev_min {
                    -candle_color(in_open, in_close, index - 1) * 80
                } else {
                    0
                }
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_harami_cross_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::BodyDoji)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 1
}

pub(crate) fn cdl_harami_cross_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_harami_cross_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let prev_max = in_close[index - 1].max(in_open[index - 1]);
            let prev_min = in_close[index - 1].min(in_open[index - 1]);
            let current_max = in_close[index].max(in_open[index]);
            let current_min = in_close[index].min(in_open[index]);

            if real_body(in_open, in_close, index - 1)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 1,
                )
                && real_body(in_open, in_close, index)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyDoji,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                if current_max < prev_max && current_min > prev_min {
                    -candle_color(in_open, in_close, index - 1) * 100
                } else if current_max <= prev_max && current_min >= prev_min {
                    -candle_color(in_open, in_close, index - 1) * 80
                } else {
                    0
                }
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_belt_hold_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong).max(setting_avg_period(
        context,
        CandleSettingType::ShadowVeryShort,
    )) as i32
}

pub(crate) fn cdl_belt_hold_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_belt_hold_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let color = candle_color(in_open, in_close, index);
            if real_body(in_open, in_close, index)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && ((color == 1
                    && lower_shadow(in_open, in_low, in_close, index)
                        < candle_average(
                            context,
                            CandleSettingType::ShadowVeryShort,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index,
                        ))
                    || (color == -1
                        && upper_shadow(in_open, in_high, in_close, index)
                            < candle_average(
                                context,
                                CandleSettingType::ShadowVeryShort,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index,
                            )))
            {
                color * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_closing_marubozu_lookback(context: &Context) -> i32 {
    cdl_belt_hold_lookback(context)
}

pub(crate) fn cdl_closing_marubozu_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_closing_marubozu_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let color = candle_color(in_open, in_close, index);
            if real_body(in_open, in_close, index)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && ((color == 1
                    && upper_shadow(in_open, in_high, in_close, index)
                        < candle_average(
                            context,
                            CandleSettingType::ShadowVeryShort,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index,
                        ))
                    || (color == -1
                        && lower_shadow(in_open, in_low, in_close, index)
                            < candle_average(
                                context,
                                CandleSettingType::ShadowVeryShort,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index,
                            )))
            {
                color * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_high_wave_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyShort).max(setting_avg_period(
        context,
        CandleSettingType::ShadowVeryLong,
    )) as i32
}

pub(crate) fn cdl_high_wave_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_high_wave_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowVeryLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowVeryLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_inverted_hammer_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::ShadowLong))
        .max(setting_avg_period(
            context,
            CandleSettingType::ShadowVeryShort,
        ));
    lookback as i32 + 1
}

pub(crate) fn cdl_inverted_hammer_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_inverted_hammer_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && real_body_gap_down(in_open, in_close, index, index - 1)
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_shooting_star_lookback(context: &Context) -> i32 {
    cdl_inverted_hammer_lookback(context)
}

pub(crate) fn cdl_shooting_star_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_shooting_star_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && real_body_gap_up(in_open, in_close, index, index - 1)
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_takuri_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyDoji)
        .max(setting_avg_period(
            context,
            CandleSettingType::ShadowVeryShort,
        ))
        .max(setting_avg_period(
            context,
            CandleSettingType::ShadowVeryLong,
        )) as i32
}

pub(crate) fn cdl_takuri_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_takuri_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                <= candle_average(
                    context,
                    CandleSettingType::BodyDoji,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowVeryLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_counter_attack_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::Equal)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 1
}

pub(crate) fn cdl_counter_attack_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_counter_attack_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -candle_color(in_open, in_close, index)
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_close[index]
                    <= in_close[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Equal,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && in_close[index]
                    >= in_close[index - 1]
                        - candle_average(
                            context,
                            CandleSettingType::Equal,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_homing_pigeon_lookback(context: &Context) -> i32 {
    cdl_harami_lookback(context)
}

pub(crate) fn cdl_homing_pigeon_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_homing_pigeon_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -1
                && candle_color(in_open, in_close, index) == -1
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body(in_open, in_close, index)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_open[index] < in_open[index - 1]
                && in_close[index] > in_close[index - 1]
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_in_neck_lookback(context: &Context) -> i32 {
    cdl_counter_attack_lookback(context)
}

pub(crate) fn cdl_in_neck_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_in_neck_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -1
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_open[index] < in_low[index - 1]
                && in_close[index]
                    <= in_close[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Equal,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && in_close[index] >= in_close[index - 1]
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_on_neck_lookback(context: &Context) -> i32 {
    cdl_counter_attack_lookback(context)
}

pub(crate) fn cdl_on_neck_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_on_neck_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let equal = candle_average(
                context,
                CandleSettingType::Equal,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 1,
            );
            if candle_color(in_open, in_close, index - 1) == -1
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_open[index] < in_low[index - 1]
                && in_close[index] <= in_low[index - 1] + equal
                && in_close[index] >= in_low[index - 1] - equal
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_thrusting_lookback(context: &Context) -> i32 {
    cdl_counter_attack_lookback(context)
}

pub(crate) fn cdl_thrusting_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_thrusting_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -1
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_open[index] < in_low[index - 1]
                && in_close[index]
                    > in_close[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Equal,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && in_close[index]
                    <= in_close[index - 1] + real_body(in_open, in_close, index - 1) * 0.5
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_matching_low_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::Equal) as i32 + 1
}

pub(crate) fn cdl_matching_low_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_matching_low_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let equal = candle_average(
                context,
                CandleSettingType::Equal,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 1,
            );
            if candle_color(in_open, in_close, index - 1) == -1
                && candle_color(in_open, in_close, index) == -1
                && in_close[index] <= in_close[index - 1] + equal
                && in_close[index] >= in_close[index - 1] - equal
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_2_crows_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong) as i32 + 2
}

pub(crate) fn cdl_2_crows_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_2_crows_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2) == 1
                && real_body(in_open, in_close, index - 2)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && candle_color(in_open, in_close, index - 1) == -1
                && real_body_gap_up(in_open, in_close, index - 1, index - 2)
                && candle_color(in_open, in_close, index) == -1
                && in_open[index] < in_open[index - 1]
                && in_open[index] > in_close[index - 1]
                && in_close[index] > in_open[index - 2]
                && in_close[index] < in_close[index - 2]
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_3_black_crows_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort) as i32 + 3
}

pub(crate) fn cdl_3_black_crows_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_3_black_crows_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let short0 = candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index,
            );
            let short1 = candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 1,
            );
            let short2 = candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 2,
            );
            if candle_color(in_open, in_close, index - 3) == 1
                && candle_color(in_open, in_close, index - 2) == -1
                && lower_shadow(in_open, in_low, in_close, index - 2) < short2
                && candle_color(in_open, in_close, index - 1) == -1
                && lower_shadow(in_open, in_low, in_close, index - 1) < short1
                && candle_color(in_open, in_close, index) == -1
                && lower_shadow(in_open, in_low, in_close, index) < short0
                && in_open[index - 1] < in_open[index - 2]
                && in_open[index - 1] > in_close[index - 2]
                && in_open[index] < in_open[index - 1]
                && in_open[index] > in_close[index - 1]
                && in_high[index - 3] > in_close[index - 2]
                && in_close[index - 2] > in_close[index - 1]
                && in_close[index - 1] > in_close[index]
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_dark_cloud_cover_lookback(context: &Context, penetration: f64) -> i32 {
    if !(0.0..=3.0e37).contains(&penetration) {
        return -1;
    }
    setting_avg_period(context, CandleSettingType::BodyLong) as i32 + 1
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_dark_cloud_cover_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let penetration = if penetration == crate::REAL_DEFAULT {
        0.5
    } else {
        penetration
    };
    if !(0.0..=3.0e37).contains(&penetration) {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    }
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_dark_cloud_cover_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == 1
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == -1
                && in_open[index] > in_high[index - 1]
                && in_close[index] > in_open[index - 1]
                && in_close[index]
                    < in_close[index - 1] - real_body(in_open, in_close, index - 1) * penetration
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_piercing_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong) as i32 + 1
}

pub(crate) fn cdl_piercing_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_piercing_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -1
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == 1
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_open[index] < in_low[index - 1]
                && in_close[index] < in_open[index - 1]
                && in_close[index]
                    > in_close[index - 1] + real_body(in_open, in_close, index - 1) * 0.5
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_separating_lines_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::ShadowVeryShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong))
        .max(setting_avg_period(context, CandleSettingType::Equal));
    lookback as i32 + 1
}

pub(crate) fn cdl_separating_lines_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_separating_lines_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let equal = candle_average(
                context,
                CandleSettingType::Equal,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 1,
            );
            let color = candle_color(in_open, in_close, index);
            if candle_color(in_open, in_close, index - 1) == -color
                && in_open[index] <= in_open[index - 1] + equal
                && in_open[index] >= in_open[index - 1] - equal
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && ((color == 1
                    && lower_shadow(in_open, in_low, in_close, index)
                        < candle_average(
                            context,
                            CandleSettingType::ShadowVeryShort,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index,
                        ))
                    || (color == -1
                        && upper_shadow(in_open, in_high, in_close, index)
                            < candle_average(
                                context,
                                CandleSettingType::ShadowVeryShort,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index,
                            )))
            {
                color * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_stick_sandwich_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::Equal) as i32 + 2
}

pub(crate) fn cdl_stick_sandwich_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_stick_sandwich_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let equal = candle_average(
                context,
                CandleSettingType::Equal,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 2,
            );
            if candle_color(in_open, in_close, index - 2) == -1
                && candle_color(in_open, in_close, index - 1) == 1
                && candle_color(in_open, in_close, index) == -1
                && in_low[index - 1] > in_close[index - 2]
                && in_close[index] <= in_close[index - 2] + equal
                && in_close[index] >= in_close[index - 2] - equal
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_3_inside_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 2
}

pub(crate) fn cdl_3_inside_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_3_inside_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let first_max = in_close[index - 2].max(in_open[index - 2]);
            let first_min = in_close[index - 2].min(in_open[index - 2]);
            let second_max = in_close[index - 1].max(in_open[index - 1]);
            let second_min = in_close[index - 1].min(in_open[index - 1]);
            let first_color = candle_color(in_open, in_close, index - 2);
            let third_color = candle_color(in_open, in_close, index);
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && real_body(in_open, in_close, index - 1)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && second_max < first_max
                && second_min > first_min
                && ((first_color == 1 && third_color == -1 && in_close[index] < in_open[index - 2])
                    || (first_color == -1
                        && third_color == 1
                        && in_close[index] > in_open[index - 2]))
            {
                -first_color * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_3_outside_lookback(_context: &Context) -> i32 {
    3
}

pub(crate) fn cdl_3_outside_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_3_outside_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |_, in_open, _in_high, _in_low, in_close, index| {
            if (candle_color(in_open, in_close, index - 1) == 1
                && candle_color(in_open, in_close, index - 2) == -1
                && in_close[index - 1] > in_open[index - 2]
                && in_open[index - 1] < in_close[index - 2]
                && in_close[index] > in_close[index - 1])
                || (candle_color(in_open, in_close, index - 1) == -1
                    && candle_color(in_open, in_close, index - 2) == 1
                    && in_open[index - 1] > in_close[index - 2]
                    && in_close[index - 1] < in_open[index - 2]
                    && in_close[index] < in_close[index - 1])
            {
                candle_color(in_open, in_close, index - 1) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_hanging_man_lookback(context: &Context) -> i32 {
    cdl_hammer_lookback(context)
}

pub(crate) fn cdl_hanging_man_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_hanging_man_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                < candle_average(
                    context,
                    CandleSettingType::BodyShort,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && lower_shadow(in_open, in_low, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_close[index].min(in_open[index])
                    >= in_high[index - 1]
                        - candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_doji_star_lookback(context: &Context) -> i32 {
    let lookback = setting_avg_period(context, CandleSettingType::BodyDoji)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 1
}

pub(crate) fn cdl_doji_star_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_doji_star_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let prev_color = candle_color(in_open, in_close, index - 1);
            if real_body(in_open, in_close, index - 1)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 1,
                )
                && real_body(in_open, in_close, index)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyDoji,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && ((prev_color == 1 && real_body_gap_up(in_open, in_close, index, index - 1))
                    || (prev_color == -1
                        && real_body_gap_down(in_open, in_close, index, index - 1)))
            {
                -prev_color * 100
            } else {
                0
            }
        },
    )
}

fn normalize_penetration(value: f64, default_value: f64) -> Option<f64> {
    let normalized = if value == crate::REAL_DEFAULT {
        default_value
    } else {
        value
    };
    if !(0.0..=3.0e37).contains(&normalized) {
        return None;
    }
    Some(normalized)
}

pub(crate) fn cdl_evening_star_lookback(context: &Context, penetration: f64) -> i32 {
    if normalize_penetration(penetration, 0.3).is_none() {
        return -1;
    }
    let lookback = setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 2
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_evening_star_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let Some(penetration) = normalize_penetration(penetration, 0.3) else {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    };
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_evening_star_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && candle_color(in_open, in_close, index - 2) == 1
                && real_body(in_open, in_close, index - 1)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body_gap_up(in_open, in_close, index - 1, index - 2)
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && candle_color(in_open, in_close, index) == -1
                && in_close[index]
                    < in_close[index - 2] - real_body(in_open, in_close, index - 2) * penetration
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_evening_doji_star_lookback(context: &Context, penetration: f64) -> i32 {
    if normalize_penetration(penetration, 0.3).is_none() {
        return -1;
    }
    let lookback = setting_avg_period(context, CandleSettingType::BodyLong)
        .max(setting_avg_period(context, CandleSettingType::BodyDoji))
        .max(setting_avg_period(context, CandleSettingType::BodyShort));
    lookback as i32 + 2
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_evening_doji_star_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let Some(penetration) = normalize_penetration(penetration, 0.3) else {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    };
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_evening_doji_star_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && candle_color(in_open, in_close, index - 2) == 1
                && real_body(in_open, in_close, index - 1)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyDoji,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body_gap_up(in_open, in_close, index - 1, index - 2)
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && candle_color(in_open, in_close, index) == -1
                && in_close[index]
                    < in_close[index - 2] - real_body(in_open, in_close, index - 2) * penetration
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_morning_star_lookback(context: &Context, penetration: f64) -> i32 {
    if normalize_penetration(penetration, 0.3).is_none() {
        return -1;
    }
    let lookback = setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong));
    lookback as i32 + 2
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_morning_star_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let Some(penetration) = normalize_penetration(penetration, 0.3) else {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    };
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_morning_star_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && candle_color(in_open, in_close, index - 2) == -1
                && real_body(in_open, in_close, index - 1)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body_gap_down(in_open, in_close, index - 1, index - 2)
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_close[index]
                    > in_close[index - 2] + real_body(in_open, in_close, index - 2) * penetration
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_morning_doji_star_lookback(context: &Context, penetration: f64) -> i32 {
    if normalize_penetration(penetration, 0.3).is_none() {
        return -1;
    }
    let lookback = setting_avg_period(context, CandleSettingType::BodyLong)
        .max(setting_avg_period(context, CandleSettingType::BodyDoji))
        .max(setting_avg_period(context, CandleSettingType::BodyShort));
    lookback as i32 + 2
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_morning_doji_star_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let Some(penetration) = normalize_penetration(penetration, 0.3) else {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    };
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_morning_doji_star_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && candle_color(in_open, in_close, index - 2) == -1
                && real_body(in_open, in_close, index - 1)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyDoji,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body_gap_down(in_open, in_close, index - 1, index - 2)
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_close[index]
                    > in_close[index - 2] + real_body(in_open, in_close, index - 2) * penetration
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_long_legged_doji_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyDoji)
        .max(setting_avg_period(context, CandleSettingType::ShadowLong)) as i32
}

pub(crate) fn cdl_long_legged_doji_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_long_legged_doji_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index)
                <= candle_average(
                    context,
                    CandleSettingType::BodyDoji,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && (lower_shadow(in_open, in_low, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                    || upper_shadow(in_open, in_high, in_close, index)
                        > candle_average(
                            context,
                            CandleSettingType::ShadowLong,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index,
                        ))
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_rickshaw_man_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyDoji)
        .max(setting_avg_period(context, CandleSettingType::ShadowLong))
        .max(setting_avg_period(context, CandleSettingType::Near)) as i32
}

pub(crate) fn cdl_rickshaw_man_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_rickshaw_man_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let midpoint = in_low[index] + high_low_range(in_high, in_low, index) / 2.0;
            let near = candle_average(
                context,
                CandleSettingType::Near,
                in_open,
                in_high,
                in_low,
                in_close,
                index,
            );
            if real_body(in_open, in_close, index)
                <= candle_average(
                    context,
                    CandleSettingType::BodyDoji,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index,
                )
                && lower_shadow(in_open, in_low, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && upper_shadow(in_open, in_high, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_open[index].min(in_close[index]) <= midpoint + near
                && in_open[index].max(in_close[index]) >= midpoint - near
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_tristar_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyDoji) as i32 + 2
}

pub(crate) fn cdl_tristar_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_tristar_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let doji = candle_average(
                context,
                CandleSettingType::BodyDoji,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 2,
            );
            if real_body(in_open, in_close, index - 2) <= doji
                && real_body(in_open, in_close, index - 1) <= doji
                && real_body(in_open, in_close, index) <= doji
            {
                if real_body_gap_up(in_open, in_close, index - 1, index - 2)
                    && in_open[index].max(in_close[index])
                        < in_open[index - 1].max(in_close[index - 1])
                {
                    -100
                } else if real_body_gap_down(in_open, in_close, index - 1, index - 2)
                    && in_open[index].min(in_close[index])
                        > in_open[index - 1].min(in_close[index - 1])
                {
                    100
                } else {
                    0
                }
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_unique_3_river_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong)) as i32
        + 2
}

pub(crate) fn cdl_unique_3_river_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_unique_3_river_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && candle_color(in_open, in_close, index - 2) == -1
                && candle_color(in_open, in_close, index - 1) == -1
                && in_close[index - 1] > in_close[index - 2]
                && in_open[index - 1] <= in_open[index - 2]
                && in_low[index - 1] < in_low[index - 2]
                && real_body(in_open, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_open[index] > in_low[index - 1]
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_breakaway_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong) as i32 + 4
}

pub(crate) fn cdl_breakaway_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_breakaway_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 4)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 4,
                )
                && candle_color(in_open, in_close, index - 4)
                    == candle_color(in_open, in_close, index - 3)
                && candle_color(in_open, in_close, index - 3)
                    == candle_color(in_open, in_close, index - 1)
                && candle_color(in_open, in_close, index - 1)
                    == -candle_color(in_open, in_close, index)
                && ((candle_color(in_open, in_close, index - 4) == -1
                    && real_body_gap_down(in_open, in_close, index - 3, index - 4)
                    && in_high[index - 2] < in_high[index - 3]
                    && in_low[index - 2] < in_low[index - 3]
                    && in_high[index - 1] < in_high[index - 2]
                    && in_low[index - 1] < in_low[index - 2]
                    && in_close[index] > in_open[index - 3]
                    && in_close[index] < in_close[index - 4])
                    || (candle_color(in_open, in_close, index - 4) == 1
                        && real_body_gap_up(in_open, in_close, index - 3, index - 4)
                        && in_high[index - 2] > in_high[index - 3]
                        && in_low[index - 2] > in_low[index - 3]
                        && in_high[index - 1] > in_high[index - 2]
                        && in_low[index - 1] > in_low[index - 2]
                        && in_close[index] < in_open[index - 3]
                        && in_close[index] > in_close[index - 4]))
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_tasuki_gap_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::Near) as i32 + 2
}

pub(crate) fn cdl_tasuki_gap_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_tasuki_gap_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let near = candle_average(
                context,
                CandleSettingType::Near,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 1,
            );
            if ((real_body_gap_up(in_open, in_close, index - 1, index - 2)
                && candle_color(in_open, in_close, index - 1) == 1
                && candle_color(in_open, in_close, index) == -1
                && in_open[index] < in_close[index - 1]
                && in_open[index] > in_open[index - 1]
                && in_close[index] < in_open[index - 1]
                && in_close[index] > in_close[index - 2])
                || (real_body_gap_down(in_open, in_close, index - 1, index - 2)
                    && candle_color(in_open, in_close, index - 1) == -1
                    && candle_color(in_open, in_close, index) == 1
                    && in_open[index] > in_close[index - 1]
                    && in_open[index] < in_open[index - 1]
                    && in_close[index] > in_open[index - 1]
                    && in_close[index] < in_close[index - 2]))
                && (real_body(in_open, in_close, index - 1) - real_body(in_open, in_close, index))
                    .abs()
                    < near
            {
                candle_color(in_open, in_close, index - 1) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_upside_gap_2_crows_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong) as i32 + 2
}

pub(crate) fn cdl_upside_gap_2_crows_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_upside_gap_2_crows_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2) == 1
                && real_body(in_open, in_close, index - 2)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && candle_color(in_open, in_close, index - 1) == -1
                && real_body_gap_up(in_open, in_close, index - 1, index - 2)
                && candle_color(in_open, in_close, index) == -1
                && in_open[index] > in_open[index - 1]
                && in_close[index] < in_close[index - 1]
                && in_close[index] > in_close[index - 2]
                && in_open[index] < in_close[index - 2]
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_kicking_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong)) as i32
        + 1
}

fn is_marubozu_like(
    context: &Context,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    index: usize,
) -> bool {
    real_body(in_open, in_close, index)
        > candle_average(
            context,
            CandleSettingType::BodyLong,
            in_open,
            in_high,
            in_low,
            in_close,
            index,
        )
        && upper_shadow(in_open, in_high, in_close, index)
            < candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index,
            )
        && lower_shadow(in_open, in_low, in_close, index)
            < candle_average(
                context,
                CandleSettingType::ShadowVeryShort,
                in_open,
                in_high,
                in_low,
                in_close,
                index,
            )
}

pub(crate) fn cdl_kicking_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_kicking_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -candle_color(in_open, in_close, index)
                && is_marubozu_like(context, in_open, in_high, in_low, in_close, index - 1)
                && is_marubozu_like(context, in_open, in_high, in_low, in_close, index)
                && ((candle_color(in_open, in_close, index - 1) == -1
                    && candle_gap_up(in_high, in_low, index, index - 1))
                    || (candle_color(in_open, in_close, index - 1) == 1
                        && candle_gap_down(in_high, in_low, index, index - 1)))
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_kicking_by_length_lookback(context: &Context) -> i32 {
    cdl_kicking_lookback(context)
}

pub(crate) fn cdl_kicking_by_length_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_kicking_by_length_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 1) == -candle_color(in_open, in_close, index)
                && is_marubozu_like(context, in_open, in_high, in_low, in_close, index - 1)
                && is_marubozu_like(context, in_open, in_high, in_low, in_close, index)
                && ((candle_color(in_open, in_close, index - 1) == -1
                    && candle_gap_up(in_high, in_low, index, index - 1))
                    || (candle_color(in_open, in_close, index - 1) == 1
                        && candle_gap_down(in_high, in_low, index, index - 1)))
            {
                if real_body(in_open, in_close, index) > real_body(in_open, in_close, index - 1) {
                    candle_color(in_open, in_close, index) * 100
                } else {
                    candle_color(in_open, in_close, index - 1) * 100
                }
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_rise_fall_3_methods_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong)) as i32
        + 4
}

pub(crate) fn cdl_rise_fall_3_methods_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_rise_fall_3_methods_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let first_color = candle_color(in_open, in_close, index - 4);
            if real_body(in_open, in_close, index - 4)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 4,
                )
                && real_body(in_open, in_close, index - 3)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 3,
                    )
                && real_body(in_open, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && real_body(in_open, in_close, index - 1)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && candle_color(in_open, in_close, index - 3) == -first_color
                && candle_color(in_open, in_close, index - 2)
                    == candle_color(in_open, in_close, index - 3)
                && candle_color(in_open, in_close, index - 1)
                    == candle_color(in_open, in_close, index - 3)
                && candle_color(in_open, in_close, index) == first_color
                && in_open[index - 3].min(in_close[index - 3]) < in_high[index - 4]
                && in_open[index - 3].max(in_close[index - 3]) > in_low[index - 4]
                && in_open[index - 2].min(in_close[index - 2]) < in_high[index - 4]
                && in_open[index - 2].max(in_close[index - 2]) > in_low[index - 4]
                && in_open[index - 1].min(in_close[index - 1]) < in_high[index - 4]
                && in_open[index - 1].max(in_close[index - 1]) > in_low[index - 4]
                && in_close[index - 2] * f64::from(first_color)
                    < in_close[index - 3] * f64::from(first_color)
                && in_close[index - 1] * f64::from(first_color)
                    < in_close[index - 2] * f64::from(first_color)
                && in_open[index] * f64::from(first_color)
                    > in_close[index - 1] * f64::from(first_color)
                && in_close[index] * f64::from(first_color)
                    > in_close[index - 4] * f64::from(first_color)
            {
                100 * first_color
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_mat_hold_lookback(context: &Context, penetration: f64) -> i32 {
    if normalize_penetration(penetration, 0.5).is_none() {
        return -1;
    }
    setting_avg_period(context, CandleSettingType::BodyShort)
        .max(setting_avg_period(context, CandleSettingType::BodyLong)) as i32
        + 4
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_mat_hold_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let Some(penetration) = normalize_penetration(penetration, 0.5) else {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    };
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_mat_hold_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 4)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 4,
                )
                && real_body(in_open, in_close, index - 3)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 3,
                    )
                && real_body(in_open, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && real_body(in_open, in_close, index - 1)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index - 4) == 1
                && candle_color(in_open, in_close, index - 3) == -1
                && candle_color(in_open, in_close, index) == 1
                && real_body_gap_up(in_open, in_close, index - 3, index - 4)
                && in_open[index - 2].min(in_close[index - 2]) < in_close[index - 4]
                && in_open[index - 1].min(in_close[index - 1]) < in_close[index - 4]
                && in_open[index - 2].min(in_close[index - 2])
                    > in_close[index - 4] - real_body(in_open, in_close, index - 4) * penetration
                && in_open[index - 1].min(in_close[index - 1])
                    > in_close[index - 4] - real_body(in_open, in_close, index - 4) * penetration
                && in_open[index - 2].max(in_close[index - 2]) < in_open[index - 3]
                && in_open[index - 1].max(in_close[index - 1])
                    < in_open[index - 2].max(in_close[index - 2])
                && in_open[index] > in_close[index - 1]
                && in_close[index]
                    > in_high[index - 3]
                        .max(in_high[index - 2])
                        .max(in_high[index - 1])
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_identical_3_crows_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort)
        .max(setting_avg_period(context, CandleSettingType::Equal)) as i32
        + 2
}

pub(crate) fn cdl_identical_3_crows_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_identical_3_crows_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let equal_2 = candle_average(
                context,
                CandleSettingType::Equal,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 2,
            );
            let equal_1 = candle_average(
                context,
                CandleSettingType::Equal,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 1,
            );
            if candle_color(in_open, in_close, index - 2) == -1
                && lower_shadow(in_open, in_low, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && candle_color(in_open, in_close, index - 1) == -1
                && lower_shadow(in_open, in_low, in_close, index - 1)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == -1
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_close[index - 2] > in_close[index - 1]
                && in_close[index - 1] > in_close[index]
                && in_open[index - 1] <= in_close[index - 2] + equal_2
                && in_open[index - 1] >= in_close[index - 2] - equal_2
                && in_open[index] <= in_close[index - 1] + equal_1
                && in_open[index] >= in_close[index - 1] - equal_1
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_3_white_soldiers_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort)
        .max(setting_avg_period(context, CandleSettingType::BodyShort))
        .max(setting_avg_period(context, CandleSettingType::Far))
        .max(setting_avg_period(context, CandleSettingType::Near)) as i32
        + 2
}

pub(crate) fn cdl_3_white_soldiers_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_3_white_soldiers_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2) == 1
                && upper_shadow(in_open, in_high, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && candle_color(in_open, in_close, index - 1) == 1
                && upper_shadow(in_open, in_high, in_close, index - 1)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == 1
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_close[index] > in_close[index - 1]
                && in_close[index - 1] > in_close[index - 2]
                && in_open[index - 1] > in_open[index - 2]
                && in_open[index - 1]
                    <= in_close[index - 2]
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 2,
                        )
                && in_open[index] > in_open[index - 1]
                && in_open[index]
                    <= in_close[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && real_body(in_open, in_close, index - 1)
                    > real_body(in_open, in_close, index - 2)
                        - candle_average(
                            context,
                            CandleSettingType::Far,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 2,
                        )
                && real_body(in_open, in_close, index)
                    > real_body(in_open, in_close, index - 1)
                        - candle_average(
                            context,
                            CandleSettingType::Far,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_advance_block_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowLong)
        .max(setting_avg_period(context, CandleSettingType::ShadowShort))
        .max(setting_avg_period(context, CandleSettingType::Far))
        .max(setting_avg_period(context, CandleSettingType::Near))
        .max(setting_avg_period(context, CandleSettingType::BodyLong)) as i32
        + 2
}

pub(crate) fn cdl_advance_block_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_advance_block_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2) == 1
                && candle_color(in_open, in_close, index - 1) == 1
                && candle_color(in_open, in_close, index) == 1
                && in_close[index] > in_close[index - 1]
                && in_close[index - 1] > in_close[index - 2]
                && in_open[index - 1] > in_open[index - 2]
                && in_open[index - 1]
                    <= in_close[index - 2]
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 2,
                        )
                && in_open[index] > in_open[index - 1]
                && in_open[index]
                    <= in_close[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && real_body(in_open, in_close, index - 2)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && upper_shadow(in_open, in_high, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && ((real_body(in_open, in_close, index - 1)
                    < real_body(in_open, in_close, index - 2)
                        - candle_average(
                            context,
                            CandleSettingType::Far,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 2,
                        )
                    && real_body(in_open, in_close, index)
                        < real_body(in_open, in_close, index - 1)
                            + candle_average(
                                context,
                                CandleSettingType::Near,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index - 1,
                            ))
                    || (real_body(in_open, in_close, index)
                        < real_body(in_open, in_close, index - 1)
                            - candle_average(
                                context,
                                CandleSettingType::Far,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index - 1,
                            ))
                    || (real_body(in_open, in_close, index)
                        < real_body(in_open, in_close, index - 1)
                        && real_body(in_open, in_close, index - 1)
                            < real_body(in_open, in_close, index - 2)
                        && (upper_shadow(in_open, in_high, in_close, index)
                            > candle_average(
                                context,
                                CandleSettingType::ShadowShort,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index,
                            )
                            || upper_shadow(in_open, in_high, in_close, index - 1)
                                > candle_average(
                                    context,
                                    CandleSettingType::ShadowShort,
                                    in_open,
                                    in_high,
                                    in_low,
                                    in_close,
                                    index - 1,
                                )))
                    || (real_body(in_open, in_close, index)
                        < real_body(in_open, in_close, index - 1)
                        && upper_shadow(in_open, in_high, in_close, index)
                            > candle_average(
                                context,
                                CandleSettingType::ShadowLong,
                                in_open,
                                in_high,
                                in_low,
                                in_close,
                                index,
                            )))
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_stalled_pattern_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::BodyLong)
        .max(setting_avg_period(context, CandleSettingType::BodyShort))
        .max(setting_avg_period(
            context,
            CandleSettingType::ShadowVeryShort,
        ))
        .max(setting_avg_period(context, CandleSettingType::Near)) as i32
        + 2
}

pub(crate) fn cdl_stalled_pattern_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_stalled_pattern_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2) == 1
                && candle_color(in_open, in_close, index - 1) == 1
                && candle_color(in_open, in_close, index) == 1
                && in_close[index] > in_close[index - 1]
                && in_close[index - 1] > in_close[index - 2]
                && real_body(in_open, in_close, index - 2)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && real_body(in_open, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && upper_shadow(in_open, in_high, in_close, index - 1)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && in_open[index - 1] > in_open[index - 2]
                && in_open[index - 1]
                    <= in_close[index - 2]
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 2,
                        )
                && real_body(in_open, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_open[index]
                    >= in_close[index - 1]
                        - real_body(in_open, in_close, index)
                        - candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
            {
                -100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_gap_side_side_white_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::Near)
        .max(setting_avg_period(context, CandleSettingType::Equal)) as i32
        + 2
}

pub(crate) fn cdl_gap_side_side_white_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_gap_side_side_white_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if ((real_body_gap_up(in_open, in_close, index - 1, index - 2)
                && real_body_gap_up(in_open, in_close, index, index - 2))
                || (real_body_gap_down(in_open, in_close, index - 1, index - 2)
                    && real_body_gap_down(in_open, in_close, index, index - 2)))
                && candle_color(in_open, in_close, index - 1) == 1
                && candle_color(in_open, in_close, index) == 1
                && real_body(in_open, in_close, index)
                    >= real_body(in_open, in_close, index - 1)
                        - candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && real_body(in_open, in_close, index)
                    <= real_body(in_open, in_close, index - 1)
                        + candle_average(
                            context,
                            CandleSettingType::Near,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && in_open[index]
                    >= in_open[index - 1]
                        - candle_average(
                            context,
                            CandleSettingType::Equal,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
                && in_open[index]
                    <= in_open[index - 1]
                        + candle_average(
                            context,
                            CandleSettingType::Equal,
                            in_open,
                            in_high,
                            in_low,
                            in_close,
                            index - 1,
                        )
            {
                if real_body_gap_up(in_open, in_close, index - 1, index - 2) {
                    100
                } else {
                    -100
                }
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_3_stars_in_south_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort)
        .max(setting_avg_period(context, CandleSettingType::ShadowLong))
        .max(setting_avg_period(context, CandleSettingType::BodyLong))
        .max(setting_avg_period(context, CandleSettingType::BodyShort)) as i32
        + 2
}

pub(crate) fn cdl_3_stars_in_south_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_3_stars_in_south_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2) == -1
                && candle_color(in_open, in_close, index - 1) == -1
                && candle_color(in_open, in_close, index) == -1
                && real_body(in_open, in_close, index - 2)
                    > candle_average(
                        context,
                        CandleSettingType::BodyLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && lower_shadow(in_open, in_low, in_close, index - 2)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowLong,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && real_body(in_open, in_close, index - 1) < real_body(in_open, in_close, index - 2)
                && in_open[index - 1] > in_close[index - 2]
                && in_open[index - 1] <= in_high[index - 2]
                && in_low[index - 1] < in_close[index - 2]
                && in_low[index - 1] >= in_low[index - 2]
                && lower_shadow(in_open, in_low, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body(in_open, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && lower_shadow(in_open, in_low, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && upper_shadow(in_open, in_high, in_close, index)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && in_low[index] > in_low[index - 1]
                && in_high[index] < in_high[index - 1]
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_abandoned_baby_lookback(context: &Context, penetration: f64) -> i32 {
    if normalize_penetration(penetration, 0.3).is_none() {
        return -1;
    }
    setting_avg_period(context, CandleSettingType::BodyDoji)
        .max(setting_avg_period(context, CandleSettingType::BodyLong))
        .max(setting_avg_period(context, CandleSettingType::BodyShort)) as i32
        + 2
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cdl_abandoned_baby_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    penetration: f64,
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    let Some(penetration) = normalize_penetration(penetration, 0.3) else {
        *out_beg_idx = 0;
        *out_nb_element = 0;
        return RetCode::BadParam;
    };
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_abandoned_baby_lookback(context, penetration) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if real_body(in_open, in_close, index - 2)
                > candle_average(
                    context,
                    CandleSettingType::BodyLong,
                    in_open,
                    in_high,
                    in_low,
                    in_close,
                    index - 2,
                )
                && real_body(in_open, in_close, index - 1)
                    <= candle_average(
                        context,
                        CandleSettingType::BodyDoji,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && real_body(in_open, in_close, index)
                    > candle_average(
                        context,
                        CandleSettingType::BodyShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index,
                    )
                && ((candle_color(in_open, in_close, index - 2) == 1
                    && candle_color(in_open, in_close, index) == -1
                    && in_close[index]
                        < in_close[index - 2]
                            - real_body(in_open, in_close, index - 2) * penetration
                    && candle_gap_up(in_high, in_low, index - 1, index - 2)
                    && candle_gap_down(in_high, in_low, index, index - 1))
                    || (candle_color(in_open, in_close, index - 2) == -1
                        && candle_color(in_open, in_close, index) == 1
                        && in_close[index]
                            > in_close[index - 2]
                                + real_body(in_open, in_close, index - 2) * penetration
                        && candle_gap_down(in_high, in_low, index - 1, index - 2)
                        && candle_gap_up(in_high, in_low, index, index - 1)))
            {
                candle_color(in_open, in_close, index) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_conceal_babys_wall_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort) as i32 + 3
}

pub(crate) fn cdl_conceal_babys_wall_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_conceal_babys_wall_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 3) == -1
                && candle_color(in_open, in_close, index - 2) == -1
                && candle_color(in_open, in_close, index - 1) == -1
                && candle_color(in_open, in_close, index) == -1
                && lower_shadow(in_open, in_low, in_close, index - 3)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 3,
                    )
                && upper_shadow(in_open, in_high, in_close, index - 3)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 3,
                    )
                && lower_shadow(in_open, in_low, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && upper_shadow(in_open, in_high, in_close, index - 2)
                    < candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 2,
                    )
                && real_body_gap_down(in_open, in_close, index - 1, index - 2)
                && upper_shadow(in_open, in_high, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && in_high[index - 1] > in_close[index - 2]
                && in_high[index] > in_high[index - 1]
                && in_low[index] < in_low[index - 1]
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_hikkake_lookback(_context: &Context) -> i32 {
    5
}

pub(crate) fn cdl_hikkake_run(
    _context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_ohlc(start_idx, end_idx, in_open, in_high, in_low, in_close) {
        return ret_code;
    }
    let adjusted_start = start_idx.max(cdl_hikkake_lookback(&Context::new()) as usize);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }
    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_integer, needed) {
        return ret_code;
    }

    let mut pattern_idx = 0usize;
    let mut pattern_result = 0i32;
    let mut i = adjusted_start - 3;
    while i < adjusted_start {
        if in_high[i - 1] < in_high[i - 2]
            && in_low[i - 1] > in_low[i - 2]
            && ((in_high[i] < in_high[i - 1] && in_low[i] < in_low[i - 1])
                || (in_high[i] > in_high[i - 1] && in_low[i] > in_low[i - 1]))
        {
            pattern_result = 100 * if in_high[i] < in_high[i - 1] { 1 } else { -1 };
            pattern_idx = i;
        } else if i <= pattern_idx + 3
            && ((pattern_result > 0 && in_close[i] > in_high[pattern_idx - 1])
                || (pattern_result < 0 && in_close[i] < in_low[pattern_idx - 1]))
        {
            pattern_idx = 0;
        }
        i += 1;
    }

    i = adjusted_start;
    for out in out_integer.iter_mut().take(needed) {
        if in_high[i - 1] < in_high[i - 2]
            && in_low[i - 1] > in_low[i - 2]
            && ((in_high[i] < in_high[i - 1] && in_low[i] < in_low[i - 1])
                || (in_high[i] > in_high[i - 1] && in_low[i] > in_low[i - 1]))
        {
            pattern_result = 100 * if in_high[i] < in_high[i - 1] { 1 } else { -1 };
            pattern_idx = i;
            *out = pattern_result;
        } else if i <= pattern_idx + 3
            && ((pattern_result > 0 && in_close[i] > in_high[pattern_idx - 1])
                || (pattern_result < 0 && in_close[i] < in_low[pattern_idx - 1]))
        {
            *out = pattern_result + 100 * if pattern_result > 0 { 1 } else { -1 };
            pattern_idx = 0;
        } else {
            *out = 0;
        }
        i += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn cdl_hikkake_mod_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::Near).max(1) as i32 + 5
}

pub(crate) fn cdl_hikkake_mod_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_ohlc(start_idx, end_idx, in_open, in_high, in_low, in_close) {
        return ret_code;
    }
    let adjusted_start = start_idx.max(cdl_hikkake_mod_lookback(context) as usize);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }
    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_integer, needed) {
        return ret_code;
    }

    let mut pattern_idx = 0usize;
    let mut pattern_result = 0i32;
    let mut i = adjusted_start - 3;
    while i < adjusted_start {
        let near = candle_average(
            context,
            CandleSettingType::Near,
            in_open,
            in_high,
            in_low,
            in_close,
            i - 2,
        );
        if in_high[i - 2] < in_high[i - 3]
            && in_low[i - 2] > in_low[i - 3]
            && in_high[i - 1] < in_high[i - 2]
            && in_low[i - 1] > in_low[i - 2]
            && ((in_high[i] < in_high[i - 1]
                && in_low[i] < in_low[i - 1]
                && in_close[i - 2] <= in_low[i - 2] + near)
                || (in_high[i] > in_high[i - 1]
                    && in_low[i] > in_low[i - 1]
                    && in_close[i - 2] >= in_high[i - 2] - near))
        {
            pattern_result = 100 * if in_high[i] < in_high[i - 1] { 1 } else { -1 };
            pattern_idx = i;
        } else if i <= pattern_idx + 3
            && ((pattern_result > 0 && in_close[i] > in_high[pattern_idx - 1])
                || (pattern_result < 0 && in_close[i] < in_low[pattern_idx - 1]))
        {
            pattern_idx = 0;
        }
        i += 1;
    }

    i = adjusted_start;
    for out in out_integer.iter_mut().take(needed) {
        let near = candle_average(
            context,
            CandleSettingType::Near,
            in_open,
            in_high,
            in_low,
            in_close,
            i - 2,
        );
        if in_high[i - 2] < in_high[i - 3]
            && in_low[i - 2] > in_low[i - 3]
            && in_high[i - 1] < in_high[i - 2]
            && in_low[i - 1] > in_low[i - 2]
            && ((in_high[i] < in_high[i - 1]
                && in_low[i] < in_low[i - 1]
                && in_close[i - 2] <= in_low[i - 2] + near)
                || (in_high[i] > in_high[i - 1]
                    && in_low[i] > in_low[i - 1]
                    && in_close[i - 2] >= in_high[i - 2] - near))
        {
            pattern_result = 100 * if in_high[i] < in_high[i - 1] { 1 } else { -1 };
            pattern_idx = i;
            *out = pattern_result;
        } else if i <= pattern_idx + 3
            && ((pattern_result > 0 && in_close[i] > in_high[pattern_idx - 1])
                || (pattern_result < 0 && in_close[i] < in_low[pattern_idx - 1]))
        {
            *out = pattern_result + 100 * if pattern_result > 0 { 1 } else { -1 };
            pattern_idx = 0;
        } else {
            *out = 0;
        }
        i += 1;
    }

    *out_beg_idx = adjusted_start;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn cdl_ladder_bottom_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::ShadowVeryShort) as i32 + 4
}

pub(crate) fn cdl_ladder_bottom_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_ladder_bottom_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 4) == -1
                && candle_color(in_open, in_close, index - 3) == -1
                && candle_color(in_open, in_close, index - 2) == -1
                && in_open[index - 4] > in_open[index - 3]
                && in_open[index - 3] > in_open[index - 2]
                && in_close[index - 4] > in_close[index - 3]
                && in_close[index - 3] > in_close[index - 2]
                && candle_color(in_open, in_close, index - 1) == -1
                && upper_shadow(in_open, in_high, in_close, index - 1)
                    > candle_average(
                        context,
                        CandleSettingType::ShadowVeryShort,
                        in_open,
                        in_high,
                        in_low,
                        in_close,
                        index - 1,
                    )
                && candle_color(in_open, in_close, index) == 1
                && in_open[index] > in_open[index - 1]
                && in_close[index] > in_high[index - 1]
            {
                100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_x_side_gap_3_methods_lookback(_context: &Context) -> i32 {
    2
}

pub(crate) fn cdl_x_side_gap_3_methods_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_x_side_gap_3_methods_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |_, in_open, _in_high, _in_low, in_close, index| {
            if candle_color(in_open, in_close, index - 2)
                == candle_color(in_open, in_close, index - 1)
                && candle_color(in_open, in_close, index - 1)
                    == -candle_color(in_open, in_close, index)
                && in_open[index] < in_close[index - 1].max(in_open[index - 1])
                && in_open[index] > in_close[index - 1].min(in_open[index - 1])
                && in_close[index] < in_close[index - 2].max(in_open[index - 2])
                && in_close[index] > in_close[index - 2].min(in_open[index - 2])
                && ((candle_color(in_open, in_close, index - 2) == 1
                    && real_body_gap_up(in_open, in_close, index - 1, index - 2))
                    || (candle_color(in_open, in_close, index - 2) == -1
                        && real_body_gap_down(in_open, in_close, index - 1, index - 2)))
            {
                candle_color(in_open, in_close, index - 2) * 100
            } else {
                0
            }
        },
    )
}

pub(crate) fn cdl_3_line_strike_lookback(context: &Context) -> i32 {
    setting_avg_period(context, CandleSettingType::Near) as i32 + 3
}

pub(crate) fn cdl_3_line_strike_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_open: &[f64],
    in_high: &[f64],
    in_low: &[f64],
    in_close: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_integer: &mut [i32],
) -> RetCode {
    run_pattern(
        context,
        start_idx,
        end_idx,
        in_open,
        in_high,
        in_low,
        in_close,
        cdl_3_line_strike_lookback(context) as usize,
        out_beg_idx,
        out_nb_element,
        out_integer,
        |context, in_open, in_high, in_low, in_close, index| {
            let near_3 = candle_average(
                context,
                CandleSettingType::Near,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 3,
            );
            let near_2 = candle_average(
                context,
                CandleSettingType::Near,
                in_open,
                in_high,
                in_low,
                in_close,
                index - 2,
            );
            if candle_color(in_open, in_close, index - 3)
                == candle_color(in_open, in_close, index - 2)
                && candle_color(in_open, in_close, index - 2)
                    == candle_color(in_open, in_close, index - 1)
                && candle_color(in_open, in_close, index)
                    == -candle_color(in_open, in_close, index - 1)
                && in_open[index - 2] >= in_open[index - 3].min(in_close[index - 3]) - near_3
                && in_open[index - 2] <= in_open[index - 3].max(in_close[index - 3]) + near_3
                && in_open[index - 1] >= in_open[index - 2].min(in_close[index - 2]) - near_2
                && in_open[index - 1] <= in_open[index - 2].max(in_close[index - 2]) + near_2
                && ((candle_color(in_open, in_close, index - 1) == 1
                    && in_close[index - 1] > in_close[index - 2]
                    && in_close[index - 2] > in_close[index - 3]
                    && in_open[index] > in_close[index - 1]
                    && in_close[index] < in_open[index - 3])
                    || (candle_color(in_open, in_close, index - 1) == -1
                        && in_close[index - 1] < in_close[index - 2]
                        && in_close[index - 2] < in_close[index - 3]
                        && in_open[index] < in_close[index - 1]
                        && in_close[index] > in_open[index - 3]))
            {
                candle_color(in_open, in_close, index - 1) * 100
            } else {
                0
            }
        },
    )
}

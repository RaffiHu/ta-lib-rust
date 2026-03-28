use crate::helpers::{validate_input_len, validate_output_len, validate_range};
use crate::{Context, FuncUnstId, RetCode};

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

    fn transform_even(&mut self, input: f64, hilbert_idx: usize, adjusted_prev_period: f64) -> f64 {
        let a = 0.0962;
        let b = 0.5769;
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

    fn transform_odd(&mut self, input: f64, hilbert_idx: usize, adjusted_prev_period: f64) -> f64 {
        let a = 0.0962;
        let b = 0.5769;
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

struct PriceWma4<'a> {
    in_real: &'a [f64],
    trailing_idx: usize,
    period_sub: f64,
    period_sum: f64,
    trailing_value: f64,
}

impl<'a> PriceWma4<'a> {
    fn new(in_real: &'a [f64], start_idx: usize) -> Self {
        let mut today = start_idx;
        let mut temp_real = in_real[today];
        today += 1;
        let mut period_sub = temp_real;
        let mut period_sum = temp_real;
        temp_real = in_real[today];
        today += 1;
        period_sub += temp_real;
        period_sum += temp_real * 2.0;
        temp_real = in_real[today];
        period_sub += temp_real;
        period_sum += temp_real * 3.0;

        Self {
            in_real,
            trailing_idx: start_idx,
            period_sub,
            period_sum,
            trailing_value: 0.0,
        }
    }

    fn next(&mut self, new_price: f64) -> f64 {
        self.period_sub += new_price;
        self.period_sub -= self.trailing_value;
        self.period_sum += new_price * 4.0;
        self.trailing_value = self.in_real[self.trailing_idx];
        self.trailing_idx += 1;
        let smoothed_value = self.period_sum * 0.1;
        self.period_sum -= self.period_sub;
        smoothed_value
    }
}

#[derive(Debug, Clone)]
struct HilbertCore {
    hilbert_idx: usize,
    detrender: HilbertState,
    q1: HilbertState,
    ji: HilbertState,
    jq: HilbertState,
    period: f64,
    prev_i2: f64,
    prev_q2: f64,
    re: f64,
    im: f64,
    i1_odd_prev2: f64,
    i1_odd_prev3: f64,
    i1_even_prev2: f64,
    i1_even_prev3: f64,
}

#[derive(Debug, Clone, Copy)]
struct HilbertStep {
    q1: f64,
    i1_prev3: f64,
}

impl HilbertCore {
    fn new() -> Self {
        Self {
            hilbert_idx: 0,
            detrender: HilbertState::new(),
            q1: HilbertState::new(),
            ji: HilbertState::new(),
            jq: HilbertState::new(),
            period: 0.0,
            prev_i2: 0.0,
            prev_q2: 0.0,
            re: 0.0,
            im: 0.0,
            i1_odd_prev2: 0.0,
            i1_odd_prev3: 0.0,
            i1_even_prev2: 0.0,
            i1_even_prev3: 0.0,
        }
    }

    fn step(&mut self, today: usize, smoothed_value: f64) -> HilbertStep {
        let adjusted_prev_period = (0.075 * self.period) + 0.54;

        let (q1_value, q2, i2, i1_prev3) = if today % 2 == 0 {
            let detrender = self.detrender.transform_even(
                smoothed_value,
                self.hilbert_idx,
                adjusted_prev_period,
            );
            let q1_value =
                self.q1
                    .transform_even(detrender, self.hilbert_idx, adjusted_prev_period);
            let ji =
                self.ji
                    .transform_even(self.i1_even_prev3, self.hilbert_idx, adjusted_prev_period);
            let jq = self
                .jq
                .transform_even(q1_value, self.hilbert_idx, adjusted_prev_period);
            self.hilbert_idx += 1;
            if self.hilbert_idx == 3 {
                self.hilbert_idx = 0;
            }
            let q2 = (0.2 * (q1_value + ji)) + (0.8 * self.prev_q2);
            let i2 = (0.2 * (self.i1_even_prev3 - jq)) + (0.8 * self.prev_i2);
            let i1_prev3 = self.i1_even_prev3;
            self.i1_odd_prev3 = self.i1_odd_prev2;
            self.i1_odd_prev2 = detrender;
            (q1_value, q2, i2, i1_prev3)
        } else {
            let detrender = self.detrender.transform_odd(
                smoothed_value,
                self.hilbert_idx,
                adjusted_prev_period,
            );
            let q1_value = self
                .q1
                .transform_odd(detrender, self.hilbert_idx, adjusted_prev_period);
            let ji =
                self.ji
                    .transform_odd(self.i1_odd_prev3, self.hilbert_idx, adjusted_prev_period);
            let jq = self
                .jq
                .transform_odd(q1_value, self.hilbert_idx, adjusted_prev_period);
            let q2 = (0.2 * (q1_value + ji)) + (0.8 * self.prev_q2);
            let i2 = (0.2 * (self.i1_odd_prev3 - jq)) + (0.8 * self.prev_i2);
            let i1_prev3 = self.i1_odd_prev3;
            self.i1_even_prev3 = self.i1_even_prev2;
            self.i1_even_prev2 = detrender;
            (q1_value, q2, i2, i1_prev3)
        };

        self.re = (0.2 * ((i2 * self.prev_i2) + (q2 * self.prev_q2))) + (0.8 * self.re);
        self.im = (0.2 * ((i2 * self.prev_q2) - (q2 * self.prev_i2))) + (0.8 * self.im);
        self.prev_q2 = q2;
        self.prev_i2 = i2;

        let previous_period = self.period;
        if self.im != 0.0 && self.re != 0.0 {
            let rad2deg = 180.0 / (4.0 * 1.0f64.atan());
            self.period = 360.0 / ((self.im / self.re).atan() * rad2deg);
        }
        let mut temp = 1.5 * previous_period;
        if self.period > temp {
            self.period = temp;
        }
        temp = 0.67 * previous_period;
        if self.period < temp {
            self.period = temp;
        }
        self.period = self.period.clamp(6.0, 50.0);
        self.period = (0.2 * self.period) + (0.8 * previous_period);

        HilbertStep {
            q1: q1_value,
            i1_prev3,
        }
    }
}

pub(crate) fn ht_dcperiod_lookback(context: &Context) -> i32 {
    32 + context.get_unstable_period(FuncUnstId::HtDcPeriod) as i32
}

pub(crate) fn ht_dcperiod_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
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

    let lookback_total = 32 + context.get_unstable_period(FuncUnstId::HtDcPeriod) as usize;
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
    let mut smoother = PriceWma4::new(in_real, today);
    today += 3;
    for _ in 0..9 {
        let _ = smoother.next(in_real[today]);
        today += 1;
    }

    let mut core = HilbertCore::new();
    let mut smooth_period = 0.0;
    let mut out_idx = 0usize;

    while today <= end_idx {
        let smoothed_value = smoother.next(in_real[today]);
        let _ = core.step(today, smoothed_value);
        smooth_period = (0.33 * core.period) + (0.67 * smooth_period);
        if today >= adjusted_start {
            out_real[out_idx] = smooth_period;
            out_idx += 1;
        }
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn ht_phasor_lookback(context: &Context) -> i32 {
    32 + context.get_unstable_period(FuncUnstId::HtPhasor) as i32
}

pub(crate) fn ht_phasor_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_in_phase: &mut [f64],
    out_quadrature: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let lookback_total = 32 + context.get_unstable_period(FuncUnstId::HtPhasor) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_in_phase, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_quadrature, needed) {
        return ret_code;
    }

    *out_beg_idx = adjusted_start;
    let mut today = adjusted_start - lookback_total;
    let mut smoother = PriceWma4::new(in_real, today);
    today += 3;
    for _ in 0..9 {
        let _ = smoother.next(in_real[today]);
        today += 1;
    }

    let mut core = HilbertCore::new();
    let mut out_idx = 0usize;

    while today <= end_idx {
        let smoothed_value = smoother.next(in_real[today]);
        let step = core.step(today, smoothed_value);
        if today >= adjusted_start {
            out_quadrature[out_idx] = step.q1;
            out_in_phase[out_idx] = step.i1_prev3;
            out_idx += 1;
        }
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn ht_dcphase_lookback(context: &Context) -> i32 {
    63 + context.get_unstable_period(FuncUnstId::HtDcPhase) as i32
}

pub(crate) fn ht_dcphase_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
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

    let lookback_total = 63 + context.get_unstable_period(FuncUnstId::HtDcPhase) as usize;
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
    let mut smoother = PriceWma4::new(in_real, today);
    today += 3;
    for _ in 0..34 {
        let _ = smoother.next(in_real[today]);
        today += 1;
    }

    let mut core = HilbertCore::new();
    let mut smooth_period = 0.0;
    let mut smooth_price = [0.0; 50];
    let mut smooth_price_idx = 0usize;
    let temp_real = 1.0f64.atan();
    let rad2deg = 45.0 / temp_real;
    let const_deg2rad_by360 = temp_real * 8.0;
    let mut dc_phase = 0.0;
    let mut out_idx = 0usize;

    while today <= end_idx {
        let smoothed_value = smoother.next(in_real[today]);
        smooth_price[smooth_price_idx] = smoothed_value;
        let _ = core.step(today, smoothed_value);
        smooth_period = (0.33 * core.period) + (0.67 * smooth_period);

        let dc_period = smooth_period + 0.5;
        let dc_period_int = dc_period as usize;
        let mut real_part = 0.0;
        let mut imag_part = 0.0;
        let mut idx = smooth_price_idx;
        for i in 0..dc_period_int {
            let angle = (i as f64 * const_deg2rad_by360) / dc_period_int as f64;
            let price = smooth_price[idx];
            real_part += angle.sin() * price;
            imag_part += angle.cos() * price;
            idx = if idx == 0 { 49 } else { idx - 1 };
        }

        let abs_imag = imag_part.abs();
        if abs_imag > 0.0 {
            dc_phase = (real_part / imag_part).atan() * rad2deg;
        } else if abs_imag <= 0.01 {
            if real_part < 0.0 {
                dc_phase -= 90.0;
            } else if real_part > 0.0 {
                dc_phase += 90.0;
            }
        }
        dc_phase += 90.0;
        dc_phase += 360.0 / smooth_period;
        if imag_part < 0.0 {
            dc_phase += 180.0;
        }
        if dc_phase > 315.0 {
            dc_phase -= 360.0;
        }

        if today >= adjusted_start {
            out_real[out_idx] = dc_phase;
            out_idx += 1;
        }

        smooth_price_idx = (smooth_price_idx + 1) % 50;
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn ht_sine_lookback(context: &Context) -> i32 {
    63 + context.get_unstable_period(FuncUnstId::HtSine) as i32
}

pub(crate) fn ht_sine_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_sine: &mut [f64],
    out_lead_sine: &mut [f64],
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;
    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let lookback_total = 63 + context.get_unstable_period(FuncUnstId::HtSine) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_sine, needed) {
        return ret_code;
    }
    if let Err(ret_code) = validate_output_len(out_lead_sine, needed) {
        return ret_code;
    }

    *out_beg_idx = adjusted_start;
    let mut today = adjusted_start - lookback_total;
    let mut smoother = PriceWma4::new(in_real, today);
    today += 3;
    for _ in 0..34 {
        let _ = smoother.next(in_real[today]);
        today += 1;
    }

    let mut core = HilbertCore::new();
    let mut smooth_period = 0.0;
    let mut smooth_price = [0.0; 50];
    let mut smooth_price_idx = 0usize;
    let temp_real = 1.0f64.atan();
    let rad2deg = 45.0 / temp_real;
    let deg2rad = 1.0 / rad2deg;
    let const_deg2rad_by360 = temp_real * 8.0;
    let mut dc_phase = 0.0;
    let mut out_idx = 0usize;

    while today <= end_idx {
        let smoothed_value = smoother.next(in_real[today]);
        smooth_price[smooth_price_idx] = smoothed_value;
        let _ = core.step(today, smoothed_value);
        smooth_period = (0.33 * core.period) + (0.67 * smooth_period);

        let dc_period_int = (smooth_period + 0.5) as usize;
        let mut real_part = 0.0;
        let mut imag_part = 0.0;
        let mut idx = smooth_price_idx;
        for i in 0..dc_period_int {
            let angle = (i as f64 * const_deg2rad_by360) / dc_period_int as f64;
            let price = smooth_price[idx];
            real_part += angle.sin() * price;
            imag_part += angle.cos() * price;
            idx = if idx == 0 { 49 } else { idx - 1 };
        }

        let abs_imag = imag_part.abs();
        if abs_imag > 0.0 {
            dc_phase = (real_part / imag_part).atan() * rad2deg;
        } else if abs_imag <= 0.01 {
            if real_part < 0.0 {
                dc_phase -= 90.0;
            } else if real_part > 0.0 {
                dc_phase += 90.0;
            }
        }
        dc_phase += 90.0;
        dc_phase += 360.0 / smooth_period;
        if imag_part < 0.0 {
            dc_phase += 180.0;
        }
        if dc_phase > 315.0 {
            dc_phase -= 360.0;
        }

        if today >= adjusted_start {
            out_sine[out_idx] = (dc_phase * deg2rad).sin();
            out_lead_sine[out_idx] = ((dc_phase + 45.0) * deg2rad).sin();
            out_idx += 1;
        }

        smooth_price_idx = (smooth_price_idx + 1) % 50;
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn ht_trendline_lookback(context: &Context) -> i32 {
    63 + context.get_unstable_period(FuncUnstId::HtTrendline) as i32
}

pub(crate) fn ht_trendline_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
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

    let lookback_total = 63 + context.get_unstable_period(FuncUnstId::HtTrendline) as usize;
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
    let mut smoother = PriceWma4::new(in_real, today);
    today += 3;
    for _ in 0..34 {
        let _ = smoother.next(in_real[today]);
        today += 1;
    }

    let mut core = HilbertCore::new();
    let mut smooth_period = 0.0;
    let mut smooth_price = [0.0; 50];
    let mut smooth_price_idx = 0usize;
    let mut itrend1 = 0.0;
    let mut itrend2 = 0.0;
    let mut itrend3 = 0.0;
    let mut out_idx = 0usize;

    while today <= end_idx {
        let smoothed_value = smoother.next(in_real[today]);
        smooth_price[smooth_price_idx] = smoothed_value;
        let _ = core.step(today, smoothed_value);
        smooth_period = (0.33 * core.period) + (0.67 * smooth_period);

        let dc_period_int = (smooth_period + 0.5) as usize;
        let mut avg = 0.0;
        let mut idx = today;
        for _ in 0..dc_period_int {
            avg += in_real[idx];
            idx -= 1;
        }
        if dc_period_int > 0 {
            avg /= dc_period_int as f64;
        }

        let trendline = (4.0 * avg + 3.0 * itrend1 + 2.0 * itrend2 + itrend3) / 10.0;
        itrend3 = itrend2;
        itrend2 = itrend1;
        itrend1 = avg;

        if today >= adjusted_start {
            out_real[out_idx] = trendline;
            out_idx += 1;
        }

        smooth_price_idx = (smooth_price_idx + 1) % 50;
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

pub(crate) fn ht_trendmode_lookback(context: &Context) -> i32 {
    63 + context.get_unstable_period(FuncUnstId::HtTrendMode) as i32
}

pub(crate) fn ht_trendmode_run(
    context: &Context,
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
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

    let lookback_total = 63 + context.get_unstable_period(FuncUnstId::HtTrendMode) as usize;
    let adjusted_start = start_idx.max(lookback_total);
    if adjusted_start > end_idx {
        return RetCode::Success;
    }

    let needed = end_idx - adjusted_start + 1;
    if let Err(ret_code) = validate_output_len(out_integer, needed) {
        return ret_code;
    }

    *out_beg_idx = adjusted_start;
    let mut today = adjusted_start - lookback_total;
    let mut smoother = PriceWma4::new(in_real, today);
    today += 3;
    for _ in 0..34 {
        let _ = smoother.next(in_real[today]);
        today += 1;
    }

    let mut core = HilbertCore::new();
    let mut smooth_period = 0.0;
    let mut smooth_price = [0.0; 50];
    let mut smooth_price_idx = 0usize;
    let temp_real = 1.0f64.atan();
    let rad2deg = 45.0 / temp_real;
    let deg2rad = 1.0 / rad2deg;
    let const_deg2rad_by360 = temp_real * 8.0;
    let mut dc_phase = 0.0;
    let mut prev_dc_phase: f64;
    let mut prev_sine: f64;
    let mut prev_lead_sine: f64;
    let mut sine = 0.0;
    let mut lead_sine = 0.0;
    let mut days_in_trend = 0i32;
    let mut itrend1 = 0.0;
    let mut itrend2 = 0.0;
    let mut itrend3 = 0.0;
    let mut out_idx = 0usize;

    while today <= end_idx {
        let smoothed_value = smoother.next(in_real[today]);
        smooth_price[smooth_price_idx] = smoothed_value;
        let _ = core.step(today, smoothed_value);
        smooth_period = (0.33 * core.period) + (0.67 * smooth_period);

        prev_dc_phase = dc_phase;
        let dc_period = smooth_period + 0.5;
        let dc_period_int = dc_period as usize;
        let mut real_part = 0.0;
        let mut imag_part = 0.0;
        let mut idx = smooth_price_idx;
        for i in 0..dc_period_int {
            let angle = (i as f64 * const_deg2rad_by360) / dc_period_int as f64;
            let price = smooth_price[idx];
            real_part += angle.sin() * price;
            imag_part += angle.cos() * price;
            idx = if idx == 0 { 49 } else { idx - 1 };
        }

        let abs_imag = imag_part.abs();
        if abs_imag > 0.0 {
            dc_phase = (real_part / imag_part).atan() * rad2deg;
        } else if abs_imag <= 0.01 {
            if real_part < 0.0 {
                dc_phase -= 90.0;
            } else if real_part > 0.0 {
                dc_phase += 90.0;
            }
        }
        dc_phase += 90.0;
        dc_phase += 360.0 / smooth_period;
        if imag_part < 0.0 {
            dc_phase += 180.0;
        }
        if dc_phase > 315.0 {
            dc_phase -= 360.0;
        }

        prev_sine = sine;
        prev_lead_sine = lead_sine;
        sine = (dc_phase * deg2rad).sin();
        lead_sine = ((dc_phase + 45.0) * deg2rad).sin();

        let mut avg = 0.0;
        let mut raw_idx = today;
        for _ in 0..dc_period_int {
            avg += in_real[raw_idx];
            raw_idx -= 1;
        }
        if dc_period_int > 0 {
            avg /= dc_period_int as f64;
        }

        let trendline = (4.0 * avg + 3.0 * itrend1 + 2.0 * itrend2 + itrend3) / 10.0;
        itrend3 = itrend2;
        itrend2 = itrend1;
        itrend1 = avg;

        let mut trend = 1i32;
        if ((sine > lead_sine) && (prev_sine <= prev_lead_sine))
            || ((sine < lead_sine) && (prev_sine >= prev_lead_sine))
        {
            days_in_trend = 0;
            trend = 0;
        }

        days_in_trend += 1;
        if (days_in_trend as f64) < (0.5 * smooth_period) {
            trend = 0;
        }

        let temp = dc_phase - prev_dc_phase;
        if smooth_period != 0.0
            && (temp > (0.67 * 360.0 / smooth_period))
            && (temp < (1.5 * 360.0 / smooth_period))
        {
            trend = 0;
        }

        let price = smooth_price[smooth_price_idx];
        if trendline != 0.0 && ((price - trendline) / trendline).abs() >= 0.015 {
            trend = 1;
        }

        if today >= adjusted_start {
            out_integer[out_idx] = trend;
            out_idx += 1;
        }

        smooth_price_idx = (smooth_price_idx + 1) % 50;
        today += 1;
    }

    *out_nb_element = out_idx;
    RetCode::Success
}

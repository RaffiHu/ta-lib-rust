use crate::RetCode;
use crate::helpers::{validate_input_len, validate_output_len, validate_range};

pub(crate) fn avgprice(
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

    for (out_idx, i) in (start_idx..=end_idx).enumerate() {
        out_real[out_idx] = (in_open[i] + in_high[i] + in_low[i] + in_close[i]) / 4.0;
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn medprice(
    start_idx: usize,
    end_idx: usize,
    in_high: &[f64],
    in_low: &[f64],
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
    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, i) in (start_idx..=end_idx).enumerate() {
        out_real[out_idx] = (in_high[i] + in_low[i]) / 2.0;
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn typprice(
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
    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, i) in (start_idx..=end_idx).enumerate() {
        out_real[out_idx] = (in_high[i] + in_low[i] + in_close[i]) / 3.0;
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) fn wclprice(
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
    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, i) in (start_idx..=end_idx).enumerate() {
        out_real[out_idx] = (in_high[i] + in_low[i] + (in_close[i] * 2.0)) / 4.0;
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

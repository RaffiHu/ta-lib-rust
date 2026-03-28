use crate::RetCode;
use crate::helpers::{validate_input_len, validate_output_len, validate_range};

pub(crate) fn unary_op(
    start_idx: usize,
    end_idx: usize,
    in_real: &[f64],
    out_beg_idx: &mut usize,
    out_nb_element: &mut usize,
    out_real: &mut [f64],
    op: impl Fn(f64) -> f64,
) -> RetCode {
    *out_beg_idx = 0;
    *out_nb_element = 0;

    if let Err(ret_code) = validate_range(start_idx, end_idx) {
        return ret_code;
    }
    if let Err(ret_code) = validate_input_len(in_real, end_idx) {
        return ret_code;
    }

    let needed = end_idx - start_idx + 1;
    if let Err(ret_code) = validate_output_len(out_real, needed) {
        return ret_code;
    }

    for (out_idx, index) in (start_idx..=end_idx).enumerate() {
        out_real[out_idx] = op(in_real[index]);
    }

    *out_beg_idx = start_idx;
    *out_nb_element = needed;
    RetCode::Success
}

pub(crate) const EPSILON: f64 = 1.0e-14;

/// TA-Lib integer default sentinel.
pub const INTEGER_DEFAULT: i32 = i32::MIN;

/// TA-Lib real default sentinel.
pub const REAL_DEFAULT: f64 = -4.0e37;

use crate::RetCode;

pub(crate) fn validate_range(start_idx: usize, end_idx: usize) -> Result<(), RetCode> {
    if end_idx < start_idx {
        return Err(RetCode::OutOfRangeEndIndex);
    }

    Ok(())
}

pub(crate) fn validate_input_len<T>(input: &[T], end_idx: usize) -> Result<(), RetCode> {
    if end_idx >= input.len() {
        return Err(RetCode::BadParam);
    }

    Ok(())
}

pub(crate) fn validate_output_len<T>(output: &[T], needed: usize) -> Result<(), RetCode> {
    if output.len() < needed {
        return Err(RetCode::BadParam);
    }

    Ok(())
}

pub(crate) fn normalize_period(
    value: i32,
    default_value: i32,
    min_value: i32,
    max_value: i32,
) -> Result<usize, RetCode> {
    let normalized = if value == INTEGER_DEFAULT {
        default_value
    } else {
        value
    };

    if normalized < min_value || normalized > max_value {
        return Err(RetCode::BadParam);
    }

    usize::try_from(normalized).map_err(|_| RetCode::BadParam)
}

pub(crate) fn per_to_k(period: usize) -> f64 {
    2.0 / (period as f64 + 1.0)
}

pub(crate) fn is_zero(value: f64) -> bool {
    (-EPSILON..EPSILON).contains(&value)
}

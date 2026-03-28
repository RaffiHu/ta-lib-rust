use ta_lib::ta_func::{Core, RetCode};

#[test]
fn test_mult_double_precision() {
    // Test data: simple multiplication arrays
    let in_real0 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let in_real1 = [2.0, 3.0, 4.0, 5.0, 6.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    // Expected results: [2.0, 6.0, 12.0, 20.0, 30.0]
    let expected = [2.0, 6.0, 12.0, 20.0, 30.0];

    let result = Core::mult(
        0,                   // startIdx
        4,                   // endIdx
        &in_real0,           // inReal0
        &in_real1,           // inReal1
        &mut out_beg_idx,    // outBegIdx
        &mut out_nb_element, // outNBElement
        &mut out_real,       // outReal
    );

    assert_eq!(result, RetCode::Success);
    assert_eq!(out_beg_idx, 0);
    assert_eq!(out_nb_element, 5);

    for i in 0..5 {
        assert_eq!(out_real[i], expected[i], "Mismatch at index {}", i);
    }
}

#[test]
fn test_mult_single_precision() {
    // Test data: f32 inputs, f64 outputs
    let in_real0: [f32; 5] = [1.5, 2.5, 3.5, 4.5, 5.5];
    let in_real1: [f32; 5] = [2.0, 2.0, 2.0, 2.0, 2.0];
    let mut out_real = [0.0f64; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    // Expected results: [3.0, 5.0, 7.0, 9.0, 11.0]
    let expected = [3.0, 5.0, 7.0, 9.0, 11.0];

    let result = Core::mult_s(
        0,                   // startIdx
        4,                   // endIdx
        &in_real0,           // inReal0
        &in_real1,           // inReal1
        &mut out_beg_idx,    // outBegIdx
        &mut out_nb_element, // outNBElement
        &mut out_real,       // outReal
    );

    assert_eq!(result, RetCode::Success);
    assert_eq!(out_beg_idx, 0);
    assert_eq!(out_nb_element, 5);

    for i in 0..5 {
        assert_eq!(out_real[i], expected[i], "Mismatch at index {}", i);
    }
}

#[test]
fn test_mult_partial_range() {
    // Test processing only a subset of the data
    let in_real0 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let in_real1 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mut out_real = [0.0; 3];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    // Process indices 1-3: expected [4.0, 9.0, 16.0]
    let expected = [4.0, 9.0, 16.0];

    let result = Core::mult(
        1,                   // startIdx
        3,                   // endIdx
        &in_real0,           // inReal0
        &in_real1,           // inReal1
        &mut out_beg_idx,    // outBegIdx
        &mut out_nb_element, // outNBElement
        &mut out_real,       // outReal
    );

    assert_eq!(result, RetCode::Success);
    assert_eq!(out_beg_idx, 1);
    assert_eq!(out_nb_element, 3);

    for i in 0..3 {
        assert_eq!(out_real[i], expected[i], "Mismatch at index {}", i);
    }
}

#[test]
fn test_mult_error_conditions() {
    let in_real0 = [1.0, 2.0, 3.0];
    let in_real1 = [1.0, 2.0, 3.0];
    let mut out_real = [0.0; 3];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    // Test endIdx < startIdx (can't test negative values with usize)
    let result = Core::mult(
        2,
        1,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result, RetCode::OutOfRangeEndIndex);
}

#[test]
fn test_mult_lookback() {
    // MULT function should have zero lookback period
    assert_eq!(Core::mult_lookback(), 0);
}

#[test]
fn test_usize_negative_validation() {
    // Test what happens with usize parameters that would be "negative" in C
    let in_real0 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let in_real1 = [2.0, 3.0, 4.0, 5.0, 6.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    // This should test the validation logic
    // In C, passing -1 would be caught by startIdx < 0
    // In Rust with usize, what happens?

    // Test 1: Normal valid case
    let result = Core::mult(
        0,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    println!("Normal case result: {:?}", result);
    assert_eq!(result, RetCode::Success);

    // Test 2: What happens if we try to pass usize::MAX (equivalent to -1 in signed)?
    // This simulates what would happen if C code passed -1
    let result_max = Core::mult(
        usize::MAX,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    println!("usize::MAX startIdx result: {:?}", result_max);

    // Test 3: endIdx < startIdx case (already tested above, but let's be explicit)
    let result_backwards = Core::mult(
        4,
        0,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    println!("endIdx < startIdx result: {:?}", result_backwards);
    assert_eq!(result_backwards, RetCode::OutOfRangeEndIndex);
}

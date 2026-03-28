use ta_lib::generated::{SMOKE_CASES, SmokeRun};
use ta_lib::{Context, RetCode};

fn assert_smoke_output_consistent(case_name: &str, run: &SmokeRun) {
    assert_eq!(
        run.out_nb_element(),
        match run {
            SmokeRun::Real(values) => values.values.len(),
            SmokeRun::Integer(values) => values.values.len(),
            SmokeRun::RealPair(values) => {
                assert_eq!(
                    values.left.len(),
                    values.right.len(),
                    "{case_name} pair output length mismatch"
                );
                values.left.len()
            }
            SmokeRun::IntegerPair(values) => {
                assert_eq!(
                    values.left.len(),
                    values.right.len(),
                    "{case_name} pair output length mismatch"
                );
                values.left.len()
            }
            SmokeRun::RealTriple(values) => {
                assert_eq!(
                    values.first.len(),
                    values.second.len(),
                    "{case_name} triple output second length mismatch"
                );
                assert_eq!(
                    values.first.len(),
                    values.third.len(),
                    "{case_name} triple output third length mismatch"
                );
                values.first.len()
            }
        },
        "{case_name} output count mismatch",
    );
}

#[test]
fn generated_default_matrix_succeeds_for_every_function() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let lookback = (case.lookback_run)(&context);
        assert!(lookback >= 0, "{} lookback was negative", case.abbreviation);

        let run = (case.default_run)(&context);
        assert_eq!(
            run.ret_code(),
            RetCode::Success,
            "{} default smoke case failed",
            case.abbreviation
        );
        assert_smoke_output_consistent(case.abbreviation, &run);
        assert!(
            run.out_beg_idx() <= 15,
            "{} produced an outBegIdx outside the generated input window",
            case.abbreviation
        );
    }
}

#[test]
fn generated_boundary_matrix_succeeds_for_every_function() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let run = (case.boundary_run)(&context);
        assert_eq!(
            run.ret_code(),
            RetCode::Success,
            "{} boundary smoke case failed",
            case.abbreviation
        );
        assert_smoke_output_consistent(case.abbreviation, &run);
    }
}

#[test]
fn generated_seeded_matrix_succeeds_for_every_function() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let run = (case.seeded_run)(&context);
        assert_eq!(
            run.ret_code(),
            RetCode::Success,
            "{} seeded smoke case failed",
            case.abbreviation
        );
        assert_smoke_output_consistent(case.abbreviation, &run);
    }
}

#[test]
fn generated_variant_matrix_succeeds_for_every_function() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let run = (case.variant_run)(&context);
        assert_eq!(
            run.ret_code(),
            RetCode::Success,
            "{} variant smoke case failed",
            case.abbreviation
        );
        assert_smoke_output_consistent(case.abbreviation, &run);
    }
}

#[test]
fn generated_error_matrix_matches_shared_contracts() {
    let context = Context::new();

    for case in SMOKE_CASES {
        assert_eq!(
            (case.invalid_range_run)(&context),
            RetCode::OutOfRangeEndIndex,
            "{} invalid-range smoke case diverged",
            case.abbreviation
        );
        assert_eq!(
            (case.bad_param_run)(&context),
            RetCode::BadParam,
            "{} bad-param smoke case diverged",
            case.abbreviation
        );
    }
}

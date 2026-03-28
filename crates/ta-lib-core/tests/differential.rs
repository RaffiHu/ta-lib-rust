use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use ta_lib::generated::{SMOKE_CASES, SmokeRun};
use ta_lib::{Compatibility, Context, Core, FuncUnstId, RetCode, initialize, shutdown};

#[derive(Debug)]
struct OracleOutput {
    ret_code: i32,
    out_beg_idx: usize,
    out_nb_element: usize,
    values: Vec<f64>,
}

#[derive(Debug)]
struct OracleIntOutput {
    ret_code: i32,
    out_beg_idx: usize,
    out_nb_element: usize,
    values: Vec<i32>,
}

#[derive(Debug)]
struct OraclePairOutput {
    ret_code: i32,
    out_beg_idx: usize,
    out_nb_element: usize,
    left: Vec<f64>,
    right: Vec<f64>,
}

#[derive(Debug)]
struct OracleIntPairOutput {
    ret_code: i32,
    out_beg_idx: usize,
    out_nb_element: usize,
    left: Vec<i32>,
    right: Vec<i32>,
}

#[derive(Debug)]
struct OracleTripleOutput {
    ret_code: i32,
    out_beg_idx: usize,
    out_nb_element: usize,
    first: Vec<f64>,
    second: Vec<f64>,
    third: Vec<f64>,
}

static ORACLE_BIN: OnceLock<PathBuf> = OnceLock::new();

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn compatibility_runtime_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn find_file(root: &Path, needle: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path).ok()? {
            let entry = entry.ok()?;
            let entry_path = entry.path();
            if entry.file_type().ok()?.is_dir() {
                stack.push(entry_path);
            } else if entry.file_name() == needle {
                return Some(entry_path);
            }
        }
    }

    None
}

fn ensure_success(status: std::process::ExitStatus, step: &str) {
    assert!(status.success(), "{step} failed with status {status}");
}

fn find_oracle_library(root: &Path) -> Option<PathBuf> {
    find_file(root, "libta-lib.a").or_else(|| find_file(root, "libta-lib.so"))
}

fn oracle_bin() -> &'static PathBuf {
    ORACLE_BIN.get_or_init(|| {
        let workspace_root = workspace_root();
        let upstream_root = workspace_root.join("upstream-ta-lib-c");
        let build_dir = workspace_root.join("target/c-oracle/upstream-build");
        let oracle_bin = workspace_root.join("target/c-oracle/oracle_seed");

        if find_oracle_library(&build_dir).is_none() {
            fs::create_dir_all(&build_dir).expect("create c-oracle build dir");
            ensure_success(
                Command::new("cmake")
                    .arg("-S")
                    .arg(&upstream_root)
                    .arg("-B")
                    .arg(&build_dir)
                    .arg("-DBUILD_SHARED_LIBS=OFF")
                    .arg("-DBUILD_DEV_TOOLS=OFF")
                    .arg("-DCMAKE_BUILD_TYPE=Release")
                    .status()
                    .expect("run cmake configure"),
                "cmake configure",
            );
            ensure_success(
                Command::new("cmake")
                    .arg("--build")
                    .arg(&build_dir)
                    .arg("--target")
                    .arg("ta-lib")
                    .status()
                    .expect("run cmake build"),
                "cmake build",
            );
        }

        let library = find_oracle_library(&build_dir).expect("find built ta-lib library");
        let library_dir = library.parent().expect("library parent");
        let oracle_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/c_oracle_seed.c");
        let include_dir = upstream_root.join("include");
        let output_dir = workspace_root.join("target/c-oracle");
        fs::create_dir_all(&output_dir).expect("create c-oracle output dir");

        let mut compile = Command::new("cc");
        compile
            .arg("-std=c99")
            .arg("-I")
            .arg(include_dir)
            .arg(&oracle_src)
            .arg(&library)
            .arg("-lm")
            .arg("-o")
            .arg(&oracle_bin);

        if library.extension().and_then(|extension| extension.to_str()) == Some("so") {
            compile.arg(format!("-Wl,-rpath,{}", library_dir.display()));
        }

        ensure_success(
            compile.status().expect("compile oracle helper"),
            "compile oracle helper",
        );

        oracle_bin
    })
}

fn run_oracle(case_name: &str) -> OracleOutput {
    let output = Command::new(oracle_bin())
        .arg(case_name)
        .output()
        .expect("run c oracle helper");
    assert!(
        output.status.success(),
        "oracle helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output is utf8");
    let mut parts = stdout.split_whitespace();
    let ret_code = parts
        .next()
        .expect("retcode")
        .parse::<i32>()
        .expect("retcode integer");
    let out_beg_idx = parts
        .next()
        .expect("outBegIdx")
        .parse::<usize>()
        .expect("outBegIdx usize");
    let out_nb_element = parts
        .next()
        .expect("outNBElement")
        .parse::<usize>()
        .expect("outNBElement usize");
    let values = parts
        .map(|value| value.parse::<f64>().expect("oracle value"))
        .collect();

    OracleOutput {
        ret_code,
        out_beg_idx,
        out_nb_element,
        values,
    }
}

fn run_oracle_int(case_name: &str) -> OracleIntOutput {
    let output = Command::new(oracle_bin())
        .arg(case_name)
        .output()
        .expect("run c oracle helper");
    assert!(
        output.status.success(),
        "oracle helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output is utf8");
    let mut parts = stdout.split_whitespace();
    let ret_code = parts
        .next()
        .expect("retcode")
        .parse::<i32>()
        .expect("retcode integer");
    let out_beg_idx = parts
        .next()
        .expect("outBegIdx")
        .parse::<usize>()
        .expect("outBegIdx usize");
    let out_nb_element = parts
        .next()
        .expect("outNBElement")
        .parse::<usize>()
        .expect("outNBElement usize");
    let values = parts
        .map(|value| value.parse::<i32>().expect("oracle integer value"))
        .collect();

    OracleIntOutput {
        ret_code,
        out_beg_idx,
        out_nb_element,
        values,
    }
}

fn run_oracle_pair(case_name: &str) -> OraclePairOutput {
    let output = Command::new(oracle_bin())
        .arg(case_name)
        .output()
        .expect("run c oracle helper");
    assert!(
        output.status.success(),
        "oracle helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output is utf8");
    let (header, rest) = stdout.split_once('|').expect("pair oracle separator");
    let mut header_parts = header.split_whitespace();
    let ret_code = header_parts
        .next()
        .expect("retcode")
        .parse::<i32>()
        .expect("retcode integer");
    let out_beg_idx = header_parts
        .next()
        .expect("outBegIdx")
        .parse::<usize>()
        .expect("outBegIdx usize");
    let out_nb_element = header_parts
        .next()
        .expect("outNBElement")
        .parse::<usize>()
        .expect("outNBElement usize");
    let left = header_parts
        .map(|value| value.parse::<f64>().expect("left pair value"))
        .collect();
    let right = rest
        .split_whitespace()
        .map(|value| value.parse::<f64>().expect("right pair value"))
        .collect();

    OraclePairOutput {
        ret_code,
        out_beg_idx,
        out_nb_element,
        left,
        right,
    }
}

fn run_oracle_int_pair(case_name: &str) -> OracleIntPairOutput {
    let output = Command::new(oracle_bin())
        .arg(case_name)
        .output()
        .expect("run c oracle helper");
    assert!(
        output.status.success(),
        "oracle helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output is utf8");
    let (header, rest) = stdout.split_once('|').expect("pair oracle separator");
    let mut header_parts = header.split_whitespace();
    let ret_code = header_parts
        .next()
        .expect("retcode")
        .parse::<i32>()
        .expect("retcode integer");
    let out_beg_idx = header_parts
        .next()
        .expect("outBegIdx")
        .parse::<usize>()
        .expect("outBegIdx usize");
    let out_nb_element = header_parts
        .next()
        .expect("outNBElement")
        .parse::<usize>()
        .expect("outNBElement usize");
    let left = header_parts
        .map(|value| value.parse::<i32>().expect("left pair int"))
        .collect();
    let right = rest
        .split_whitespace()
        .map(|value| value.parse::<i32>().expect("right pair int"))
        .collect();

    OracleIntPairOutput {
        ret_code,
        out_beg_idx,
        out_nb_element,
        left,
        right,
    }
}

fn run_oracle_triple(case_name: &str) -> OracleTripleOutput {
    let output = Command::new(oracle_bin())
        .arg(case_name)
        .output()
        .expect("run c oracle helper");
    assert!(
        output.status.success(),
        "oracle helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output is utf8");
    let mut parts = stdout.split('|');
    let first_part = parts.next().expect("first triple segment");
    let second_part = parts.next().expect("second triple segment");
    let third_part = parts.next().expect("third triple segment");

    let mut header_parts = first_part.split_whitespace();
    let ret_code = header_parts
        .next()
        .expect("retcode")
        .parse::<i32>()
        .expect("retcode integer");
    let out_beg_idx = header_parts
        .next()
        .expect("outBegIdx")
        .parse::<usize>()
        .expect("outBegIdx usize");
    let out_nb_element = header_parts
        .next()
        .expect("outNBElement")
        .parse::<usize>()
        .expect("outNBElement usize");
    let first = header_parts
        .map(|value| value.parse::<f64>().expect("first triple value"))
        .collect();
    let second = second_part
        .split_whitespace()
        .map(|value| value.parse::<f64>().expect("second triple value"))
        .collect();
    let third = third_part
        .split_whitespace()
        .map(|value| value.parse::<f64>().expect("third triple value"))
        .collect();

    OracleTripleOutput {
        ret_code,
        out_beg_idx,
        out_nb_element,
        first,
        second,
        third,
    }
}

fn assert_close(actual: &[f64], expected: &[f64]) {
    assert_close_named(actual, expected, "oracle");
}

fn assert_close_named(actual: &[f64], expected: &[f64], label: &str) {
    assert_eq!(actual.len(), expected.len(), "value length mismatch");
    for (index, (lhs, rhs)) in actual.iter().zip(expected).enumerate() {
        if lhs.is_nan() && rhs.is_nan() {
            continue;
        }
        let delta = (lhs - rhs).abs();
        assert!(
            delta <= 1.0e-10,
            "{label} value mismatch at index {index}: left={lhs}, right={rhs}, delta={delta}"
        );
    }
}

fn assert_generated_smoke_matches_oracle(rust_run: &SmokeRun, case_name: &str) {
    match rust_run {
        SmokeRun::Real(run) => {
            let oracle = run_oracle(case_name);
            assert_eq!(
                run.ret_code as i32, oracle.ret_code,
                "{case_name} retcode mismatch"
            );
            assert_eq!(
                run.out_beg_idx, oracle.out_beg_idx,
                "{case_name} outBegIdx mismatch"
            );
            assert_eq!(
                run.out_nb_element, oracle.out_nb_element,
                "{case_name} outNbElement mismatch"
            );
            assert_close_named(&run.values, &oracle.values, case_name);
        }
        SmokeRun::Integer(run) => {
            let oracle = run_oracle_int(case_name);
            assert_eq!(
                run.ret_code as i32, oracle.ret_code,
                "{case_name} retcode mismatch"
            );
            assert_eq!(
                run.out_beg_idx, oracle.out_beg_idx,
                "{case_name} outBegIdx mismatch"
            );
            assert_eq!(
                run.out_nb_element, oracle.out_nb_element,
                "{case_name} outNbElement mismatch"
            );
            assert_eq!(
                run.values, oracle.values,
                "{case_name} integer values mismatch"
            );
        }
        SmokeRun::RealPair(run) => {
            let oracle = run_oracle_pair(case_name);
            assert_eq!(
                run.ret_code as i32, oracle.ret_code,
                "{case_name} retcode mismatch"
            );
            assert_eq!(
                run.out_beg_idx, oracle.out_beg_idx,
                "{case_name} outBegIdx mismatch"
            );
            assert_eq!(
                run.out_nb_element, oracle.out_nb_element,
                "{case_name} outNbElement mismatch"
            );
            assert_close_named(&run.left, &oracle.left, case_name);
            assert_close_named(&run.right, &oracle.right, case_name);
        }
        SmokeRun::IntegerPair(run) => {
            let oracle = run_oracle_int_pair(case_name);
            assert_eq!(
                run.ret_code as i32, oracle.ret_code,
                "{case_name} retcode mismatch"
            );
            assert_eq!(
                run.out_beg_idx, oracle.out_beg_idx,
                "{case_name} outBegIdx mismatch"
            );
            assert_eq!(
                run.out_nb_element, oracle.out_nb_element,
                "{case_name} outNbElement mismatch"
            );
            assert_eq!(
                run.left, oracle.left,
                "{case_name} left integer values mismatch"
            );
            assert_eq!(
                run.right, oracle.right,
                "{case_name} right integer values mismatch"
            );
        }
        SmokeRun::RealTriple(run) => {
            let oracle = run_oracle_triple(case_name);
            assert_eq!(
                run.ret_code as i32, oracle.ret_code,
                "{case_name} retcode mismatch"
            );
            assert_eq!(
                run.out_beg_idx, oracle.out_beg_idx,
                "{case_name} outBegIdx mismatch"
            );
            assert_eq!(
                run.out_nb_element, oracle.out_nb_element,
                "{case_name} outNbElement mismatch"
            );
            assert_close_named(&run.first, &oracle.first, case_name);
            assert_close_named(&run.second, &oracle.second, case_name);
            assert_close_named(&run.third, &oracle.third, case_name);
        }
    }
}

#[test]
fn core_requires_initialize() {
    let _guard = compatibility_runtime_test_lock()
        .lock()
        .expect("compatibility runtime test mutex poisoned");
    let _ = shutdown();

    let in_real0 = [1.0, 2.0, 3.0];
    let in_real1 = [1.0, 2.0, 3.0];
    let mut out_real = [0.0; 3];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = Core::add(
        0,
        2,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result, RetCode::LibNotInitialize);
}

#[test]
fn add_matches_c_oracle() {
    let oracle = run_oracle("add_basic");
    let context = Context::new();
    let in_real0 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let in_real1 = [5.0, 4.0, 3.0, 2.0, 1.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.add(
        0,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
}

#[test]
fn unary_math_transforms_match_c_oracle() {
    let acos_oracle = run_oracle("acos_basic");
    let asin_oracle = run_oracle("asin_basic");
    let atan_oracle = run_oracle("atan_basic");
    let ceil_oracle = run_oracle("ceil_basic");
    let cos_oracle = run_oracle("cos_basic");
    let cosh_oracle = run_oracle("cosh_basic");
    let exp_oracle = run_oracle("exp_basic");
    let floor_oracle = run_oracle("floor_basic");
    let ln_oracle = run_oracle("ln_basic");
    let sqrt_oracle = run_oracle("sqrt_basic");
    let log10_oracle = run_oracle("log10_basic");
    let sin_oracle = run_oracle("sin_basic");
    let sinh_oracle = run_oracle("sinh_basic");
    let tan_oracle = run_oracle("tan_basic");
    let tanh_oracle = run_oracle("tanh_basic");
    let context = Context::new();

    let acos_input = [-1.0, -0.5, 0.0, 0.5, 1.0];
    let atan_input = [-2.0, -1.0, 0.0, 1.0, 2.0];
    let rounding_input = [-1.7, -0.2, 0.0, 1.2, 2.8];
    let trig_input = [-1.0, -0.5, 0.0, 0.5, 1.0];
    let exp_input = [-1.0, 0.0, 1.0, 2.0, 3.0];
    let ln_input = [1.0, 2.0, 4.0, 8.0, 16.0];
    let sqrt_input = [1.0, 4.0, 9.0, 16.0, 25.0];
    let log10_input = [1.0, 10.0, 100.0, 1000.0, 10000.0];

    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.acos(
        0,
        4,
        &acos_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, acos_oracle.ret_code);
    assert_eq!(out_beg_idx, acos_oracle.out_beg_idx);
    assert_eq!(out_nb_element, acos_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &acos_oracle.values);

    let result = context.asin(
        0,
        4,
        &acos_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, asin_oracle.ret_code);
    assert_eq!(out_beg_idx, asin_oracle.out_beg_idx);
    assert_eq!(out_nb_element, asin_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &asin_oracle.values);

    let result = context.atan(
        0,
        4,
        &atan_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, atan_oracle.ret_code);
    assert_eq!(out_beg_idx, atan_oracle.out_beg_idx);
    assert_eq!(out_nb_element, atan_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &atan_oracle.values);

    let result = context.ceil(
        0,
        4,
        &rounding_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ceil_oracle.ret_code);
    assert_eq!(out_beg_idx, ceil_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ceil_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ceil_oracle.values);

    let result = context.cos(
        0,
        4,
        &trig_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, cos_oracle.ret_code);
    assert_eq!(out_beg_idx, cos_oracle.out_beg_idx);
    assert_eq!(out_nb_element, cos_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &cos_oracle.values);

    let result = context.cosh(
        0,
        4,
        &trig_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, cosh_oracle.ret_code);
    assert_eq!(out_beg_idx, cosh_oracle.out_beg_idx);
    assert_eq!(out_nb_element, cosh_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &cosh_oracle.values);

    let result = context.exp(
        0,
        4,
        &exp_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, exp_oracle.ret_code);
    assert_eq!(out_beg_idx, exp_oracle.out_beg_idx);
    assert_eq!(out_nb_element, exp_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &exp_oracle.values);

    let result = context.floor(
        0,
        4,
        &rounding_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, floor_oracle.ret_code);
    assert_eq!(out_beg_idx, floor_oracle.out_beg_idx);
    assert_eq!(out_nb_element, floor_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &floor_oracle.values);

    let result = context.ln(
        0,
        4,
        &ln_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ln_oracle.ret_code);
    assert_eq!(out_beg_idx, ln_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ln_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ln_oracle.values);

    let result = context.sqrt(
        0,
        4,
        &sqrt_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, sqrt_oracle.ret_code);
    assert_eq!(out_beg_idx, sqrt_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sqrt_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sqrt_oracle.values);

    let result = context.log10(
        0,
        4,
        &log10_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, log10_oracle.ret_code);
    assert_eq!(out_beg_idx, log10_oracle.out_beg_idx);
    assert_eq!(out_nb_element, log10_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &log10_oracle.values);

    let result = context.sin(
        0,
        4,
        &trig_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, sin_oracle.ret_code);
    assert_eq!(out_beg_idx, sin_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sin_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sin_oracle.values);

    let result = context.sinh(
        0,
        4,
        &trig_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, sinh_oracle.ret_code);
    assert_eq!(out_beg_idx, sinh_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sinh_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sinh_oracle.values);

    let result = context.tan(
        0,
        4,
        &trig_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, tan_oracle.ret_code);
    assert_eq!(out_beg_idx, tan_oracle.out_beg_idx);
    assert_eq!(out_nb_element, tan_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &tan_oracle.values);

    let result = context.tanh(
        0,
        4,
        &trig_input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, tanh_oracle.ret_code);
    assert_eq!(out_beg_idx, tanh_oracle.out_beg_idx);
    assert_eq!(out_nb_element, tanh_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &tanh_oracle.values);
}

#[test]
fn rolling_window_functions_match_c_oracle() {
    let sum_oracle = run_oracle("sum_period_3");
    let max_oracle = run_oracle("max_period_3");
    let min_oracle = run_oracle("min_period_3");
    let maxindex_oracle = run_oracle_int("maxindex_period_3");
    let minindex_oracle = run_oracle_int("minindex_period_3");
    let minmax_oracle = run_oracle_pair("minmax_period_3");
    let minmaxindex_oracle = run_oracle_int_pair("minmaxindex_period_3");
    let context = Context::new();

    let sum_input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let max_input = [1.0, 7.0, 3.0, 4.0, 6.0, 2.0];
    let min_input = [5.0, 2.0, 3.0, 1.0, 6.0, 4.0];

    let mut out_real = [0.0; 8];
    let mut out_integer = [0i32; 8];
    let mut out_real_b = [0.0; 8];
    let mut out_integer_b = [0i32; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.sum(
        0,
        5,
        &sum_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, sum_oracle.ret_code);
    assert_eq!(out_beg_idx, sum_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sum_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sum_oracle.values);

    let result = context.max(
        0,
        5,
        &max_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, max_oracle.ret_code);
    assert_eq!(out_beg_idx, max_oracle.out_beg_idx);
    assert_eq!(out_nb_element, max_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &max_oracle.values);

    let result = context.min(
        0,
        5,
        &min_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, min_oracle.ret_code);
    assert_eq!(out_beg_idx, min_oracle.out_beg_idx);
    assert_eq!(out_nb_element, min_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &min_oracle.values);

    let result = context.max_index(
        0,
        5,
        &max_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, maxindex_oracle.ret_code);
    assert_eq!(out_beg_idx, maxindex_oracle.out_beg_idx);
    assert_eq!(out_nb_element, maxindex_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        maxindex_oracle.values.as_slice()
    );

    let result = context.min_index(
        0,
        5,
        &min_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, minindex_oracle.ret_code);
    assert_eq!(out_beg_idx, minindex_oracle.out_beg_idx);
    assert_eq!(out_nb_element, minindex_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        minindex_oracle.values.as_slice()
    );

    let minmax_input = [5.0, 2.0, 7.0, 1.0, 6.0, 4.0];
    let result = context.min_max(
        0,
        5,
        &minmax_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, minmax_oracle.ret_code);
    assert_eq!(out_beg_idx, minmax_oracle.out_beg_idx);
    assert_eq!(out_nb_element, minmax_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &minmax_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &minmax_oracle.right);

    let result = context.min_max_index(
        0,
        5,
        &minmax_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
        &mut out_integer_b,
    );
    assert_eq!(result as i32, minmaxindex_oracle.ret_code);
    assert_eq!(out_beg_idx, minmaxindex_oracle.out_beg_idx);
    assert_eq!(out_nb_element, minmaxindex_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        minmaxindex_oracle.left.as_slice()
    );
    assert_eq!(
        &out_integer_b[..out_nb_element],
        minmaxindex_oracle.right.as_slice()
    );
}

#[test]
fn price_transforms_match_c_oracle() {
    let avgprice_oracle = run_oracle("avgprice_basic");
    let medprice_oracle = run_oracle("medprice_basic");
    let typprice_oracle = run_oracle("typprice_basic");
    let wclprice_oracle = run_oracle("wclprice_basic");
    let context = Context::new();

    let in_open = [1.0, 2.0, 3.0, 4.0];
    let in_high = [2.0, 3.0, 4.0, 5.0];
    let in_low = [0.5, 1.5, 2.5, 3.5];
    let in_close = [1.5, 2.5, 3.5, 4.5];

    let mut out_real = [0.0; 4];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.avg_price(
        0,
        3,
        &in_open,
        &in_high,
        &in_low,
        &in_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, avgprice_oracle.ret_code);
    assert_eq!(out_beg_idx, avgprice_oracle.out_beg_idx);
    assert_eq!(out_nb_element, avgprice_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &avgprice_oracle.values);

    let result = context.med_price(
        0,
        3,
        &in_high,
        &in_low,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, medprice_oracle.ret_code);
    assert_eq!(out_beg_idx, medprice_oracle.out_beg_idx);
    assert_eq!(out_nb_element, medprice_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &medprice_oracle.values);

    let result = context.typ_price(
        0,
        3,
        &in_high,
        &in_low,
        &in_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, typprice_oracle.ret_code);
    assert_eq!(out_beg_idx, typprice_oracle.out_beg_idx);
    assert_eq!(out_nb_element, typprice_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &typprice_oracle.values);

    let result = context.wcl_price(
        0,
        3,
        &in_high,
        &in_low,
        &in_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, wclprice_oracle.ret_code);
    assert_eq!(out_beg_idx, wclprice_oracle.out_beg_idx);
    assert_eq!(out_nb_element, wclprice_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &wclprice_oracle.values);
}

#[test]
fn range_and_stat_functions_match_c_oracle() {
    let midpoint_oracle = run_oracle("midpoint_period_3");
    let midprice_oracle = run_oracle("midprice_period_3");
    let trange_oracle = run_oracle("trange_basic");
    let avgdev_oracle = run_oracle("avgdev_period_3");
    let context = Context::new();

    let midpoint_input = [5.0, 2.0, 7.0, 1.0, 6.0, 4.0];
    let midprice_high = [5.0, 7.0, 6.0, 8.0, 4.0, 9.0];
    let midprice_low = [1.0, 2.0, 3.0, 2.5, 1.5, 4.0];
    let trange_high = [10.0, 11.0, 15.0, 13.0, 14.0];
    let trange_low = [8.0, 9.0, 10.0, 11.0, 12.0];
    let trange_close = [9.0, 10.0, 12.0, 12.5, 13.0];
    let avgdev_input = [1.0, 2.0, 3.0, 5.0, 8.0, 13.0];

    let mut out_real = [0.0; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.mid_point(
        0,
        5,
        &midpoint_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, midpoint_oracle.ret_code);
    assert_eq!(out_beg_idx, midpoint_oracle.out_beg_idx);
    assert_eq!(out_nb_element, midpoint_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &midpoint_oracle.values);

    let result = context.mid_price(
        0,
        5,
        &midprice_high,
        &midprice_low,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, midprice_oracle.ret_code);
    assert_eq!(out_beg_idx, midprice_oracle.out_beg_idx);
    assert_eq!(out_nb_element, midprice_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &midprice_oracle.values);

    let result = context.true_range(
        0,
        4,
        &trange_high,
        &trange_low,
        &trange_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, trange_oracle.ret_code);
    assert_eq!(out_beg_idx, trange_oracle.out_beg_idx);
    assert_eq!(out_nb_element, trange_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &trange_oracle.values);

    let result = context.avg_dev(
        0,
        5,
        &avgdev_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, avgdev_oracle.ret_code);
    assert_eq!(out_beg_idx, avgdev_oracle.out_beg_idx);
    assert_eq!(out_nb_element, avgdev_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &avgdev_oracle.values);
}

#[test]
fn momentum_and_volatility_functions_match_c_oracle() {
    let bop_oracle = run_oracle("bop_basic");
    let mom_oracle = run_oracle("mom_period_3");
    let roc_oracle = run_oracle("roc_period_3");
    let rocp_oracle = run_oracle("rocp_period_3");
    let rocr_oracle = run_oracle("rocr_period_3");
    let rocr100_oracle = run_oracle("rocr100_period_3");
    let atr_oracle = run_oracle("atr_period_3");
    let atr_unstable_oracle = run_oracle("atr_unstable_2");
    let natr_oracle = run_oracle("natr_period_3");
    let natr_unstable_oracle = run_oracle("natr_unstable_2");
    let mut context = Context::new();

    let bop_open = [10.0, 11.0, 10.0, 14.0, 13.0];
    let bop_high = [12.0, 13.0, 11.0, 15.0, 14.0];
    let bop_low = [9.0, 10.0, 9.5, 13.0, 12.0];
    let bop_close = [11.0, 12.0, 10.5, 13.5, 13.0];
    let roc_input = [10.0, 11.0, 12.0, 15.0, 18.0, 21.0];
    let atr_high = [10.0, 11.0, 15.0, 13.0, 14.0, 16.0];
    let atr_low = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0];
    let atr_close = [9.0, 10.0, 12.0, 12.5, 13.0, 15.0];
    let atr_high_unstable = [10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0];
    let atr_low_unstable = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0];
    let atr_close_unstable = [9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5];

    let mut out_real = [0.0; 16];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.bop(
        0,
        4,
        &bop_open,
        &bop_high,
        &bop_low,
        &bop_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, bop_oracle.ret_code);
    assert_eq!(out_beg_idx, bop_oracle.out_beg_idx);
    assert_eq!(out_nb_element, bop_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &bop_oracle.values);

    let result = context.mom(
        0,
        5,
        &roc_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, mom_oracle.ret_code);
    assert_eq!(out_beg_idx, mom_oracle.out_beg_idx);
    assert_eq!(out_nb_element, mom_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &mom_oracle.values);

    let result = context.roc(
        0,
        5,
        &roc_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, roc_oracle.ret_code);
    assert_eq!(out_beg_idx, roc_oracle.out_beg_idx);
    assert_eq!(out_nb_element, roc_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &roc_oracle.values);

    let result = context.roc_p(
        0,
        5,
        &roc_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, rocp_oracle.ret_code);
    assert_eq!(out_beg_idx, rocp_oracle.out_beg_idx);
    assert_eq!(out_nb_element, rocp_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &rocp_oracle.values);

    let result = context.roc_r(
        0,
        5,
        &roc_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, rocr_oracle.ret_code);
    assert_eq!(out_beg_idx, rocr_oracle.out_beg_idx);
    assert_eq!(out_nb_element, rocr_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &rocr_oracle.values);

    let result = context.roc_r100(
        0,
        5,
        &roc_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, rocr100_oracle.ret_code);
    assert_eq!(out_beg_idx, rocr100_oracle.out_beg_idx);
    assert_eq!(out_nb_element, rocr100_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &rocr100_oracle.values);

    let result = context.atr(
        0,
        5,
        &atr_high,
        &atr_low,
        &atr_close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, atr_oracle.ret_code);
    assert_eq!(out_beg_idx, atr_oracle.out_beg_idx);
    assert_eq!(out_nb_element, atr_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &atr_oracle.values);

    context.set_unstable_period(FuncUnstId::Atr, 2);
    let result = context.atr(
        0,
        7,
        &atr_high_unstable,
        &atr_low_unstable,
        &atr_close_unstable,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, atr_unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, atr_unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, atr_unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &atr_unstable_oracle.values);

    context.set_unstable_period(FuncUnstId::Atr, 0);
    let result = context.natr(
        0,
        5,
        &atr_high,
        &atr_low,
        &atr_close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, natr_oracle.ret_code);
    assert_eq!(out_beg_idx, natr_oracle.out_beg_idx);
    assert_eq!(out_nb_element, natr_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &natr_oracle.values);

    context.set_unstable_period(FuncUnstId::Natr, 2);
    let result = context.natr(
        0,
        7,
        &atr_high_unstable,
        &atr_low_unstable,
        &atr_close_unstable,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, natr_unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, natr_unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, natr_unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &natr_unstable_oracle.values);
}

#[test]
fn imi_matches_c_oracle_default_and_unstable() {
    let oracle = run_oracle("imi_period_3");
    let unstable_oracle = run_oracle("imi_period_3_unstable_2");
    let mut context = Context::new();

    let in_open = [10.0, 11.0, 10.5, 12.0, 13.0, 12.5];
    let in_close = [11.0, 10.5, 11.5, 13.0, 12.0, 13.5];
    let unstable_open = [10.0, 11.0, 10.5, 12.0, 13.0, 12.5, 13.5, 14.0];
    let unstable_close = [11.0, 10.5, 11.5, 13.0, 12.0, 13.5, 14.5, 13.0];

    let mut out_real = [0.0; 16];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.imi(
        0,
        5,
        &in_open,
        &in_close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);

    context.set_unstable_period(FuncUnstId::Imi, 2);
    let result = context.imi(
        0,
        7,
        &unstable_open,
        &unstable_close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &unstable_oracle.values);
}

#[test]
fn volume_and_willr_functions_match_c_oracle() {
    let ad_oracle = run_oracle("ad_basic");
    let obv_oracle = run_oracle("obv_basic");
    let willr_oracle = run_oracle("willr_period_3");
    let context = Context::new();

    let ad_high = [12.0, 13.0, 11.0, 15.0, 14.0];
    let ad_low = [9.0, 10.0, 9.5, 13.0, 12.0];
    let ad_close = [11.0, 12.0, 10.5, 13.5, 13.0];
    let volume = [100.0, 110.0, 120.0, 130.0, 140.0];
    let obv_real = [10.0, 11.0, 10.5, 12.0, 11.5];
    let willr_high = [10.0, 11.0, 15.0, 13.0, 14.0, 16.0];
    let willr_low = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0];
    let willr_close = [9.0, 10.0, 12.0, 12.5, 13.0, 15.0];

    let mut out_real = [0.0; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.ad(
        0,
        4,
        &ad_high,
        &ad_low,
        &ad_close,
        &volume,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ad_oracle.ret_code);
    assert_eq!(out_beg_idx, ad_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ad_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ad_oracle.values);

    let result = context.obv(
        0,
        4,
        &obv_real,
        &volume,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, obv_oracle.ret_code);
    assert_eq!(out_beg_idx, obv_oracle.out_beg_idx);
    assert_eq!(out_nb_element, obv_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &obv_oracle.values);

    let result = context.will_r(
        0,
        5,
        &willr_high,
        &willr_low,
        &willr_close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, willr_oracle.ret_code);
    assert_eq!(out_beg_idx, willr_oracle.out_beg_idx);
    assert_eq!(out_nb_element, willr_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &willr_oracle.values);
}

#[test]
fn rolling_statistics_match_c_oracle() {
    let var_oracle = run_oracle("var_period_3");
    let stddev_oracle = run_oracle("stddev_period_3");
    let correl_oracle = run_oracle("correl_period_3");
    let beta_oracle = run_oracle("beta_period_3");
    let context = Context::new();

    let var_input = [1.0, 2.0, 3.0, 5.0, 8.0, 13.0];
    let corr_lhs = [1.0, 2.0, 3.0, 5.0, 8.0, 13.0];
    let corr_rhs = [2.0, 4.0, 6.0, 10.0, 16.0, 26.0];
    let beta_lhs = [10.0, 11.0, 12.0, 15.0, 18.0, 21.0];
    let beta_rhs = [20.0, 21.0, 22.0, 24.0, 27.0, 30.0];

    let mut out_real = [0.0; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.variance(
        0,
        5,
        &var_input,
        3,
        1.0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, var_oracle.ret_code);
    assert_eq!(out_beg_idx, var_oracle.out_beg_idx);
    assert_eq!(out_nb_element, var_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &var_oracle.values);

    let result = context.std_dev(
        0,
        5,
        &var_input,
        3,
        1.0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, stddev_oracle.ret_code);
    assert_eq!(out_beg_idx, stddev_oracle.out_beg_idx);
    assert_eq!(out_nb_element, stddev_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &stddev_oracle.values);

    let result = context.correl(
        0,
        5,
        &corr_lhs,
        &corr_rhs,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, correl_oracle.ret_code);
    assert_eq!(out_beg_idx, correl_oracle.out_beg_idx);
    assert_eq!(out_nb_element, correl_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &correl_oracle.values);

    let result = context.beta(
        0,
        5,
        &beta_lhs,
        &beta_rhs,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, beta_oracle.ret_code);
    assert_eq!(out_beg_idx, beta_oracle.out_beg_idx);
    assert_eq!(out_nb_element, beta_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &beta_oracle.values);
}

#[test]
fn linear_regression_family_matches_c_oracle() {
    let linearreg_oracle = run_oracle("linearreg_period_5");
    let angle_oracle = run_oracle("linearreg_angle_period_5");
    let intercept_oracle = run_oracle("linearreg_intercept_period_5");
    let slope_oracle = run_oracle("linearreg_slope_period_5");
    let tsf_oracle = run_oracle("tsf_period_5");
    let context = Context::new();
    let input = [2.0, 4.0, 6.0, 8.0, 10.0, 9.0, 8.0, 7.0, 6.0, 5.0];

    let mut out_real = [0.0; 16];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.linearreg(
        0,
        9,
        &input,
        5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, linearreg_oracle.ret_code);
    assert_eq!(out_beg_idx, linearreg_oracle.out_beg_idx);
    assert_eq!(out_nb_element, linearreg_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &linearreg_oracle.values);

    let result = context.linearreg_angle(
        0,
        9,
        &input,
        5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, angle_oracle.ret_code);
    assert_eq!(out_beg_idx, angle_oracle.out_beg_idx);
    assert_eq!(out_nb_element, angle_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &angle_oracle.values);

    let result = context.linearreg_intercept(
        0,
        9,
        &input,
        5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, intercept_oracle.ret_code);
    assert_eq!(out_beg_idx, intercept_oracle.out_beg_idx);
    assert_eq!(out_nb_element, intercept_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &intercept_oracle.values);

    let result = context.linearreg_slope(
        0,
        9,
        &input,
        5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, slope_oracle.ret_code);
    assert_eq!(out_beg_idx, slope_oracle.out_beg_idx);
    assert_eq!(out_nb_element, slope_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &slope_oracle.values);

    let result = context.tsf(
        0,
        9,
        &input,
        5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, tsf_oracle.ret_code);
    assert_eq!(out_beg_idx, tsf_oracle.out_beg_idx);
    assert_eq!(out_nb_element, tsf_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &tsf_oracle.values);
}

#[test]
fn cci_mfi_and_adosc_match_c_oracle() {
    let cci_oracle = run_oracle("cci_period_3");
    let mfi_oracle = run_oracle("mfi_period_3");
    let mfi_unstable_oracle = run_oracle("mfi_unstable_2");
    let adosc_oracle = run_oracle("adosc_3_10");
    let mut context = Context::new();

    let high = [10.0, 11.0, 15.0, 13.0, 14.0, 16.0];
    let low = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0];
    let close = [9.0, 10.0, 12.0, 12.5, 13.0, 15.0];
    let volume = [100.0, 110.0, 120.0, 130.0, 140.0, 150.0];
    let high_unstable = [10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0];
    let low_unstable = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0];
    let close_unstable = [9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5];
    let volume_unstable = [100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0];
    let high_long = [
        10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0,
    ];
    let low_long = [
        8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0,
    ];
    let close_long = [
        9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0,
    ];
    let volume_long = [
        100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0, 210.0,
    ];

    let mut out_real = [0.0; 16];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.cci(
        0,
        5,
        &high,
        &low,
        &close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, cci_oracle.ret_code);
    assert_eq!(out_beg_idx, cci_oracle.out_beg_idx);
    assert_eq!(out_nb_element, cci_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &cci_oracle.values);

    let result = context.mfi(
        0,
        5,
        &high,
        &low,
        &close,
        &volume,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, mfi_oracle.ret_code);
    assert_eq!(out_beg_idx, mfi_oracle.out_beg_idx);
    assert_eq!(out_nb_element, mfi_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &mfi_oracle.values);

    context.set_unstable_period(FuncUnstId::Mfi, 2);
    let result = context.mfi(
        0,
        7,
        &high_unstable,
        &low_unstable,
        &close_unstable,
        &volume_unstable,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, mfi_unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, mfi_unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, mfi_unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &mfi_unstable_oracle.values);

    context.set_unstable_period(FuncUnstId::Mfi, 0);
    let result = context.ad_osc(
        0,
        11,
        &high_long,
        &low_long,
        &close_long,
        &volume_long,
        3,
        10,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, adosc_oracle.ret_code);
    assert_eq!(out_beg_idx, adosc_oracle.out_beg_idx);
    assert_eq!(out_nb_element, adosc_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &adosc_oracle.values);
}

#[test]
fn aroon_family_matches_c_oracle() {
    let aroon_oracle = run_oracle_pair("aroon_period_3");
    let aroonosc_oracle = run_oracle("aroonosc_period_3");
    let context = Context::new();

    let high = [10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0];
    let low = [8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0];
    let mut out_real = [0.0; 16];
    let mut out_real_b = [0.0; 16];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.aroon(
        0,
        6,
        &high,
        &low,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, aroon_oracle.ret_code);
    assert_eq!(out_beg_idx, aroon_oracle.out_beg_idx);
    assert_eq!(out_nb_element, aroon_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &aroon_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &aroon_oracle.right);

    let result = context.aroon_osc(
        0,
        6,
        &high,
        &low,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, aroonosc_oracle.ret_code);
    assert_eq!(out_beg_idx, aroonosc_oracle.out_beg_idx);
    assert_eq!(out_nb_element, aroonosc_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &aroonosc_oracle.values);
}

#[test]
fn cmo_matches_c_oracle_default_and_variants() {
    let default_oracle = run_oracle("cmo_default");
    let metastock_oracle = run_oracle("cmo_metastock");
    let unstable_oracle = run_oracle("cmo_unstable_2");

    let input = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64, 46.21,
    ];
    let unstable_input = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 46.50,
    ];

    let mut context = Context::new();
    let mut out_real = [0.0; 32];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.cmo(
        0,
        20,
        &input,
        14,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, default_oracle.ret_code);
    assert_eq!(out_beg_idx, default_oracle.out_beg_idx);
    assert_eq!(out_nb_element, default_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &default_oracle.values);

    assert_eq!(
        context.set_compatibility(Compatibility::Metastock),
        RetCode::Success
    );
    let result = context.cmo(
        0,
        20,
        &input,
        14,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, metastock_oracle.ret_code);
    assert_eq!(out_beg_idx, metastock_oracle.out_beg_idx);
    assert_eq!(out_nb_element, metastock_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &metastock_oracle.values);

    let mut unstable_context = Context::new();
    assert_eq!(
        unstable_context.set_unstable_period(FuncUnstId::Cmo, 2),
        RetCode::Success
    );
    let result = unstable_context.cmo(
        0,
        22,
        &unstable_input,
        14,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &unstable_oracle.values);
}

#[test]
fn stochastic_and_ultimate_oscillators_match_c_oracle() {
    let ultosc_oracle = run_oracle("ultosc_default");
    let stochf_oracle = run_oracle_pair("stochf_default");
    let stoch_oracle = run_oracle_pair("stoch_default");
    let stochrsi_oracle = run_oracle_pair("stochrsi_default");

    let context = Context::new();
    let high = [
        10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0,
        26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0,
    ];
    let low = [
        8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0,
        23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0,
    ];
    let close = [
        9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0,
        25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0,
    ];
    let rsi_input = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 46.50, 46.70, 46.90, 47.10,
        47.30, 47.50, 47.70, 47.90,
    ];

    let mut out_real = [0.0; 64];
    let mut out_real_b = [0.0; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.ultosc(
        0,
        29,
        &high,
        &low,
        &close,
        7,
        14,
        28,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ultosc_oracle.ret_code);
    assert_eq!(out_beg_idx, ultosc_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ultosc_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ultosc_oracle.values);

    let result = context.stochf(
        0,
        11,
        &high[..12],
        &low[..12],
        &close[..12],
        5,
        3,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, stochf_oracle.ret_code);
    assert_eq!(out_beg_idx, stochf_oracle.out_beg_idx);
    assert_eq!(out_nb_element, stochf_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &stochf_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &stochf_oracle.right);

    let result = context.stoch(
        0,
        11,
        &high[..12],
        &low[..12],
        &close[..12],
        5,
        3,
        0,
        3,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, stoch_oracle.ret_code);
    assert_eq!(out_beg_idx, stoch_oracle.out_beg_idx);
    assert_eq!(out_nb_element, stoch_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &stoch_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &stoch_oracle.right);

    let result = context.stochrsi(
        0,
        29,
        &rsi_input,
        14,
        5,
        3,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, stochrsi_oracle.ret_code);
    assert_eq!(out_beg_idx, stochrsi_oracle.out_beg_idx);
    assert_eq!(out_nb_element, stochrsi_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &stochrsi_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &stochrsi_oracle.right);
}

#[test]
fn hilbert_dominant_cycle_family_matches_c_oracle() {
    let dcperiod_oracle = run_oracle("ht_dcperiod_default");
    let dcphase_oracle = run_oracle("ht_dcphase_default");
    let phasor_oracle = run_oracle_pair("ht_phasor_default");
    let context = Context::new();
    let input: Vec<f64> = (1..=90).map(|value| value as f64).collect();

    let mut out_real = [0.0; 128];
    let mut out_real_b = [0.0; 128];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.ht_dc_period(
        0,
        89,
        &input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, dcperiod_oracle.ret_code);
    assert_eq!(out_beg_idx, dcperiod_oracle.out_beg_idx);
    assert_eq!(out_nb_element, dcperiod_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &dcperiod_oracle.values);

    let result = context.ht_dc_phase(
        0,
        89,
        &input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, dcphase_oracle.ret_code);
    assert_eq!(out_beg_idx, dcphase_oracle.out_beg_idx);
    assert_eq!(out_nb_element, dcphase_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &dcphase_oracle.values);

    let result = context.ht_phasor(
        0,
        89,
        &input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, phasor_oracle.ret_code);
    assert_eq!(out_beg_idx, phasor_oracle.out_beg_idx);
    assert_eq!(out_nb_element, phasor_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &phasor_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &phasor_oracle.right);
}

#[test]
fn hilbert_sine_and_trend_family_matches_c_oracle() {
    let sine_oracle = run_oracle_pair("ht_sine_default");
    let trendline_oracle = run_oracle("ht_trendline_default");
    let trendmode_oracle = run_oracle_int("ht_trendmode_default");
    let context = Context::new();
    let input: Vec<f64> = (1..=90).map(|value| value as f64).collect();

    let mut out_real = [0.0; 128];
    let mut out_real_b = [0.0; 128];
    let mut out_int = [0; 128];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.ht_sine(
        0,
        89,
        &input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, sine_oracle.ret_code);
    assert_eq!(out_beg_idx, sine_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sine_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sine_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &sine_oracle.right);

    let result = context.ht_trendline(
        0,
        89,
        &input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, trendline_oracle.ret_code);
    assert_eq!(out_beg_idx, trendline_oracle.out_beg_idx);
    assert_eq!(out_nb_element, trendline_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &trendline_oracle.values);

    let result = context.ht_trend_mode(
        0,
        89,
        &input,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_int,
    );
    assert_eq!(result as i32, trendmode_oracle.ret_code);
    assert_eq!(out_beg_idx, trendmode_oracle.out_beg_idx);
    assert_eq!(out_nb_element, trendmode_oracle.out_nb_element);
    assert_eq!(
        &out_int[..out_nb_element],
        trendmode_oracle.values.as_slice()
    );
}

#[test]
fn moving_average_difference_family_matches_c_oracle() {
    let apo_oracle = run_oracle("apo_sma_default");
    let ppo_oracle = run_oracle("ppo_ema");
    let macd_oracle = run_oracle_triple("macd_basic");
    let macdfix_oracle = run_oracle_triple("macdfix_basic");
    let context = Context::new();

    let apo_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ];
    let ppo_input = [
        10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0,
    ];
    let macd_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0,
    ];
    let macdfix_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0,
    ];

    let mut out_real = [0.0; 64];
    let mut out_real_b = [0.0; 64];
    let mut out_real_c = [0.0; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.apo(
        0,
        11,
        &apo_input,
        3,
        5,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, apo_oracle.ret_code);
    assert_eq!(out_beg_idx, apo_oracle.out_beg_idx);
    assert_eq!(out_nb_element, apo_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &apo_oracle.values);

    let result = context.ppo(
        0,
        11,
        &ppo_input,
        3,
        5,
        1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ppo_oracle.ret_code);
    assert_eq!(out_beg_idx, ppo_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ppo_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ppo_oracle.values);

    let result = context.macd(
        0,
        17,
        &macd_input,
        3,
        6,
        4,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
        &mut out_real_c,
    );
    assert_eq!(result as i32, macd_oracle.ret_code);
    assert_eq!(out_beg_idx, macd_oracle.out_beg_idx);
    assert_eq!(out_nb_element, macd_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &macd_oracle.first);
    assert_close(&out_real_b[..out_nb_element], &macd_oracle.second);
    assert_close(&out_real_c[..out_nb_element], &macd_oracle.third);

    let result = context.macdfix(
        0,
        29,
        &macdfix_input,
        4,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
        &mut out_real_c,
    );
    assert_eq!(result as i32, macdfix_oracle.ret_code);
    assert_eq!(out_beg_idx, macdfix_oracle.out_beg_idx);
    assert_eq!(out_nb_element, macdfix_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &macdfix_oracle.first);
    assert_close(&out_real_b[..out_nb_element], &macdfix_oracle.second);
    assert_close(&out_real_c[..out_nb_element], &macdfix_oracle.third);
}

#[test]
fn ema_chain_overlap_functions_match_c_oracle() {
    let dema_oracle = run_oracle("dema_period_3");
    let kama_oracle = run_oracle("kama_period_3");
    let kama_unstable_oracle = run_oracle("kama_period_3_unstable_2");
    let mama_oracle = run_oracle_pair("mama_default");
    let mama_unstable_oracle = run_oracle_pair("mama_unstable_2");
    let tema_oracle = run_oracle("tema_period_3");
    let t3_oracle = run_oracle("t3_period_5");
    let t3_unstable_oracle = run_oracle("t3_period_5_unstable_2");
    let trima_oracle = run_oracle("trima_period_5");
    let trix_oracle = run_oracle("trix_period_3");
    let ma_oracle = run_oracle("ma_sma_period_3");
    let ma_wma_oracle = run_oracle("ma_wma_period_3");
    let wma_oracle = run_oracle("wma_period_3");
    let context = Context::new();

    let dema_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ];
    let kama_input = [1.0, 2.0, 3.0, 4.0, 3.5, 4.5, 5.5, 6.0, 7.0, 7.5, 8.5, 9.0];
    let kama_unstable_input = [
        1.0, 2.0, 3.0, 4.0, 3.5, 4.5, 5.5, 6.0, 7.0, 7.5, 8.5, 9.0, 9.5, 10.5,
    ];
    let mama_input = [
        10.0, 10.4, 10.8, 11.1, 11.5, 11.8, 12.0, 12.1, 12.0, 11.9, 11.7, 11.8, 12.0, 12.3, 12.7,
        13.0, 13.2, 13.1, 12.9, 12.7, 12.6, 12.8, 13.1, 13.5, 13.8, 14.0, 14.2, 14.1, 13.9, 13.7,
        13.5, 13.6, 13.9, 14.3, 14.6, 14.8, 15.0, 15.1, 15.0, 14.8, 14.6, 14.5, 14.7, 15.0, 15.4,
    ];
    let mama_unstable_input = [
        10.0, 10.4, 10.8, 11.1, 11.5, 11.8, 12.0, 12.1, 12.0, 11.9, 11.7, 11.8, 12.0, 12.3, 12.7,
        13.0, 13.2, 13.1, 12.9, 12.7, 12.6, 12.8, 13.1, 13.5, 13.8, 14.0, 14.2, 14.1, 13.9, 13.7,
        13.5, 13.6, 13.9, 14.3, 14.6, 14.8, 15.0, 15.1, 15.0, 14.8, 14.6, 14.5, 14.7, 15.0, 15.4,
        15.7, 15.9,
    ];
    let t3_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0,
        32.0, 33.0, 34.0, 35.0,
    ];
    let t3_unstable_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0,
        32.0, 33.0, 34.0, 35.0, 36.0, 37.0,
    ];
    let tema_input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];
    let trima_input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let ma_input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];

    let mut out_real = [0.0; 64];
    let mut out_real_b = [0.0; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.dema(
        0,
        11,
        &dema_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, dema_oracle.ret_code);
    assert_eq!(out_beg_idx, dema_oracle.out_beg_idx);
    assert_eq!(out_nb_element, dema_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &dema_oracle.values);

    let result = context.kama(
        0,
        11,
        &kama_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, kama_oracle.ret_code);
    assert_eq!(out_beg_idx, kama_oracle.out_beg_idx);
    assert_eq!(out_nb_element, kama_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &kama_oracle.values);

    let result = context.mama(
        0,
        44,
        &mama_input,
        0.5,
        0.05,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, mama_oracle.ret_code);
    assert_eq!(out_beg_idx, mama_oracle.out_beg_idx);
    assert_eq!(out_nb_element, mama_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &mama_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &mama_oracle.right);

    let result = context.tema(
        0,
        14,
        &tema_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, tema_oracle.ret_code);
    assert_eq!(out_beg_idx, tema_oracle.out_beg_idx);
    assert_eq!(out_nb_element, tema_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &tema_oracle.values);

    let result = context.t3(
        0,
        34,
        &t3_input,
        5,
        0.7,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, t3_oracle.ret_code);
    assert_eq!(out_beg_idx, t3_oracle.out_beg_idx);
    assert_eq!(out_nb_element, t3_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &t3_oracle.values);

    let result = context.trima(
        0,
        9,
        &trima_input,
        5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, trima_oracle.ret_code);
    assert_eq!(out_beg_idx, trima_oracle.out_beg_idx);
    assert_eq!(out_nb_element, trima_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &trima_oracle.values);

    let result = context.trix(
        0,
        14,
        &tema_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, trix_oracle.ret_code);
    assert_eq!(out_beg_idx, trix_oracle.out_beg_idx);
    assert_eq!(out_nb_element, trix_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &trix_oracle.values);

    let result = context.ma(
        0,
        5,
        &ma_input,
        3,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ma_oracle.ret_code);
    assert_eq!(out_beg_idx, ma_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ma_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ma_oracle.values);

    let result = context.wma(
        0,
        5,
        &ma_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, wma_oracle.ret_code);
    assert_eq!(out_beg_idx, wma_oracle.out_beg_idx);
    assert_eq!(out_nb_element, wma_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &wma_oracle.values);

    let result = context.ma(
        0,
        5,
        &ma_input,
        3,
        2,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, ma_wma_oracle.ret_code);
    assert_eq!(out_beg_idx, ma_wma_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ma_wma_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &ma_wma_oracle.values);

    let mut unstable_context = Context::new();
    assert_eq!(
        unstable_context.set_unstable_period(FuncUnstId::Kama, 2),
        RetCode::Success
    );
    let result = unstable_context.kama(
        0,
        13,
        &kama_unstable_input,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, kama_unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, kama_unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, kama_unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &kama_unstable_oracle.values);

    let mut unstable_context = Context::new();
    assert_eq!(
        unstable_context.set_unstable_period(FuncUnstId::Mama, 2),
        RetCode::Success
    );
    let result = unstable_context.mama(
        0,
        46,
        &mama_unstable_input,
        0.5,
        0.05,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
    );
    assert_eq!(result as i32, mama_unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, mama_unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, mama_unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &mama_unstable_oracle.left);
    assert_close(&out_real_b[..out_nb_element], &mama_unstable_oracle.right);

    let mut unstable_context = Context::new();
    assert_eq!(
        unstable_context.set_unstable_period(FuncUnstId::T3, 2),
        RetCode::Success
    );
    let result = unstable_context.t3(
        0,
        36,
        &t3_unstable_input,
        5,
        0.7,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, t3_unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, t3_unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, t3_unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &t3_unstable_oracle.values);
}

#[test]
fn bbands_matches_c_oracle() {
    let sma_oracle = run_oracle_triple("bbands_sma_default");
    let ema_oracle = run_oracle_triple("bbands_ema_default");
    let context = Context::new();
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let mut out_upper = [0.0; 32];
    let mut out_middle = [0.0; 32];
    let mut out_lower = [0.0; 32];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.bbands(
        0,
        9,
        &input,
        5,
        2.0,
        2.0,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_upper,
        &mut out_middle,
        &mut out_lower,
    );
    assert_eq!(result as i32, sma_oracle.ret_code);
    assert_eq!(out_beg_idx, sma_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sma_oracle.out_nb_element);
    assert_close(&out_upper[..out_nb_element], &sma_oracle.first);
    assert_close(&out_middle[..out_nb_element], &sma_oracle.second);
    assert_close(&out_lower[..out_nb_element], &sma_oracle.third);

    let result = context.bbands(
        0,
        9,
        &input,
        5,
        2.0,
        1.5,
        1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_upper,
        &mut out_middle,
        &mut out_lower,
    );
    assert_eq!(result as i32, ema_oracle.ret_code);
    assert_eq!(out_beg_idx, ema_oracle.out_beg_idx);
    assert_eq!(out_nb_element, ema_oracle.out_nb_element);
    assert_close(&out_upper[..out_nb_element], &ema_oracle.first);
    assert_close(&out_middle[..out_nb_element], &ema_oracle.second);
    assert_close(&out_lower[..out_nb_element], &ema_oracle.third);
}

#[test]
fn remaining_overlap_tail_matches_c_oracle() {
    let accbands_oracle = run_oracle_triple("accbands_period_3");
    let mavp_oracle = run_oracle("mavp_sma");
    let sar_oracle = run_oracle("sar_default");
    let sarext_oracle = run_oracle("sarext_default");
    let context = Context::new();

    let high = [10.0, 11.0, 12.0, 14.0, 13.0, 15.0];
    let low = [8.0, 9.0, 10.0, 11.0, 11.5, 12.0];
    let close = [9.0, 10.0, 11.0, 13.0, 12.0, 14.0];
    let mavp_input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let mavp_periods = [2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 2.0, 5.0, 3.0, 4.0];
    let sar_high = [10.0, 11.0, 12.0, 11.5, 13.0, 14.0, 13.5, 15.0];
    let sar_low = [9.0, 9.5, 10.5, 10.0, 11.0, 12.5, 12.0, 13.0];

    let mut out_real = [0.0; 32];
    let mut out_real_b = [0.0; 32];
    let mut out_real_c = [0.0; 32];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.accbands(
        0,
        5,
        &high,
        &low,
        &close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
        &mut out_real_c,
    );
    assert_eq!(result as i32, accbands_oracle.ret_code);
    assert_eq!(out_beg_idx, accbands_oracle.out_beg_idx);
    assert_eq!(out_nb_element, accbands_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &accbands_oracle.first);
    assert_close(&out_real_b[..out_nb_element], &accbands_oracle.second);
    assert_close(&out_real_c[..out_nb_element], &accbands_oracle.third);

    let result = context.mavp(
        0,
        9,
        &mavp_input,
        &mavp_periods,
        2,
        5,
        0,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, mavp_oracle.ret_code);
    assert_eq!(out_beg_idx, mavp_oracle.out_beg_idx);
    assert_eq!(out_nb_element, mavp_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &mavp_oracle.values);

    let result = context.sar(
        0,
        7,
        &sar_high,
        &sar_low,
        0.02,
        0.2,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, sar_oracle.ret_code);
    assert_eq!(out_beg_idx, sar_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sar_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sar_oracle.values);

    let result = context.sarext(
        0,
        7,
        &sar_high,
        &sar_low,
        0.0,
        0.0,
        0.02,
        0.02,
        0.2,
        0.02,
        0.02,
        0.2,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, sarext_oracle.ret_code);
    assert_eq!(out_beg_idx, sarext_oracle.out_beg_idx);
    assert_eq!(out_nb_element, sarext_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &sarext_oracle.values);
}

#[test]
fn macdext_matches_c_oracle() {
    let oracle = run_oracle_triple("macdext_basic");
    let context = Context::new();
    let input = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0,
    ];

    let mut out_real = [0.0; 64];
    let mut out_real_b = [0.0; 64];
    let mut out_real_c = [0.0; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.macdext(
        0,
        17,
        &input,
        3,
        1,
        6,
        1,
        4,
        1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
        &mut out_real_b,
        &mut out_real_c,
    );
    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.first);
    assert_close(&out_real_b[..out_nb_element], &oracle.second);
    assert_close(&out_real_c[..out_nb_element], &oracle.third);
}

#[test]
fn directional_movement_family_matches_c_oracle() {
    let plus_dm_oracle = run_oracle("plus_dm_period_3");
    let minus_dm_oracle = run_oracle("minus_dm_period_3");
    let plus_di_oracle = run_oracle("plus_di_period_3");
    let minus_di_oracle = run_oracle("minus_di_period_3");
    let dx_oracle = run_oracle("dx_period_3");
    let adx_oracle = run_oracle("adx_period_3");
    let adxr_oracle = run_oracle("adxr_period_3");

    let context = Context::new();
    let high = [
        10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0,
    ];
    let low = [
        8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0,
    ];
    let close = [
        9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0,
    ];

    let mut out_real = [0.0; 32];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.plus_dm(
        0,
        6,
        &high[..7],
        &low[..7],
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, plus_dm_oracle.ret_code);
    assert_eq!(out_beg_idx, plus_dm_oracle.out_beg_idx);
    assert_eq!(out_nb_element, plus_dm_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &plus_dm_oracle.values);

    let result = context.minus_dm(
        0,
        6,
        &high[..7],
        &low[..7],
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, minus_dm_oracle.ret_code);
    assert_eq!(out_beg_idx, minus_dm_oracle.out_beg_idx);
    assert_eq!(out_nb_element, minus_dm_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &minus_dm_oracle.values);

    let result = context.plus_di(
        0,
        6,
        &high[..7],
        &low[..7],
        &close[..7],
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, plus_di_oracle.ret_code);
    assert_eq!(out_beg_idx, plus_di_oracle.out_beg_idx);
    assert_eq!(out_nb_element, plus_di_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &plus_di_oracle.values);

    let result = context.minus_di(
        0,
        6,
        &high[..7],
        &low[..7],
        &close[..7],
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, minus_di_oracle.ret_code);
    assert_eq!(out_beg_idx, minus_di_oracle.out_beg_idx);
    assert_eq!(out_nb_element, minus_di_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &minus_di_oracle.values);

    let result = context.dx(
        0,
        7,
        &high[..8],
        &low[..8],
        &close[..8],
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, dx_oracle.ret_code);
    assert_eq!(out_beg_idx, dx_oracle.out_beg_idx);
    assert_eq!(out_nb_element, dx_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &dx_oracle.values);

    let result = context.adx(
        0,
        9,
        &high[..10],
        &low[..10],
        &close[..10],
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, adx_oracle.ret_code);
    assert_eq!(out_beg_idx, adx_oracle.out_beg_idx);
    assert_eq!(out_nb_element, adx_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &adx_oracle.values);

    let result = context.adxr(
        0,
        11,
        &high,
        &low,
        &close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, adxr_oracle.ret_code);
    assert_eq!(out_beg_idx, adxr_oracle.out_beg_idx);
    assert_eq!(out_nb_element, adxr_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &adxr_oracle.values);
}

#[test]
fn adx_unstable_period_matches_c_oracle() {
    let oracle = run_oracle("adx_unstable_2");
    let mut context = Context::new();
    assert_eq!(
        context.set_unstable_period(FuncUnstId::Adx, 2),
        RetCode::Success
    );

    let high = [
        10.0, 11.0, 15.0, 13.0, 14.0, 16.0, 18.0, 17.0, 19.0, 20.0, 21.0, 22.0,
    ];
    let low = [
        8.0, 9.0, 10.0, 11.0, 12.0, 14.0, 15.0, 14.0, 16.0, 17.0, 18.0, 19.0,
    ];
    let close = [
        9.0, 10.0, 12.0, 12.5, 13.0, 15.0, 16.0, 15.5, 18.0, 19.0, 20.0, 21.0,
    ];
    let mut out_real = [0.0; 32];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.adx(
        0,
        11,
        &high,
        &low,
        &close,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
}

#[test]
fn sub_matches_c_oracle() {
    let oracle = run_oracle("sub_basic");
    let context = Context::new();
    let in_real0 = [10.0, 9.0, 8.0, 7.0, 6.0];
    let in_real1 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.sub(
        0,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
}

#[test]
fn mult_matches_c_oracle() {
    let oracle = run_oracle("mult_basic");
    let context = Context::new();
    let in_real0 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let in_real1 = [2.0, 3.0, 4.0, 5.0, 6.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.mult(
        0,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
}

#[test]
fn div_matches_c_oracle() {
    let oracle = run_oracle("div_basic");
    let context = Context::new();
    let in_real0 = [10.0, 9.0, 8.0, 7.0, 6.0];
    let in_real1 = [2.0, 3.0, 4.0, 5.0, 6.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.div(
        0,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
}

#[test]
fn sma_matches_c_oracle() {
    let oracle = run_oracle("sma_period_3");
    let context = Context::new();
    let in_real = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let mut out_real = [0.0; 6];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.sma(
        0,
        5,
        &in_real,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
}

#[test]
fn ema_matches_c_oracle_default_and_variants() {
    let default_oracle = run_oracle("ema_default");
    let metastock_oracle = run_oracle("ema_metastock");
    let unstable_oracle = run_oracle("ema_unstable_2");

    let in_real_default = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let mut out_real = [0.0; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let context = Context::new();
    let result = context.ema(
        0,
        6,
        &in_real_default,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, default_oracle.ret_code);
    assert_eq!(out_beg_idx, default_oracle.out_beg_idx);
    assert_eq!(out_nb_element, default_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &default_oracle.values);

    let mut metastock = Context::new();
    assert_eq!(
        metastock.set_compatibility(Compatibility::Metastock),
        RetCode::Success
    );
    let result = metastock.ema(
        0,
        6,
        &in_real_default,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, metastock_oracle.ret_code);
    assert_eq!(out_beg_idx, metastock_oracle.out_beg_idx);
    assert_eq!(out_nb_element, metastock_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &metastock_oracle.values);

    let in_real_unstable = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let mut unstable = Context::new();
    assert_eq!(
        unstable.set_unstable_period(FuncUnstId::Ema, 2),
        RetCode::Success
    );
    let result = unstable.ema(
        0,
        7,
        &in_real_unstable,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &unstable_oracle.values);
}

#[test]
fn rsi_matches_c_oracle_default_and_variants() {
    let default_oracle = run_oracle("rsi_default");
    let metastock_oracle = run_oracle("rsi_metastock");
    let unstable_oracle = run_oracle("rsi_unstable_2");

    let in_real_default = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64, 46.21,
    ];
    let mut out_real = [0.0; 32];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let context = Context::new();
    let result = context.rsi(
        0,
        20,
        &in_real_default,
        14,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, default_oracle.ret_code);
    assert_eq!(out_beg_idx, default_oracle.out_beg_idx);
    assert_eq!(out_nb_element, default_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &default_oracle.values);

    let mut metastock = Context::new();
    assert_eq!(
        metastock.set_compatibility(Compatibility::Metastock),
        RetCode::Success
    );
    let result = metastock.rsi(
        0,
        20,
        &in_real_default,
        14,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, metastock_oracle.ret_code);
    assert_eq!(out_beg_idx, metastock_oracle.out_beg_idx);
    assert_eq!(out_nb_element, metastock_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &metastock_oracle.values);

    let in_real_unstable = [
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08, 45.89, 46.03, 45.61,
        46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64, 46.21, 46.25, 46.50,
    ];
    let mut unstable = Context::new();
    assert_eq!(
        unstable.set_unstable_period(FuncUnstId::Rsi, 2),
        RetCode::Success
    );
    let result = unstable.rsi(
        0,
        22,
        &in_real_unstable,
        14,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );
    assert_eq!(result as i32, unstable_oracle.ret_code);
    assert_eq!(out_beg_idx, unstable_oracle.out_beg_idx);
    assert_eq!(out_nb_element, unstable_oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &unstable_oracle.values);
}

#[test]
fn compatibility_facade_executes_seed_function_after_initialize() {
    let _guard = compatibility_runtime_test_lock()
        .lock()
        .expect("compatibility runtime test mutex poisoned");
    assert_eq!(initialize(), RetCode::Success);

    let in_real0 = [1.0, 2.0, 3.0, 4.0, 5.0];
    let in_real1 = [5.0, 4.0, 3.0, 2.0, 1.0];
    let mut out_real = [0.0; 5];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = Core::add(
        0,
        4,
        &in_real0,
        &in_real1,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result, RetCode::Success);
    assert_eq!(out_beg_idx, 0);
    assert_eq!(out_nb_element, 5);
    assert_eq!(out_real, [6.0, 6.0, 6.0, 6.0, 6.0]);
    assert_eq!(shutdown(), RetCode::Success);
}

#[test]
fn candlestick_patterns_match_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let base_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0,
    ];
    let base_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.05,
    ];
    let base_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.5,
    ];
    let base_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.5,
    ];

    let doji_oracle = run_oracle_int("cdldoji_basic");
    let result = context.cdl_doji(
        0,
        11,
        &base_open,
        &base_high,
        &base_low,
        &base_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, doji_oracle.ret_code);
    assert_eq!(out_beg_idx, doji_oracle.out_beg_idx);
    assert_eq!(out_nb_element, doji_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        doji_oracle.values.as_slice()
    );

    let dragonfly_oracle = run_oracle_int("cdldragonflydoji_basic");
    let dragonfly_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.05,
    ];
    let dragonfly_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 10.8,
    ];
    let dragonfly_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.02,
    ];
    let result = context.cdl_dragonfly_doji(
        0,
        11,
        &base_open,
        &dragonfly_high,
        &dragonfly_low,
        &dragonfly_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, dragonfly_oracle.ret_code);
    assert_eq!(out_beg_idx, dragonfly_oracle.out_beg_idx);
    assert_eq!(out_nb_element, dragonfly_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        dragonfly_oracle.values.as_slice()
    );

    let gravestone_oracle = run_oracle_int("cdlgravestonedoji_basic");
    let gravestone_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 13.2,
    ];
    let gravestone_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.99,
    ];
    let result = context.cdl_gravestone_doji(
        0,
        11,
        &base_open,
        &gravestone_high,
        &gravestone_low,
        &dragonfly_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, gravestone_oracle.ret_code);
    assert_eq!(out_beg_idx, gravestone_oracle.out_beg_idx);
    assert_eq!(out_nb_element, gravestone_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        gravestone_oracle.values.as_slice()
    );

    let spinning_oracle = run_oracle_int("cdlspinningtop_basic");
    let spinning_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 13.4,
    ];
    let spinning_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.2,
    ];
    let spinning_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.6,
    ];
    let result = context.cdl_spinning_top(
        0,
        11,
        &base_open,
        &spinning_high,
        &spinning_low,
        &spinning_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, spinning_oracle.ret_code);
    assert_eq!(out_beg_idx, spinning_oracle.out_beg_idx);
    assert_eq!(out_nb_element, spinning_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        spinning_oracle.values.as_slice()
    );

    let marubozu_oracle = run_oracle_int("cdlmarubozu_basic");
    let marubozu_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 14.05,
    ];
    let marubozu_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.95,
    ];
    let marubozu_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 14.0,
    ];
    let result = context.cdl_marubozu(
        0,
        11,
        &base_open,
        &marubozu_high,
        &marubozu_low,
        &marubozu_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, marubozu_oracle.ret_code);
    assert_eq!(out_beg_idx, marubozu_oracle.out_beg_idx);
    assert_eq!(out_nb_element, marubozu_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        marubozu_oracle.values.as_slice()
    );

    let long_line_oracle = run_oracle_int("cdllongline_basic");
    let long_line_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 14.2,
    ];
    let long_line_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8,
    ];
    let result = context.cdl_long_line(
        0,
        11,
        &base_open,
        &long_line_high,
        &long_line_low,
        &marubozu_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, long_line_oracle.ret_code);
    assert_eq!(out_beg_idx, long_line_oracle.out_beg_idx);
    assert_eq!(out_nb_element, long_line_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        long_line_oracle.values.as_slice()
    );

    let short_line_oracle = run_oracle_int("cdlshortline_basic");
    let short_line_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.8,
    ];
    let short_line_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8,
    ];
    let result = context.cdl_short_line(
        0,
        11,
        &base_open,
        &short_line_high,
        &short_line_low,
        &spinning_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, short_line_oracle.ret_code);
    assert_eq!(out_beg_idx, short_line_oracle.out_beg_idx);
    assert_eq!(out_nb_element, short_line_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        short_line_oracle.values.as_slice()
    );

    let hammer_oracle = run_oracle_int("cdlhammer_basic");
    let hammer_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 11.0, 10.4,
    ];
    let hammer_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 11.6, 10.65,
    ];
    let hammer_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.2, 9.4,
    ];
    let hammer_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.4, 10.6,
    ];
    let result = context.cdl_hammer(
        0,
        11,
        &hammer_open,
        &hammer_high,
        &hammer_low,
        &hammer_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, hammer_oracle.ret_code);
    assert_eq!(out_beg_idx, hammer_oracle.out_beg_idx);
    assert_eq!(out_nb_element, hammer_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        hammer_oracle.values.as_slice()
    );

    let engulfing_oracle = run_oracle_int("cdlengulfing_basic");
    let engulfing_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 10.8,
    ];
    let engulfing_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.2, 12.6,
    ];
    let engulfing_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.8, 10.6,
    ];
    let engulfing_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.0, 12.4,
    ];
    let result = context.cdl_engulfing(
        0,
        11,
        &engulfing_open,
        &engulfing_high,
        &engulfing_low,
        &engulfing_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, engulfing_oracle.ret_code);
    assert_eq!(out_beg_idx, engulfing_oracle.out_beg_idx);
    assert_eq!(out_nb_element, engulfing_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        engulfing_oracle.values.as_slice()
    );

    let harami_oracle = run_oracle_int("cdlharami_basic");
    let harami_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.2, 11.3,
    ];
    let harami_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.8, 10.7,
    ];
    let harami_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 11.2,
    ];
    let result = context.cdl_harami(
        0,
        11,
        &engulfing_open,
        &harami_high,
        &harami_low,
        &harami_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, harami_oracle.ret_code);
    assert_eq!(out_beg_idx, harami_oracle.out_beg_idx);
    assert_eq!(out_nb_element, harami_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        harami_oracle.values.as_slice()
    );

    let harami_cross_oracle = run_oracle_int("cdlharamicross_basic");
    let harami_cross_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 10.0, 11.1,
    ];
    let harami_cross_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.2, 11.4,
    ];
    let harami_cross_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.8, 11.0,
    ];
    let harami_cross_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 12.0, 11.12,
    ];
    let result = context.cdl_harami_cross(
        0,
        11,
        &harami_cross_open,
        &harami_cross_high,
        &harami_cross_low,
        &harami_cross_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, harami_cross_oracle.ret_code);
    assert_eq!(out_beg_idx, harami_cross_oracle.out_beg_idx);
    assert_eq!(out_nb_element, harami_cross_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        harami_cross_oracle.values.as_slice()
    );
}

#[test]
fn candlestick_patterns_respect_candle_setting_overrides() {
    let oracle = run_oracle_int("cdldoji_bodydoji_override");
    let mut context = Context::new();
    assert_eq!(
        context.set_candle_settings(
            ta_lib::CandleSettingType::BodyDoji,
            ta_lib::RangeType::HighLow,
            10,
            0.4
        ),
        RetCode::Success
    );

    let in_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0,
    ];
    let in_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.5,
    ];
    let in_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.5,
    ];
    let in_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.20,
    ];
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = context.cdl_doji(
        0,
        11,
        &in_open,
        &in_high,
        &in_low,
        &in_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_eq!(&out_integer[..out_nb_element], oracle.values.as_slice());
}

#[test]
fn compatibility_facade_executes_candlestick_pattern_after_initialize() {
    let _guard = compatibility_runtime_test_lock()
        .lock()
        .expect("compatibility runtime test mutex poisoned");
    assert_eq!(initialize(), RetCode::Success);

    let in_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0,
    ];
    let in_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.5,
    ];
    let in_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.5,
    ];
    let in_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.05,
    ];
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = Core::cdl_doji(
        0,
        11,
        &in_open,
        &in_high,
        &in_low,
        &in_close,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );

    assert_eq!(result, RetCode::Success);
    assert_eq!(out_beg_idx, 10);
    assert_eq!(out_nb_element, 2);
    assert_eq!(&out_integer[..out_nb_element], &[0, 100]);
    assert_eq!(shutdown(), RetCode::Success);
}

#[test]
fn compatibility_facade_applies_compatibility_mode_to_execution() {
    let _guard = compatibility_runtime_test_lock()
        .lock()
        .expect("compatibility runtime test mutex poisoned");
    let oracle = run_oracle("ema_metastock");
    assert_eq!(initialize(), RetCode::Success);
    assert_eq!(
        Core::set_compatibility(Compatibility::Metastock),
        RetCode::Success
    );

    let in_real = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let mut out_real = [0.0; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = Core::ema(
        0,
        6,
        &in_real,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
    assert_eq!(shutdown(), RetCode::Success);
}

#[test]
fn compatibility_facade_applies_unstable_period_to_execution() {
    let _guard = compatibility_runtime_test_lock()
        .lock()
        .expect("compatibility runtime test mutex poisoned");
    let oracle = run_oracle("ema_unstable_2");
    assert_eq!(initialize(), RetCode::Success);
    assert_eq!(
        Core::set_unstable_period(FuncUnstId::Ema, 2),
        RetCode::Success
    );

    let in_real = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let mut out_real = [0.0; 8];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = Core::ema(
        0,
        7,
        &in_real,
        3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_real,
    );

    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_close(&out_real[..out_nb_element], &oracle.values);
    assert_eq!(shutdown(), RetCode::Success);
}

#[test]
fn compatibility_facade_restores_candle_settings_to_default() {
    let _guard = compatibility_runtime_test_lock()
        .lock()
        .expect("compatibility runtime test mutex poisoned");
    let override_oracle = run_oracle_int("cdldoji_bodydoji_override");
    let default_oracle = run_oracle_int("cdldoji_basic");
    assert_eq!(initialize(), RetCode::Success);
    assert_eq!(
        Core::set_candle_settings(
            ta_lib::CandleSettingType::BodyDoji,
            ta_lib::RangeType::HighLow,
            10,
            0.4
        ),
        RetCode::Success
    );

    let in_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0,
    ];
    let in_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.5,
    ];
    let in_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.5,
    ];
    let in_close_override = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.20,
    ];
    let in_close_default = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.05,
    ];
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    let result = Core::cdl_doji(
        0,
        11,
        &in_open,
        &in_high,
        &in_low,
        &in_close_override,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, override_oracle.ret_code);
    assert_eq!(out_beg_idx, override_oracle.out_beg_idx);
    assert_eq!(out_nb_element, override_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        override_oracle.values.as_slice()
    );

    assert_eq!(
        Core::restore_candle_default_settings(ta_lib::CandleSettingType::BodyDoji),
        RetCode::Success
    );

    let result = Core::cdl_doji(
        0,
        11,
        &in_open,
        &in_high,
        &in_low,
        &in_close_default,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, default_oracle.ret_code);
    assert_eq!(out_beg_idx, default_oracle.out_beg_idx);
    assert_eq!(out_nb_element, default_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        default_oracle.values.as_slice()
    );
    assert_eq!(shutdown(), RetCode::Success);
}

#[test]
fn generated_default_matrix_matches_c_oracle() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let rust_run = (case.default_run)(&context);
        let oracle_case = format!("matrix_default_{}", case.rust_name);
        assert_generated_smoke_matches_oracle(&rust_run, &oracle_case);
    }
}

#[test]
fn generated_boundary_matrix_matches_c_oracle() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let rust_run = (case.boundary_run)(&context);
        let oracle_case = format!("matrix_boundary_{}", case.rust_name);
        assert_generated_smoke_matches_oracle(&rust_run, &oracle_case);
    }
}

#[test]
fn generated_seeded_matrix_matches_c_oracle() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let rust_run = (case.seeded_run)(&context);
        let oracle_case = format!("matrix_seeded_{}", case.rust_name);
        assert_generated_smoke_matches_oracle(&rust_run, &oracle_case);
    }
}

#[test]
fn generated_variant_matrix_matches_c_oracle() {
    let context = Context::new();

    for case in SMOKE_CASES {
        let rust_run = (case.variant_run)(&context);
        let oracle_case = format!("matrix_variant_{}", case.rust_name);
        assert_generated_smoke_matches_oracle(&rust_run, &oracle_case);
    }
}

#[test]
fn additional_candlestick_patterns_match_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    macro_rules! assert_pattern {
        ($oracle_name:literal, $method:ident, $open:expr, $high:expr, $low:expr, $close:expr) => {{
            let oracle = run_oracle_int($oracle_name);
            let open = $open;
            let high = $high;
            let low = $low;
            let close = $close;
            let result = context.$method(
                0,
                11,
                &open,
                &high,
                &low,
                &close,
                &mut out_beg_idx,
                &mut out_nb_element,
                &mut out_integer,
            );
            assert_eq!(result as i32, oracle.ret_code, "{}", $oracle_name);
            assert_eq!(out_beg_idx, oracle.out_beg_idx, "{}", $oracle_name);
            assert_eq!(out_nb_element, oracle.out_nb_element, "{}", $oracle_name);
            assert_eq!(
                &out_integer[..out_nb_element],
                oracle.values.as_slice(),
                "{}",
                $oracle_name
            );
        }};
    }

    assert_pattern!(
        "cdlbelthold_basic",
        cdl_belt_hold,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 14.2
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.98
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 14.1
        ]
    );
    assert_pattern!(
        "cdlclosingmarubozu_basic",
        cdl_closing_marubozu,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 14.12
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.6
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 14.1
        ]
    );
    assert_pattern!(
        "cdlhighwave_basic",
        cdl_high_wave,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 14.4
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 10.7
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.3
        ]
    );
    assert_pattern!(
        "cdlinvertedhammer_basic",
        cdl_inverted_hammer,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 11.0, 9.6
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 11.2, 10.8
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.6, 9.55
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.8, 9.8
        ]
    );
    assert_pattern!(
        "cdlshootingstar_basic",
        cdl_shooting_star,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.3
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 14.5
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 13.25
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.1
        ]
    );
    assert_pattern!(
        "cdltakuri_basic",
        cdl_takuri,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.03
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 10.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.01
        ]
    );
    assert_pattern!(
        "cdlcounterattack_basic",
        cdl_counter_attack,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.5
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 12.2
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.5, 9.3
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 10.0
        ]
    );
    assert_pattern!(
        "cdlhomingpigeon_basic",
        cdl_homing_pigeon,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.6
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 11.7
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.7, 11.0
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 11.1
        ]
    );
    assert_pattern!(
        "cdlinneck_basic",
        cdl_in_neck,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.4
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 10.2
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.5, 9.2
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 10.02
        ]
    );
    assert_pattern!(
        "cdlonneck_basic",
        cdl_on_neck,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.3
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 9.9
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.5, 9.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 9.5
        ]
    );
    assert_pattern!(
        "cdlthrusting_basic",
        cdl_thrusting,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.3
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 10.7
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.5, 9.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 10.6
        ]
    );
    assert_pattern!(
        "cdlmatchinglow_basic",
        cdl_matching_low,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.2
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 11.5
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.7, 10.4
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 10.0
        ]
    );
}

#[test]
fn multi_candle_candlestick_patterns_match_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    macro_rules! assert_pattern {
        ($oracle_name:literal, $end_idx:expr, $method:ident, $open:expr, $high:expr, $low:expr, $close:expr) => {{
            let oracle = run_oracle_int($oracle_name);
            let open = $open;
            let high = $high;
            let low = $low;
            let close = $close;
            let result = context.$method(
                0,
                $end_idx,
                &open,
                &high,
                &low,
                &close,
                &mut out_beg_idx,
                &mut out_nb_element,
                &mut out_integer,
            );
            assert_eq!(result as i32, oracle.ret_code, "{}", $oracle_name);
            assert_eq!(out_beg_idx, oracle.out_beg_idx, "{}", $oracle_name);
            assert_eq!(out_nb_element, oracle.out_nb_element, "{}", $oracle_name);
            assert_eq!(
                &out_integer[..out_nb_element],
                oracle.values.as_slice(),
                "{}",
                $oracle_name
            );
        }};
    }

    assert_pattern!(
        "cdl2crows_basic",
        11,
        cdl_2_crows,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.6
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 13.8
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 12.8
        ]
    );
    assert_pattern!(
        "cdl3blackcrows_basic",
        12,
        cdl_3_black_crows,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.6, 11.9, 11.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.0, 12.0, 11.1
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.7, 10.9, 10.0
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.8, 10.8, 10.0
        ]
    );

    let dark_cloud_oracle = run_oracle_int("cdldarkcloudcover_basic");
    let dark_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.4,
    ];
    let dark_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 14.5,
    ];
    let dark_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.4,
    ];
    let dark_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 13.0,
    ];
    let result = context.cdl_dark_cloud_cover(
        0,
        11,
        &dark_open,
        &dark_high,
        &dark_low,
        &dark_close,
        0.5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, dark_cloud_oracle.ret_code);
    assert_eq!(out_beg_idx, dark_cloud_oracle.out_beg_idx);
    assert_eq!(out_nb_element, dark_cloud_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        dark_cloud_oracle.values.as_slice()
    );

    assert_pattern!(
        "cdlpiercing_basic",
        11,
        cdl_piercing,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.2
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 13.0
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.8, 9.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 11.3
        ]
    );
    assert_pattern!(
        "cdlseparatinglines_basic",
        11,
        cdl_separating_lines,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.2, 14.1
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.0, 11.98
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.2, 14.0
        ]
    );
    assert_pattern!(
        "cdlsticksandwich_basic",
        12,
        cdl_stick_sandwich,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 10.1, 11.1
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 11.3, 11.2
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.9, 10.05, 9.9
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 11.2, 10.0
        ]
    );
    assert_pattern!(
        "cdl3inside_basic",
        12,
        cdl_3_inside,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 10.8, 10.7
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 11.3, 10.9
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.8, 10.7, 9.8
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 11.2, 9.9
        ]
    );
    assert_pattern!(
        "cdl3linestrike_basic",
        13,
        cdl_3_line_strike,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.3, 12.8, 14.2
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.1, 13.6, 14.1, 14.3
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.2, 12.7, 11.7
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.5, 14.0, 11.8
        ]
    );
    assert_pattern!(
        "cdl3outside_basic",
        12,
        cdl_3_outside,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.6, 12.8
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 12.9, 13.2
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.0, 9.4, 12.6
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.2, 12.7, 13.0
        ]
    );
}

#[test]
fn star_and_doji_candlestick_patterns_match_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    macro_rules! assert_pattern {
        ($oracle_name:literal, $end_idx:expr, $method:ident, $open:expr, $high:expr, $low:expr, $close:expr) => {{
            let oracle = run_oracle_int($oracle_name);
            let open = $open;
            let high = $high;
            let low = $low;
            let close = $close;
            let result = context.$method(
                0,
                $end_idx,
                &open,
                &high,
                &low,
                &close,
                &mut out_beg_idx,
                &mut out_nb_element,
                &mut out_integer,
            );
            assert_eq!(result as i32, oracle.ret_code, "{}", $oracle_name);
            assert_eq!(out_beg_idx, oracle.out_beg_idx, "{}", $oracle_name);
            assert_eq!(out_nb_element, oracle.out_nb_element, "{}", $oracle_name);
            assert_eq!(
                &out_integer[..out_nb_element],
                oracle.values.as_slice(),
                "{}",
                $oracle_name
            );
        }};
    }

    assert_pattern!(
        "cdlhangingman_basic",
        11,
        cdl_hanging_man,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.3
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 12.55
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.45
        ]
    );
    assert_pattern!(
        "cdldojistar_basic",
        11,
        cdl_doji_star,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.2
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.1, 14.35
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.15
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.22
        ]
    );

    let evening_oracle = run_oracle_int("cdleveningstar_basic");
    let evening_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.2, 13.9,
    ];
    let evening_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.1, 14.35, 13.95,
    ];
    let evening_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.15, 12.6,
    ];
    let evening_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.22, 12.8,
    ];
    let result = context.cdl_evening_star(
        0,
        12,
        &evening_open,
        &evening_high,
        &evening_low,
        &evening_close,
        0.3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, evening_oracle.ret_code);
    assert_eq!(out_beg_idx, evening_oracle.out_beg_idx);
    assert_eq!(out_nb_element, evening_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        evening_oracle.values.as_slice()
    );

    let evening_doji_oracle = run_oracle_int("cdleveningdojistar_basic");
    let evening_doji_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.1, 14.28, 13.95,
    ];
    let evening_doji_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.16, 12.6,
    ];
    let evening_doji_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.21, 12.8,
    ];
    let result = context.cdl_evening_doji_star(
        0,
        12,
        &evening_open,
        &evening_doji_high,
        &evening_doji_low,
        &evening_doji_close,
        0.3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, evening_doji_oracle.ret_code);
    assert_eq!(out_beg_idx, evening_doji_oracle.out_beg_idx);
    assert_eq!(out_nb_element, evening_doji_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        evening_doji_oracle.values.as_slice()
    );

    let morning_oracle = run_oracle_int("cdlmorningstar_basic");
    let morning_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 9.0, 9.2,
    ];
    let morning_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 9.1, 11.5,
    ];
    let morning_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.8, 8.95, 9.1,
    ];
    let morning_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 9.08, 11.4,
    ];
    let result = context.cdl_morning_star(
        0,
        12,
        &morning_open,
        &morning_high,
        &morning_low,
        &morning_close,
        0.3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, morning_oracle.ret_code);
    assert_eq!(out_beg_idx, morning_oracle.out_beg_idx);
    assert_eq!(out_nb_element, morning_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        morning_oracle.values.as_slice()
    );

    let morning_doji_oracle = run_oracle_int("cdlmorningdojistar_basic");
    let morning_doji_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 9.06, 11.5,
    ];
    let morning_doji_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.8, 8.96, 9.1,
    ];
    let morning_doji_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 9.01, 11.4,
    ];
    let result = context.cdl_morning_doji_star(
        0,
        12,
        &morning_open,
        &morning_doji_high,
        &morning_doji_low,
        &morning_doji_close,
        0.3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, morning_doji_oracle.ret_code);
    assert_eq!(out_beg_idx, morning_doji_oracle.out_beg_idx);
    assert_eq!(out_nb_element, morning_doji_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        morning_doji_oracle.values.as_slice()
    );

    assert_pattern!(
        "cdllongleggeddoji_basic",
        11,
        cdl_long_legged_doji,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 13.1
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 10.9
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 12.03
        ]
    );
    assert_pattern!(
        "cdlrickshawman_basic",
        11,
        cdl_rickshaw_man,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.95
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.4, 13.1
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 10.8
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 11.98
        ]
    );
}

#[test]
fn continuation_and_gap_candlestick_patterns_match_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    macro_rules! assert_pattern {
        ($oracle_name:literal, $end_idx:expr, $method:ident, $open:expr, $high:expr, $low:expr, $close:expr) => {{
            let oracle = run_oracle_int($oracle_name);
            let open = $open;
            let high = $high;
            let low = $low;
            let close = $close;
            let result = context.$method(
                0,
                $end_idx,
                &open,
                &high,
                &low,
                &close,
                &mut out_beg_idx,
                &mut out_nb_element,
                &mut out_integer,
            );
            assert_eq!(result as i32, oracle.ret_code, "{}", $oracle_name);
            assert_eq!(out_beg_idx, oracle.out_beg_idx, "{}", $oracle_name);
            assert_eq!(out_nb_element, oracle.out_nb_element, "{}", $oracle_name);
            assert_eq!(
                &out_integer[..out_nb_element],
                oracle.values.as_slice(),
                "{}",
                $oracle_name
            );
        }};
    }

    assert_pattern!(
        "cdltristar_basic",
        12,
        cdl_tristar,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.2, 14.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 14.3, 14.05
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.15, 13.95
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 12.02, 14.22, 13.98
        ]
    );
    assert_pattern!(
        "cdlunique3river_basic",
        12,
        cdl_unique_3_river,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 10.9, 9.4
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.1, 11.0, 9.9
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 9.5, 9.3, 9.35
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 10.0, 9.6, 9.8
        ]
    );
    assert_pattern!(
        "cdlbreakaway_basic",
        14,
        cdl_breakaway,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.4, 15.1, 15.9,
            14.6
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 15.0, 15.7, 16.4,
            14.7
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.3, 14.9, 15.5, 13.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.9, 15.5, 16.1,
            13.4
        ]
    );
    assert_pattern!(
        "cdltasukigap_basic",
        12,
        cdl_tasuki_gap,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.2, 13.8
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.1, 14.2, 13.9
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 13.1, 12.4
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 14.0, 12.6
        ]
    );
    assert_pattern!(
        "cdlupsidegap2crows_basic",
        12,
        cdl_upside_gap_2_crows,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.4, 14.9
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 15.0, 15.1
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.3, 13.3
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.6, 13.8
        ]
    );
    assert_pattern!(
        "cdlkicking_basic",
        11,
        cdl_kicking,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 13.0, 14.4
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.02, 15.1
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.38
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 12.0, 15.0
        ]
    );
    assert_pattern!(
        "cdlkickingbylength_basic",
        11,
        cdl_kicking_by_length,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 13.0, 14.5
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.02, 15.5
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.48
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 12.0, 15.4
        ]
    );
    assert_pattern!(
        "cdlrisefall3methods_basic",
        15,
        cdl_rise_fall_3_methods,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.2, 12.7, 12.3,
            12.0, 12.8
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 13.3, 12.8, 12.4,
            12.1, 14.5
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.4, 12.2, 11.9, 11.8,
            12.7
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 12.8, 12.4, 12.1,
            11.9, 14.4
        ]
    );

    let oracle = run_oracle_int("cdlmathold_basic");
    let open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.4, 14.2, 13.9, 13.6,
        14.0,
    ];
    let high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 14.3, 14.25, 13.95,
        14.05, 15.2,
    ];
    let low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 13.3, 13.8, 13.5, 13.2,
        13.95,
    ];
    let close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.0, 13.95, 13.7, 13.4,
        15.1,
    ];
    let result = context.cdl_mat_hold(
        0,
        15,
        &open,
        &high,
        &low,
        &close,
        0.5,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, oracle.ret_code);
    assert_eq!(out_beg_idx, oracle.out_beg_idx);
    assert_eq!(out_nb_element, oracle.out_nb_element);
    assert_eq!(&out_integer[..out_nb_element], oracle.values.as_slice());
}

#[test]
fn advancing_and_shoulder_candlestick_patterns_match_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    macro_rules! assert_pattern {
        ($oracle_name:literal, $end_idx:expr, $method:ident, $open:expr, $high:expr, $low:expr, $close:expr) => {{
            let oracle = run_oracle_int($oracle_name);
            let open = $open;
            let high = $high;
            let low = $low;
            let close = $close;
            let result = context.$method(
                0,
                $end_idx,
                &open,
                &high,
                &low,
                &close,
                &mut out_beg_idx,
                &mut out_nb_element,
                &mut out_integer,
            );
            assert_eq!(result as i32, oracle.ret_code, "{}", $oracle_name);
            assert_eq!(out_beg_idx, oracle.out_beg_idx, "{}", $oracle_name);
            assert_eq!(out_nb_element, oracle.out_nb_element, "{}", $oracle_name);
            assert_eq!(
                &out_integer[..out_nb_element],
                oracle.values.as_slice(),
                "{}",
                $oracle_name
            );
        }};
    }

    assert_pattern!(
        "cdlidentical3crows_basic",
        12,
        cdl_identical_3_crows,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.02, 10.04
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.05, 11.05, 10.05
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.9, 10.0, 9.0
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.0, 10.0, 9.0
        ]
    );
    assert_pattern!(
        "cdl3whitesoldiers_basic",
        12,
        cdl_3_white_soldiers,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.3, 12.8
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.02, 13.52, 14.02
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.28, 12.78
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.5, 14.0
        ]
    );
    assert_pattern!(
        "cdladvanceblock_basic",
        12,
        cdl_advance_block,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.0, 13.4
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.02, 13.9, 14.3
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.95, 13.35
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.6, 13.8
        ]
    );
    assert_pattern!(
        "cdlstalledpattern_basic",
        12,
        cdl_stalled_pattern,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.4, 13.15
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.3, 14.02, 13.62
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 12.35, 13.1
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.2, 14.0, 13.55
        ]
    );
    assert_pattern!(
        "cdlgapsidesidewhite_basic",
        12,
        cdl_gap_side_side_white,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.2, 13.19
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.1, 14.1, 14.0
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 13.15, 13.14
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.9, 13.85
        ]
    );
}

#[test]
fn final_candlestick_tail_matches_c_oracle() {
    let context = Context::new();
    let mut out_integer = [0i32; 64];
    let mut out_beg_idx = 0usize;
    let mut out_nb_element = 0usize;

    macro_rules! assert_pattern {
        ($oracle_name:literal, $end_idx:expr, $method:ident, $open:expr, $high:expr, $low:expr, $close:expr) => {{
            let oracle = run_oracle_int($oracle_name);
            let open = $open;
            let high = $high;
            let low = $low;
            let close = $close;
            let result = context.$method(
                0,
                $end_idx,
                &open,
                &high,
                &low,
                &close,
                &mut out_beg_idx,
                &mut out_nb_element,
                &mut out_integer,
            );
            assert_eq!(result as i32, oracle.ret_code, "{}", $oracle_name);
            assert_eq!(out_beg_idx, oracle.out_beg_idx, "{}", $oracle_name);
            assert_eq!(out_nb_element, oracle.out_nb_element, "{}", $oracle_name);
            assert_eq!(
                &out_integer[..out_nb_element],
                oracle.values.as_slice(),
                "{}",
                $oracle_name
            );
        }};
    }

    assert_pattern!(
        "cdl3starsinsouth_basic",
        12,
        cdl_3_stars_in_south,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.3, 10.95
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.05, 11.35, 10.9
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.3, 10.6, 10.7
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.0, 10.8, 10.75
        ]
    );

    let abandoned_oracle = run_oracle_int("cdlabandonedbaby_basic");
    let abandoned_open = [
        10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 14.2, 13.9,
    ];
    let abandoned_high = [
        11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 14.2, 14.25, 13.95,
    ];
    let abandoned_low = [
        9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 14.18, 12.7,
    ];
    let abandoned_close = [
        11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 14.0, 14.21, 12.8,
    ];
    let result = context.cdl_abandoned_baby(
        0,
        12,
        &abandoned_open,
        &abandoned_high,
        &abandoned_low,
        &abandoned_close,
        0.3,
        &mut out_beg_idx,
        &mut out_nb_element,
        &mut out_integer,
    );
    assert_eq!(result as i32, abandoned_oracle.ret_code);
    assert_eq!(out_beg_idx, abandoned_oracle.out_beg_idx);
    assert_eq!(out_nb_element, abandoned_oracle.out_nb_element);
    assert_eq!(
        &out_integer[..out_nb_element],
        abandoned_oracle.values.as_slice()
    );

    assert_pattern!(
        "cdlconcealbabyswall_basic",
        13,
        cdl_conceal_babys_wall,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.0, 10.2, 10.25
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.02, 11.02, 10.7, 10.9
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.9, 10.1, 9.8, 9.6
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.0, 10.2, 9.9, 9.7
        ]
    );
    assert_pattern!(
        "cdlhikkake_basic",
        8,
        cdl_hikkake,
        [10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6],
        [11.4, 11.6, 12.0, 11.7, 11.5, 11.2, 11.1, 11.05, 11.0],
        [9.6, 9.8, 10.0, 10.3, 10.5, 10.2, 10.1, 10.0, 9.9],
        [11.0, 11.2, 11.5, 11.1, 10.8, 10.4, 11.3, 10.2, 10.1]
    );
    assert_pattern!(
        "cdlhikkakemod_basic",
        9,
        cdl_hikkake_mod,
        [10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8],
        [11.4, 11.6, 12.4, 12.1, 11.8, 11.6, 11.3, 11.2, 11.15, 11.1],
        [9.6, 9.8, 10.0, 10.4, 10.6, 10.8, 10.2, 10.1, 10.0, 9.9],
        [11.0, 11.2, 10.2, 10.8, 11.0, 11.2, 10.3, 11.5, 10.2, 10.1]
    );
    assert_pattern!(
        "cdlladderbottom_basic",
        14,
        cdl_ladder_bottom,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 11.0, 10.4, 9.8, 9.9
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 12.05, 11.02, 10.42, 10.3,
            10.8
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 10.9, 10.0, 9.4, 8.9, 9.85
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 11.0, 10.0, 9.4, 9.0, 10.7
        ]
    );
    assert_pattern!(
        "cdlxsidegap3methods_basic",
        12,
        cdl_x_side_gap_3_methods,
        [
            10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 13.2, 14.0
        ],
        [
            11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 13.2, 13.1, 14.1, 14.05
        ],
        [
            9.6, 9.8, 10.0, 10.2, 10.4, 10.6, 10.8, 11.0, 11.2, 11.4, 11.9, 13.1, 12.5
        ],
        [
            11.0, 11.2, 11.4, 11.6, 11.8, 12.0, 12.2, 12.4, 12.6, 12.8, 13.0, 14.0, 12.6
        ]
    );
}

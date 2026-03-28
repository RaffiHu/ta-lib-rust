use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn find_file(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).ok()?;
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.file_name().and_then(|file| file.to_str()) == Some(name) {
                return Some(entry_path);
            }
        }
    }
    None
}

fn ensure_staticlib(workspace_root: &Path) -> PathBuf {
    let status = Command::new("cargo")
        .current_dir(workspace_root)
        .args(["build", "-p", "ta-lib-capi", "--lib"])
        .status()
        .expect("build ta-lib-capi staticlib");
    assert!(status.success(), "cargo build -p ta-lib-capi --lib failed");

    find_file(&workspace_root.join("target/debug"), "libta_lib_capi.a")
        .expect("locate built libta_lib_capi.a")
}

#[test]
fn upstream_ta_regtest_builds_and_passes_against_rust_capi() {
    let workspace_root = workspace_root();
    let upstream = workspace_root.join("upstream-ta-lib-c");
    let staticlib = ensure_staticlib(&workspace_root);
    let tempdir = tempfile::tempdir().expect("create ta_regtest tempdir");
    let binary = tempdir.path().join("ta_regtest");

    let mut command = Command::new("cc");
    command.current_dir(&workspace_root);
    command.arg("-std=c99");
    command.arg("-O0");
    command.arg("-I").arg(upstream.join("include"));
    command.arg("-I").arg(upstream.join("src/ta_common"));
    command.arg("-I").arg(upstream.join("src/ta_func"));
    command.arg("-I").arg(upstream.join("src/tools/ta_regtest"));

    for relative in [
        "src/tools/ta_regtest/ta_regtest.c",
        "src/tools/ta_regtest/test_data.c",
        "src/tools/ta_regtest/test_util.c",
        "src/tools/ta_regtest/test_abstract.c",
        "src/tools/ta_regtest/test_internals.c",
        "src/tools/ta_regtest/ta_test_func/test_adx.c",
        "src/tools/ta_regtest/ta_test_func/test_mom.c",
        "src/tools/ta_regtest/ta_test_func/test_sar.c",
        "src/tools/ta_regtest/ta_test_func/test_rsi.c",
        "src/tools/ta_regtest/ta_test_func/test_candlestick.c",
        "src/tools/ta_regtest/ta_test_func/test_per_ema.c",
        "src/tools/ta_regtest/ta_test_func/test_per_hlc.c",
        "src/tools/ta_regtest/ta_test_func/test_stoch.c",
        "src/tools/ta_regtest/ta_test_func/test_macd.c",
        "src/tools/ta_regtest/ta_test_func/test_minmax.c",
        "src/tools/ta_regtest/ta_test_func/test_per_hlcv.c",
        "src/tools/ta_regtest/ta_test_func/test_1in_1out.c",
        "src/tools/ta_regtest/ta_test_func/test_1in_2out.c",
        "src/tools/ta_regtest/ta_test_func/test_per_ohlc.c",
        "src/tools/ta_regtest/ta_test_func/test_stddev.c",
        "src/tools/ta_regtest/ta_test_func/test_bbands.c",
        "src/tools/ta_regtest/ta_test_func/test_ma.c",
        "src/tools/ta_regtest/ta_test_func/test_po.c",
        "src/tools/ta_regtest/ta_test_func/test_per_hl.c",
        "src/tools/ta_regtest/ta_test_func/test_trange.c",
        "src/tools/ta_regtest/ta_test_func/test_imi.c",
        "src/tools/ta_regtest/ta_test_func/test_avgdev.c",
    ] {
        command.arg(upstream.join(relative));
    }

    command.arg(staticlib);
    command.arg("-lm");
    command.arg("-o");
    command.arg(&binary);

    let output = command.output().expect("compile upstream ta_regtest");
    assert!(
        output.status.success(),
        "ta_regtest compile failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let output = Command::new(&binary)
        .current_dir(tempdir.path())
        .output()
        .expect("run upstream ta_regtest");
    assert!(
        output.status.success(),
        "ta_regtest failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

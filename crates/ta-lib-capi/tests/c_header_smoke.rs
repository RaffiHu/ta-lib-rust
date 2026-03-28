use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
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
                return Some(entry.path());
            }
        }
    }

    None
}

#[test]
fn c_can_include_upstream_headers_and_link_to_rust_staticlib() {
    let workspace_root = workspace_root();
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target"));
    let staticlib = if let Some(path) = find_file(&target_dir.join("debug"), "libta_lib_capi.a") {
        path
    } else {
        let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
            .arg("build")
            .arg("-p")
            .arg("ta-lib-capi")
            .arg("--lib")
            .status()
            .expect("build ta-lib-capi staticlib");
        assert!(status.success(), "cargo build for staticlib failed with {status}");
        find_file(&target_dir.join("debug"), "libta_lib_capi.a").expect("find staticlib")
    };
    assert!(
        staticlib.exists(),
        "expected static library at {}",
        staticlib.display()
    );

    let temp_dir = target_dir.join("capi-smoke");
    fs::create_dir_all(&temp_dir).expect("create smoke dir");
    let c_file = temp_dir.join("smoke.c");
    let bin_file = temp_dir.join("smoke");

    fs::write(
        &c_file,
        r#"#include <stddef.h>
#include "ta_libc.h"

int main(void) {
    double lhs[3] = {1.0, 2.0, 3.0};
    double rhs[3] = {4.0, 5.0, 6.0};
    double out[3] = {0.0, 0.0, 0.0};
    int outBegIdx = -1;
    int outNbElement = -1;

    if (TA_Initialize() != TA_SUCCESS) {
        return 1;
    }

    if (TA_ADD(0, 2, lhs, rhs, &outBegIdx, &outNbElement, out) != TA_SUCCESS) {
        return 2;
    }

    if (outBegIdx != 0 || outNbElement != 3) {
        return 3;
    }

    if (out[0] != 5.0 || out[1] != 7.0 || out[2] != 9.0) {
        return 4;
    }

    if (TA_Shutdown() != TA_SUCCESS) {
        return 5;
    }

    return 0;
}
"#,
    )
    .expect("write smoke source");

    let include_dir = workspace_root.join("upstream-ta-lib-c/include");
    let status = Command::new("cc")
        .arg("-std=c99")
        .arg("-I")
        .arg(&include_dir)
        .arg(&c_file)
        .arg(&staticlib)
        .arg("-ldl")
        .arg("-lpthread")
        .arg("-lm")
        .arg("-o")
        .arg(&bin_file)
        .status()
        .expect("compile c smoke");
    assert!(status.success(), "c smoke compile failed with {status}");

    let status = Command::new(&bin_file)
        .status()
        .expect("run c smoke binary");
    assert!(status.success(), "c smoke binary failed with {status}");
}

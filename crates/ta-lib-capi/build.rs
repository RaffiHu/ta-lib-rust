use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root");
    let xml_path = workspace_root.join("upstream-ta-lib-c/ta_func_api.xml");
    let xml = fs::read_to_string(&xml_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", xml_path.display()));
    let functions = ta_lib_codegen::parse_functions(&xml).expect("parse upstream function xml");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", xml_path.display());

    fs::write(
        out_dir.join("generated_c_api_wrappers.rs"),
        ta_lib_codegen::render_generated_c_api_wrappers(&functions),
    )
    .expect("write generated c api wrappers");
    fs::write(
        out_dir.join("generated_abstract_api.rs"),
        ta_lib_codegen::render_generated_abstract_api(&functions),
    )
    .expect("write generated abstract api");
}

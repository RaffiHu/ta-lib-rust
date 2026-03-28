use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let mut manifest_root = PathBuf::from(".");
    let mut emit_rust_index = false;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest-root" => {
                if let Some(value) = args.next() {
                    manifest_root = PathBuf::from(value);
                }
            }
            "--emit-rust-index" => emit_rust_index = true,
            _ => {}
        }
    }

    let upstream_xml = manifest_root.join("upstream-ta-lib-c/ta_func_api.xml");
    let xml = fs::read_to_string(&upstream_xml)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", upstream_xml.display()));
    let functions = ta_lib_codegen::parse_functions(&xml).expect("failed to parse ta_func_api.xml");

    if emit_rust_index {
        print!("{}", ta_lib_codegen::emit_rust_index_table(&functions));
        return;
    }

    println!("parsed {} functions", functions.len());
    for (group, count) in ta_lib_codegen::group_counts(&functions) {
        println!("{group}: {count}");
    }
}

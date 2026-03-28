use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use ta_lib::generated::FUNCTIONS;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn parse_function_abbreviations(xml: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    let open = "<Abbreviation>";
    let close = "</Abbreviation>";
    let mut remaining = xml;

    while let Some(start) = remaining.find(open) {
        let after_open = &remaining[start + open.len()..];
        let Some(end) = after_open.find(close) else {
            break;
        };
        set.insert(after_open[..end].trim().to_string());
        remaining = &after_open[end + close.len()..];
    }

    set
}

fn extract_called_ta_functions(c_source: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    let bytes = c_source.as_bytes();
    let mut index = 0usize;

    while let Some(relative) = c_source[index..].find("TA_") {
        let start = index + relative + 3;
        let mut end = start;
        while end < bytes.len()
            && (bytes[end].is_ascii_uppercase()
                || bytes[end].is_ascii_digit()
                || bytes[end] == b'_')
        {
            end += 1;
        }
        if end < bytes.len() && bytes[end] == b'(' {
            set.insert(c_source[start..end].to_string());
        }
        index = end;
    }

    set
}

#[test]
fn generated_metadata_covers_all_upstream_functions() {
    let xml = fs::read_to_string(workspace_root().join("upstream-ta-lib-c/ta_func_api.xml"))
        .expect("read upstream xml");
    let upstream = parse_function_abbreviations(&xml);
    let generated: BTreeSet<_> = FUNCTIONS
        .iter()
        .map(|function| function.abbreviation.to_string())
        .collect();

    assert_eq!(generated.len(), 161);
    assert_eq!(generated, upstream);
}

#[test]
fn generated_metadata_has_unique_public_names() {
    let rust_names: Vec<_> = FUNCTIONS
        .iter()
        .map(|function| function.rust_name)
        .collect();
    let unique: BTreeSet<_> = rust_names.iter().copied().collect();
    assert_eq!(
        rust_names.len(),
        unique.len(),
        "duplicate rust_name entries detected"
    );
}

#[test]
fn oracle_inventory_calls_every_upstream_function() {
    let xml = fs::read_to_string(workspace_root().join("upstream-ta-lib-c/ta_func_api.xml"))
        .expect("read upstream xml");
    let upstream = parse_function_abbreviations(&xml);
    let oracle_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/c_oracle_seed.c"))
            .expect("read oracle source");
    let called = extract_called_ta_functions(&oracle_source);

    let missing: Vec<_> = upstream.difference(&called).cloned().collect();
    assert!(
        missing.is_empty(),
        "oracle inventory missing functions: {missing:?}"
    );
}

#[test]
fn compatibility_facade_declares_every_generated_function() {
    let compat_source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/compat.rs"))
            .expect("read compat.rs");

    let missing: Vec<_> = FUNCTIONS
        .iter()
        .filter(|function| {
            let has_explicit = compat_source
                .contains(&format!("pub fn {}_lookback", function.rust_name))
                && compat_source.contains(&format!("pub fn {}(", function.rust_name));
            let has_cdl_macro = compat_source.contains(&format!(
                "compat_cdl_noopt!({}_lookback, {});",
                function.rust_name, function.rust_name
            ));

            !(has_explicit || has_cdl_macro)
        })
        .map(|function| function.abbreviation)
        .collect();

    assert!(
        missing.is_empty(),
        "compatibility facade missing functions: {missing:?}"
    );
}

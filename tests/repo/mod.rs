mod progress;

use self::progress::Progress;
use crate::common;
use anyhow::Result;
use flate2::read::GzDecoder;
use std::fs;
use std::path::Path;
use tar::Archive;
use walkdir::DirEntry;

const REVISION: &str = "5e8897b7b51636f157630e6639b711d698e1d101";

pub fn base_dir_filter(entry: &DirEntry) -> bool {
    let path = entry.path();
    if path.is_dir() {
        return true; // otherwise walkdir does not visit the files
    }
    if path.extension().map(|e| e != "rs").unwrap_or(true) {
        return false;
    }
    let path_string = path.to_string_lossy();
    let path_string = if cfg!(windows) {
        path_string.replace('\\', "/").into()
    } else {
        path_string
    };
    // TODO assert that parsing fails on the parse-fail cases
    if path_string.starts_with("tests/rust/src/test/parse-fail")
        || path_string.starts_with("tests/rust/src/test/compile-fail")
        || path_string.starts_with("tests/rust/src/test/rustfix")
    {
        return false;
    }

    if path_string.starts_with("tests/rust/src/test/ui") {
        let stderr_path = path.with_extension("stderr");
        if stderr_path.exists() {
            // Expected to fail in some way
            return false;
        }
    }

    match path_string.as_ref() {
        // Deprecated placement syntax
        "tests/rust/src/test/ui/obsolete-in-place/bad.rs" |
        // Deprecated anonymous parameter syntax in traits
        "tests/rust/src/test/ui/error-codes/e0119/auxiliary/issue-23563-a.rs" |
        "tests/rust/src/test/ui/issues/issue-13105.rs" |
        "tests/rust/src/test/ui/issues/issue-13775.rs" |
        "tests/rust/src/test/ui/issues/issue-34074.rs" |
        // 2015-style dyn that libsyntax rejects
        "tests/rust/src/test/ui/dyn-keyword/dyn-2015-no-warnings-without-lints.rs" |
        // TODO: visibility on enum variants
        "tests/rust/src/test/pretty/enum-variant-vis.rs" |
        "tests/rust/src/test/ui/parser/issue-65041-empty-vis-matcher-in-enum.rs" |
        // TODO: &raw address-of
        "tests/rust/src/test/pretty/raw-address-of.rs" |
        "tests/rust/src/test/ui/borrowck/borrow-raw-address-of-deref-mutability-ok.rs" |
        "tests/rust/src/test/ui/borrowck/borrow-raw-address-of-mutability-ok.rs" |
        "tests/rust/src/test/ui/consts/const-address-of.rs" |
        "tests/rust/src/test/ui/consts/const-mut-refs/const_mut_address_of.rs" |
        "tests/rust/src/test/ui/consts/min_const_fn/address_of_const.rs" |
        "tests/rust/src/test/ui/packed/packed-struct-address-of-element.rs" |
        "tests/rust/src/test/ui/raw-ref-op/raw-ref-op.rs" |
        "tests/rust/src/test/ui/raw-ref-op/raw-ref-temp-deref.rs" |
        "tests/rust/src/test/ui/raw-ref-op/unusual_locations.rs" |
        // TODO: half open range patterns
        "tests/rust/src/test/ui/half-open-range-patterns/half-open-range-pats-syntactic-pass.rs" |
        "tests/rust/src/test/ui/half-open-range-patterns/pat-tuple-4.rs" |
        // TODO: inherent associated const
        "tests/rust/src/test/ui/parser/impl-item-const-pass.rs" |
        // TODO: inherent associated type
        "tests/rust/src/test/ui/parser/impl-item-type-no-body-pass.rs" |
        // TODO: visibility on trait items
        "tests/rust/src/test/ui/parser/issue-65041-empty-vis-matcher-in-trait.rs" |
        // TODO: default const
        "tests/rust/src/test/ui/parser/trait-item-with-defaultness-pass.rs" |
        // TODO: variadic ellipses before the last function argument
        "tests/rust/src/test/ui/parser/variadic-ffi-syntactic-pass.rs" |
        // TODO: const trait impls and bounds
        "tests/rust/src/test/ui/rfc-2632-const-trait-impl/const-trait-bound-opt-out/feature-gate.rs" |
        "tests/rust/src/test/ui/rfc-2632-const-trait-impl/const-trait-bound-opt-out/syntax.rs" |
        "tests/rust/src/test/ui/rfc-2632-const-trait-impl/feature-gate.rs" |
        "tests/rust/src/test/ui/rfc-2632-const-trait-impl/syntax.rs" |
        // not actually test cases
        "tests/rust/src/test/rustdoc-ui/test-compile-fail2.rs" |
        "tests/rust/src/test/rustdoc-ui/test-compile-fail3.rs" |
        "tests/rust/src/test/ui/include-single-expr-helper.rs" |
        "tests/rust/src/test/ui/include-single-expr-helper-1.rs" |
        "tests/rust/src/test/ui/issues/auxiliary/issue-21146-inc.rs" |
        "tests/rust/src/test/ui/macros/auxiliary/macro-comma-support.rs" |
        "tests/rust/src/test/ui/macros/auxiliary/macro-include-items-expr.rs" => false,
        _ => true,
    }
}

pub fn clone_rust() {
    let needs_clone = match fs::read_to_string("tests/rust/COMMIT") {
        Err(_) => true,
        Ok(contents) => contents.trim() != REVISION,
    };
    if needs_clone {
        download_and_unpack().unwrap();
    }
}

fn download_and_unpack() -> Result<()> {
    let url = format!(
        "https://github.com/rust-lang/rust/archive/{}.tar.gz",
        REVISION
    );
    let response = reqwest::blocking::get(&url)?.error_for_status()?;
    let progress = Progress::new(response);
    let decoder = GzDecoder::new(progress);
    let mut archive = Archive::new(decoder);
    let prefix = format!("rust-{}", REVISION);

    let tests_rust = Path::new("tests/rust");
    if tests_rust.exists() {
        fs::remove_dir_all(tests_rust)?;
    }

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path == Path::new("pax_global_header") {
            continue;
        }
        let relative = path.strip_prefix(&prefix)?;
        let out = tests_rust.join(relative);
        entry.unpack(&out)?;
        if common::travis_ci() {
            // Something about this makes the travis build not deadlock...
            errorf!(".");
        }
    }

    fs::write("tests/rust/COMMIT", REVISION)?;
    Ok(())
}

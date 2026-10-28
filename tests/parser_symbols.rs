use std::ffi::c_void;
use std::path::Path;

// 模拟宿主程序携带的另一版 parser；库不能链接到宿主的 scanner。
#[no_mangle]
pub extern "C" fn tree_sitter_kotlin_external_scanner_create() -> *mut c_void {
    panic!("difftastic used the host Kotlin scanner")
}

#[no_mangle]
pub extern "C" fn tree_sitter_latex_external_scanner_create() -> *mut c_void {
    panic!("difftastic used the host LaTeX scanner")
}

#[test]
fn vendored_parsers_coexist_with_host_scanners() {
    let engine = difftastic::JsonDiff::from_env().unwrap();
    for (name, lhs, rhs) in [
        (
            "example.kt",
            "fun main() { println(1) }",
            "fun main() { println(2) }",
        ),
        ("example.tex", "\\section{old}", "\\section{new}"),
    ] {
        let result = engine
            .diff_slices(
                name,
                Path::new(name),
                Path::new(name),
                lhs.as_bytes(),
                rhs.as_bytes(),
            )
            .unwrap();
        assert!(!result.is_empty());
    }
}

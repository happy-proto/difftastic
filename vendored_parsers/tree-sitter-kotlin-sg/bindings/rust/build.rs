fn main() {
    let src_dir = std::path::Path::new("src");

    let mut c_config = cc::Build::new();
    c_config.std("c11").include(src_dir);

    // 与宿主的另一版 Kotlin parser 隔离，保持生成的 parser 与 scanner 配套。
    for suffix in [
        "",
        "_external_scanner_create",
        "_external_scanner_destroy",
        "_external_scanner_reset",
        "_external_scanner_scan",
        "_external_scanner_serialize",
        "_external_scanner_deserialize",
    ] {
        let symbol = format!("tree_sitter_kotlin{suffix}");
        let namespaced = format!("difftastic_{symbol}");
        c_config.define(&symbol, namespaced.as_str());
    }

    #[cfg(target_env = "msvc")]
    c_config.flag("-utf-8");

    let parser_path = src_dir.join("parser.c");
    c_config.file(&parser_path);
    println!("cargo:rerun-if-changed={}", parser_path.to_str().unwrap());

    let scanner_path = src_dir.join("scanner.c");
    c_config.file(&scanner_path);
    println!("cargo:rerun-if-changed={}", scanner_path.to_str().unwrap());

    c_config.compile("tree-sitter-kotlin");
}

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn library_json_matches_cli() {
    let mut engine = difftastic::JsonDiff::from_env().unwrap();
    for (display, lhs, rhs) in [
        (
            "sample.js",
            "sample_files/simple_1.js",
            "sample_files/simple_2.js",
        ),
        (
            "sample.txt",
            "sample_files/simple_1.txt",
            "sample_files/simple_2.txt",
        ),
        (
            "same.js",
            "sample_files/simple_1.js",
            "sample_files/simple_1.js",
        ),
    ] {
        engine.set_binary_attributes(std::collections::HashMap::from([(
            display.to_owned(),
            false,
        )]));
        let library = engine
            .diff_slices(
                display,
                std::path::Path::new(lhs),
                std::path::Path::new(rhs),
                &std::fs::read(lhs).unwrap(),
                &std::fs::read(rhs).unwrap(),
            )
            .unwrap();
        let output = get_base_command()
            .env("DFT_UNSTABLE", "yes")
            .env("DFT_EXIT_CODE", "false")
            .env("DFT_CHECK_ONLY", "false")
            .args([
                "--display=json",
                "--color=never",
                "--",
                display,
                lhs,
                "0",
                "100644",
                rhs,
                "0",
                "100644",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&library).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
            "{display}"
        );
    }
}

#[test]
fn library_invalid_config_returns_errors_without_exiting() {
    const CHILD: &str = "DIFFTASTIC_TEST_CONFIG_ERROR_CHILD";
    if std::env::var_os(CHILD).is_some() {
        assert!(difftastic::JsonDiff::from_env().is_err());
        return;
    }
    // 使用独立子进程验证，避免原 CLI 的 exit 路径终止整个测试进程。
    for (name, value) in [
        ("DFT_OVERRIDE_1", "missing-colon"),
        ("DFT_OVERRIDE_1", "*.rs:no-such-language"),
        ("DFT_OVERRIDE_1", "[:Rust"),
        ("DFT_OVERRIDE_BINARY_1", "["),
    ] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "library_invalid_config_returns_errors_without_exiting",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .env(name, value)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}={value}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn library_bytes_use_metadata_paths_and_replace_attributes() {
    let path = std::path::Path::new("library-test-does-not-create/example.rs");
    assert!(!path.exists());
    let mut engine = difftastic::JsonDiff::from_env().unwrap();
    let compare = |engine: &difftastic::JsonDiff| -> serde_json::Value {
        serde_json::from_slice(
            &engine
                .diff_slices(
                    "example.rs",
                    path,
                    path,
                    b"fn value() -> i32 { 1 }\n",
                    b"fn value() -> i32 { 2 }\n",
                )
                .unwrap(),
        )
        .unwrap()
    };
    engine.set_binary_attributes(std::collections::HashMap::from([(
        "example.rs".to_owned(),
        true,
    )]));
    assert_eq!(compare(&engine)["language"], "Binary");
    engine.set_binary_attributes(std::collections::HashMap::from([(
        "example.rs".to_owned(),
        false,
    )]));
    let expected = compare(&engine);
    assert_eq!(expected["language"], "Rust");
    assert_eq!(expected["status"], "changed");
    assert!(!path.exists());
    // 同一个只读配置可以从多个线程调用，不共享逐文件解析状态。
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| assert_eq!(compare(&engine), expected));
        }
    });
}

#[test]
fn library_encodings_and_empty_sides_match_cli() {
    let cases: &[(&str, &[u8], &[u8])] = &[
        (
            "sample.rs",
            b"fn main() { let x = 1; }\n",
            b"fn main() { let x = 2; }\n",
        ),
        ("sample.json", b"{\"x\":1}\n", b"{\"x\":2,\"y\":true}\n"),
        ("sample.kt", b"fun value() = 1\n", b"fun value() = 2\n"),
        ("sample.fs", b"let value = 1\n", b"let value = 2\n"),
        (
            "sample.smali",
            b".class public Old\n",
            b".class public New\n",
        ),
        (
            "sample.sv",
            b"module old; endmodule\n",
            b"module new; endmodule\n",
        ),
        ("plain.txt", b"before", b"after"),
        ("added.rs", b"", b"fn added() {}\n"),
        ("removed.rs", b"fn removed() {}\n", b""),
        ("same.rs", b"fn same() {}\n", b"fn same() {}\n"),
        ("binary.bin", b"\0\x01\x02", b"\0\x02\x03"),
        (
            "latin.txt",
            b"\xe9\xe9\xe9 before\n",
            b"\xe9\xe9\xe9 after\n",
        ),
        ("utf16.txt", b"\xff\xfea\0\n\0", b"\xff\xfeb\0\n\0"),
        ("invalid.rs", b"fn { broken\n", b"fn { changed\n"),
    ];
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    let mut engine = difftastic::JsonDiff::from_env().unwrap();
    for &(display, lhs, rhs) in cases {
        std::fs::write(&old, lhs).unwrap();
        std::fs::write(&new, rhs).unwrap();
        engine.set_binary_attributes(std::collections::HashMap::from([(
            display.to_owned(),
            false,
        )]));
        let bytes = engine.diff_slices(display, &old, &new, lhs, rhs).unwrap();
        let files = engine.diff_files(display, &old, &new).unwrap();
        assert_eq!(bytes, files);
        let output = get_base_command()
            .env("DFT_UNSTABLE", "yes")
            .env("DFT_EXIT_CODE", "false")
            .env("DFT_CHECK_ONLY", "false")
            .args(["--display=json", "--color=never", "--", display])
            .arg(&old)
            .args(["0", "100644"])
            .arg(&new)
            .args(["0", "100644"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
            "{display}"
        );
    }
}

fn find_runner() -> Option<String> {
    for (key, value) in std::env::vars() {
        if key.starts_with("CARGO_TARGET_") && key.ends_with("_RUNNER") && !value.is_empty() {
            return Some(value);
        }
    }
    None
}

// Sample code from
// https://github.com/assert-rs/assert_cmd/issues/139, supports
// cross-compiled binaries.
fn get_base_command() -> Command {
    let mut cmd;
    let path = assert_cmd::cargo_bin!("difft");
    if let Some(runner) = find_runner() {
        let mut runner = runner.split_whitespace();
        cmd = Command::new(runner.next().unwrap());
        for arg in runner {
            cmd.arg(arg);
        }
        cmd.arg(path);
    } else {
        cmd = Command::new(path);
    }
    cmd
}

#[test]
fn no_such_files() {
    let mut cmd = get_base_command();

    cmd.arg("no_such_file").arg("no_such_file_either");
    cmd.assert().failure().code(2);
}

#[test]
fn inline() {
    let mut cmd = get_base_command();

    cmd.arg("--display=inline")
        .arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");
    cmd.assert().success();
}

#[test]
fn inline_unchanged_lines_between_changes() {
    let mut cmd = get_base_command();

    cmd.arg("--display=inline")
        .arg("sample_files/cli_tests/gitconfig_1.gitconfig")
        .arg("sample_files/cli_tests/gitconfig_2.gitconfig");

    let url_rx = r#"(?m)^\s*7\s+\[url "ssh://git@github\.com"\]$"#;
    let url_lines = predicate::str::is_match(url_rx).unwrap().count(2);

    let blank_rx = r"(?m)^\s*6\s*$";
    let blank_lines = predicate::str::is_match(blank_rx).unwrap().count(2);

    cmd.assert().stdout(url_lines.and(blank_lines));
}

#[test]
fn binary_changed() {
    let mut cmd = get_base_command();

    cmd.arg("--display=inline")
        .arg("img/logo.png")
        .arg("/dev/null");

    let predicate_fn = predicate::str::contains("Binary file removed");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn binary_override() {
    let mut cmd = get_base_command();

    cmd.arg("--override-binary=*.js")
        .arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");
    let predicate_fn = predicate::str::contains("Binary file modified");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn has_changes_default_exit_code() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");
    cmd.assert().success();
}

#[test]
fn has_changes_requested_exit_code() {
    let mut cmd = get_base_command();

    cmd.arg("--exit-code")
        .arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");
    cmd.assert().failure().code(1);
}

#[test]
fn ignore_comments() {
    let mut cmd = get_base_command();

    cmd.arg("--ignore-comments")
        .arg("sample_files/comma_and_comment_1.js")
        .arg("sample_files/comma_and_comment_2.js");

    let predicate_fn = predicate::str::contains("No syntactic changes");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn check_only() {
    let mut cmd = get_base_command();

    cmd.arg("--check-only")
        .arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");

    let predicate_fn = predicate::str::contains("Has syntactic changes");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn check_only_text_file() {
    let mut cmd = get_base_command();

    cmd.arg("--check-only")
        .arg("sample_files/text_1.txt")
        .arg("sample_files/text_2.txt");

    let predicate_fn = predicate::str::contains("Has changes");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn text_changes_at_end_doesnt_crash() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/cli_tests/changes_at_end_1.txt")
        .arg("sample_files/cli_tests/changes_at_end_2.txt");

    cmd.assert().success();
}

/// Regression test for https://github.com/Wilfred/difftastic/issues/995.
#[test]
fn out_of_order_hunk_doesnt_crash() {
    let mut cmd = get_base_command();

    cmd.arg("--color=never")
        .arg("sample_files/cli_tests/out_of_order_hunk_1.py")
        .arg("sample_files/cli_tests/out_of_order_hunk_2.py");

    let predicate_fn = predicate::str::contains("preset_services={");
    cmd.assert().success().stdout(predicate_fn);
}

#[test]
fn makefile_text_as_atom() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/cli_tests/makefile_1.mk")
        .arg("sample_files/cli_tests/makefile_2.mk");

    let predicate_fn = predicate::str::contains("CCFLAGS");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn yaml_parse_errors() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/cli_tests/bad_yaml_1.yml")
        .arg("sample_files/cli_tests/bad_yaml_2.yml");

    let predicate_fn = predicate::str::contains("exceeded DFT_PARSE_ERROR_LIMIT");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn list_languages() {
    let mut cmd = get_base_command();

    cmd.arg("--list-languages");

    let predicate_fn = predicate::str::contains("TOML");
    cmd.assert().stdout(predicate_fn);

    let predicate_fn = predicate::str::contains("*.toml");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn test_mime_type_false_positive() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/cli_tests/x_mod_false_positive.js")
        .arg("sample_files/cli_tests/empty.js");

    let predicate_fn = predicate::str::contains("JavaScript");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn slightly_invalid_utf8() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/cli_tests/MainWindowViewModel.cs")
        .arg("/dev/null");

    let predicate_fn = predicate::str::contains("using");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn directory_arguments() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/dir_1").arg("sample_files/dir_2");

    let predicate_fn =
        predicate::str::contains("only_in_1.c").and(predicate::str::contains("only_in_2.rs"));
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn git_style_arguments_rename() {
    let mut cmd = get_base_command();

    cmd.arg("elisp_oldname.el")
        .arg("sample_files/elisp_1.el")
        .arg("lhs_hash_placeholder")
        .arg("lhs_mode_placeholder")
        .arg("sample_files/elisp_2.el")
        .arg("rhs_hash_placeholder")
        .arg("rhs_mode_placeholder")
        .arg("elisp_newname.el")
        .arg("similarity_placeholder");
    let predicate_fn = predicate::str::contains("Renamed");
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn git_style_arguments_new_file() {
    let mut cmd = get_base_command();

    cmd.arg("simple.txt")
        .arg("/dev/null")
        .arg(".")
        .arg(".")
        .arg("sample_files/simple_1.txt")
        .arg("abcdef1234")
        .arg("100644");
    let predicate_fn = predicate::str::contains("File permissions changed").not();
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn drop_different_path_starts() {
    let mut cmd = get_base_command();

    cmd.arg("sample_files/dir_1/clojure.clj")
        .arg("sample_files/dir_2/clojure.clj");
    let predicate_fn = predicate::str::contains("dir_2/clojure.clj").not();
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn dump_tree_sitter() {
    let mut cmd = get_base_command();

    cmd.arg("--dump-ts")
        .arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");
    cmd.assert().success();
}

#[test]
fn dump_syntax() {
    let mut cmd = get_base_command();

    cmd.arg("--dump-syntax")
        .arg("sample_files/simple_1.js")
        .arg("sample_files/simple_2.js");
    cmd.assert().success();
}

#[test]
fn walk_hidden_items() {
    let mut cmd = get_base_command();

    cmd.args(["sample_files/hidden_1", "sample_files/hidden_2"]);

    let predicate_fn =
        predicate::str::contains(format!(".hidden{}doc.txt", std::path::MAIN_SEPARATOR))
            .and(predicate::str::contains(".hidden.txt"))
            .and(predicate::str::contains("before"))
            .and(predicate::str::contains("after"));
    cmd.assert().stdout(predicate_fn);
}

#[test]
fn git_unmerged_files() {
    let mut cmd = get_base_command();

    cmd.args(["sample_files/simple_1.js"]);
    cmd.env("GIT_EXEC_PATH", "/usr/lib/git-core");

    let predicate_fn = predicate::str::contains("Unmerged path");
    cmd.assert().stdout(predicate_fn);
}

use std::path::Path;
use std::process::Command;

/// The result of checking file's `diff` and `binary` attributes with `git check-attr`.
/// See man gitattributes, specifically **Generating diff text** section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DiffAttribute {
    /// The file type (text vs binary) will be autodetected.
    ///
    /// This is the default setting, when `diff` attribute is not present.
    Unspecified,

    /// The file should be treated as text.
    ///
    /// Corresponds to set `diff` attribute, unless `binary` attribute is set.
    AssumeText,

    /// The file should be treated as binary, and no diff will be shown.
    /// Sometimes this option is set for generated text files.
    ///
    /// Corresponds to explicitly unset `diff` attribute (`-diff`), or when `binary` attribute is
    /// set.
    AssumeBinary,
}

/// Runs `git check-attr diff binary` to get the diff and binary attributes of the path. Returns
/// [`Option::None`] when either `git` is not available, file is not inside git directory, or
/// something else went wrong.
pub(crate) fn check_diff_attr(path: &Path) -> Option<DiffAttribute> {
    let res = Command::new("git")
        .args(["check-attr", "diff", "binary", "-z", "--"])
        .arg(path)
        .output();

    match res {
        Ok(output) => {
            // Running git outside of git repository, or perhaps many other error conditions, will
            // result in this. Since we run git check-attr eagerly, and can't distinguish the error
            // conditions easily and reliably, log just a debug message.
            if !output.status.success() {
                debug!(
                    "git check-attr exited with status {}: \"{}\"",
                    output.status,
                    output.stderr.escape_ascii()
                );
                return None;
            }

            let output = &output.stdout;
            debug!(
                "git check-attr output: {:?}",
                String::from_utf8_lossy(output)
            );

            match parse_output(output) {
                Some(res) => {
                    trace!("git check-attr result: {:?}", res);
                    return Some(res);
                }
                None => {
                    warn!(
                        "malformed git check-attr output: \"{}\"",
                        output.escape_ascii()
                    );
                }
            }
        }
        Err(err) => {
            debug!("failed to execute git: {err}");
        }
    }

    None
}

fn parse_output(output: &[u8]) -> Option<DiffAttribute> {
    // The git check-attr -z output format is repeated
    //     <path> NUL <attribute> NUL <info> NUL
    //
    // This function assumes git check-attr diff binary -z -- PATH, so the output contains
    // attributes only for the specified file, and thus we don't even look at path.

    let mut binary_set = false;
    let mut result = DiffAttribute::Unspecified;

    let mut it = output.split(|&b| b == b'\0');
    while let Some(path) = it.next() {
        if path.is_empty() {
            // Bogus "path" after the last NUL.
            // It would be a tad cleaner if split_terminator was available for &[u8]...
            break;
        }
        let attribute = it.next()?;
        let info = it.next()?;

        match attribute {
            b"diff" => {
                // this is only lossy for custom driver variant, which we don't support, and it's
                // quite unlikely to contain invalid UTF-8 (who puts their binaries in files named
                // by invalid strings?)
                match info {
                    b"set" => result = DiffAttribute::AssumeText,
                    b"unset" => result = DiffAttribute::AssumeBinary,
                    _ => (),
                }
            }
            b"binary" => {
                if info == b"set" {
                    binary_set = true;
                }
            }
            _ => {
                warn!(
                    "unexpected attribute in git check-attr output: \"{}\"",
                    attribute.escape_ascii()
                );
            }
        }
    }

    if binary_set {
        // assume user doesn't want to see the diff, even if they specify diff=whatever
        result = DiffAttribute::AssumeBinary;
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_output() {
        // The test case assumes fictional git repository with the following .gitattributes
        // contents to ignore generated protobuf files, but see generated gRPC files, and uses
        // imaginary "hex" diff driver for wtf.bin files:
        //
        //     /protos/*.pb.go -diff
        //     /protos/*_grpc.pb.go diff
        //     wtf.bin binary diff=hex
        //

        // This is a plaintext file, no attributes, assume regular text diff
        assert_eq!(
            parse_output(b"protos/difft.proto\x00diff\x00unspecified\x00binary\x00unspecified\x00"),
            Some(DiffAttribute::Unspecified)
        );

        // This is a generated file with diff attribute unset (we don't want to look into its diff)
        assert_eq!(
            parse_output(b"protos/difft.pb.go\x00diff\x00unset\x00binary\x00unspecified\x00"),
            Some(DiffAttribute::AssumeBinary)
        );

        // diff attribute is explicitly re-enabled for this file
        assert_eq!(
            parse_output(b"protos/difft_grpc.pb.go\x00diff\x00set\x00binary\x00unspecified\x00"),
            Some(DiffAttribute::AssumeText)
        );

        // Although diff=hex basically undoes the effect of binary macro attribute, we assume the
        // user doesn't want to see them here. difft doesn't run git diff drivers, after all.
        assert_eq!(
            parse_output(b"wtf.bin\x00diff\x00hex\x00wtf.bin\x00binary\x00set\x00"),
            Some(DiffAttribute::AssumeBinary)
        );
    }
}

/// 一次原生 Git 调用查询本轮所有路径；查询失败时沿用 CLI 的尽力而为行为。
/// 调用者每次加载替换缓存，不跨加载复用 attributes。
pub(crate) fn check_diff_attrs(
    paths: &[String],
) -> std::collections::HashMap<String, Option<DiffAttribute>> {
    use std::io::Write;
    use std::process::Stdio;
    let mut result = paths
        .iter()
        .map(|p| (p.clone(), None))
        .collect::<std::collections::HashMap<_, _>>();
    if paths.is_empty() {
        return result;
    }
    let run = || -> std::io::Result<std::process::Output> {
        let mut child = Command::new("git")
            .args(["check-attr", "--stdin", "-z", "diff", "binary"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut input = child.stdin.take().expect("piped stdin");
        std::thread::scope(|scope| {
            let writer = scope.spawn(move || -> std::io::Result<()> {
                for path in paths {
                    input.write_all(path.as_bytes())?;
                    input.write_all(&[0])?;
                }
                Ok(())
            });
            let output = child.wait_with_output();
            writer.join().expect("attribute input writer panicked")?;
            output
        })
    };
    let Ok(output) = run() else {
        return result;
    };
    if !output.status.success() {
        return result;
    }
    let fields = output.stdout.split(|b| *b == 0).collect::<Vec<_>>();
    let mut offset = 0;
    for group in fields.chunks_exact(6) {
        let len = group.iter().map(|f| f.len() + 1).sum::<usize>();
        if group[0] == group[3] {
            let path = String::from_utf8_lossy(group[0]).into_owned();
            result.insert(path, parse_output(&output.stdout[offset..offset + len]));
        }
        offset += len;
    }
    result
}

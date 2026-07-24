//! Hardening helpers (S16 / RQ16 / AQ03 / AQ05).
//!
//! - Bind conflicts print an auditor-friendly `ERROR : Address already in use`.
//! - Source audit: the server crate must not call POSIX/`exec*` to spawn another
//!   server (`std::process::Command` / `execve` family).

use std::io::{Error as IoError, ErrorKind};

/// Format a listen/serve failure for stderr (AQ05).
pub fn format_serve_error(err: &IoError) -> String {
    if err.kind() == ErrorKind::AddrInUse {
        "ERROR : Address already in use".to_string()
    } else {
        format!("server: {err}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn addr_in_use_message_matches_audit() {
        let err = IoError::from(ErrorKind::AddrInUse);
        assert_eq!(format_serve_error(&err), "ERROR : Address already in use");
    }

    #[test]
    fn other_errors_keep_server_prefix() {
        let err = IoError::new(ErrorKind::PermissionDenied, "denied");
        let msg = format_serve_error(&err);
        assert!(msg.starts_with("server: "), "{msg}");
    }

    /// AQ03: no `exec*` / `std::process::Command` in the server library/binary sources.
    #[test]
    fn rust_sources_have_no_exec_family() {
        // Built piecewise so this file itself does not contain the needles as
        // contiguous identifiers (aside from this comment's intent).
        let forbidden: [String; 5] = [
            format!("exec{}", "ve"),
            format!("exec{}", "l"),
            format!("exec{}", "lp"),
            format!("exec{}", "vp"),
            ["std", "process", "Command"].join("::"),
        ];
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("src");
        let mut hits = Vec::new();
        for entry in fs::read_dir(&root).expect("src") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            // This module documents the policy; skip scanning its own text.
            if path.file_name().and_then(|n| n.to_str()) == Some("harden.rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read");
            for needle in &forbidden {
                if text.contains(needle) {
                    hits.push(format!("{}: contains `{needle}`", path.display()));
                }
            }
        }
        assert!(
            hits.is_empty(),
            "forbidden exec/spawn APIs in server sources:\n{}",
            hits.join("\n")
        );
    }
}

//! Host-only functional hook checks. Missing shells/Python are reported as skips;
//! byte-level replay and parser unit tests remain fully platform independent.
#![cfg(unix)]
use std::process::Command;
use tavosh_parser::StreamParser;
use tavosh_proto::feed::StructuredOutput;
#[test]
fn interactive_shell_hooks_preserve_commands_and_exit_status() {
    let root = env!("CARGO_MANIFEST_DIR");
    for (shell, hook) in [
        ("bash", "bash.sh"),
        ("zsh", "zsh.sh"),
        ("fish", "fish.fish"),
    ] {
        let result = Command::new("python3")
            .arg(format!("{root}/tests/capture_shell.py"))
            .arg(shell)
            .arg(format!("{root}/src/hooks/{hook}"))
            .output();
        let Ok(result) = result else {
            eprintln!("skip: Python unavailable");
            return;
        };
        if result.status.code() == Some(77) {
            eprintln!("skip: {shell} unavailable");
            continue;
        }
        assert!(
            result.status.success(),
            "{shell}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let mut parser = StreamParser::default();
        for byte in &result.stdout {
            parser.push(&[*byte], 0);
        }
        let snapshot = parser.snapshot();
        assert!(
            snapshot.blocks.iter().any(
                |b| matches!(b.output, StructuredOutput::JsonTree(_)) && b.exit_code == Some(0)
            ),
            "{shell}: {:?}",
            snapshot.blocks
        );
        assert!(
            snapshot
                .blocks
                .iter()
                .any(|b| b.command == "false" && b.exit_code == Some(1)),
            "{shell}: {:?}",
            snapshot.blocks
        );
        assert!(
            snapshot
                .blocks
                .iter()
                .any(|b| b.command == "printf 'hello\\n'"
                    && b.exit_code == Some(0)
                    && b.preview.contains("hello")),
            "{shell}: {:?}",
            snapshot.blocks
        );
    }
}

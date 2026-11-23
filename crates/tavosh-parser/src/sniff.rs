//! Deterministic validators with independent input, depth, node and value caps.
use crate::stream::MAX_OUTPUT;
use tavosh_proto::feed::*;
const MAX_NODES: usize = 512;
const MAX_DEPTH: usize = 24;
const MAX_VALUE: usize = 256;

pub fn strip_ansi(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity(bytes.len().min(MAX_OUTPUT));
    let mut state = 0;
    for &b in bytes.iter().take(MAX_OUTPUT) {
        match state {
            0 => {
                if b == 27 {
                    state = 1
                } else if matches!(b, b'\n' | b'\r' | b'\t' | 8) || b >= 32 {
                    out.push(b);
                }
            }
            1 => {
                state = match b {
                    b'[' => 2,
                    b']' | b'P' | b'_' | b'^' => 3,
                    _ => 0,
                }
            }
            2 => {
                if (0x40..=0x7e).contains(&b) {
                    state = 0;
                }
            }
            3 => {
                if b == 7 {
                    state = 0;
                } else if b == 27 {
                    state = 4;
                }
            }
            _ => state = if b == b'\\' { 0 } else { 3 },
        }
    }
    let text = String::from_utf8_lossy(&out);
    let mut result = String::new();
    let mut line = Vec::new();
    let mut cursor: usize = 0;
    let mut rewound = false;
    let mut rewritten = false;
    for c in text.chars() {
        match c {
            '\r' => {
                cursor = 0;
                rewound = true;
            }
            '\u{8}' => {
                cursor = cursor.saturating_sub(1);
            }
            '\n' => {
                result.extend(line.drain(..));
                result.push('\n');
                cursor = 0;
                rewound = false;
                rewritten = false;
            }
            c => {
                rewritten |= rewound;
                if cursor < line.len() {
                    line[cursor] = c;
                } else {
                    line.push(c);
                }
                cursor += 1;
            }
        }
    }
    if !(rewritten && line.iter().all(|&c| c == ' ')) {
        result.extend(line);
    }
    result
}
fn bounded(s: &str) -> String {
    let mut end = s.len().min(MAX_VALUE);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_owned()
}
fn tree(
    value: &serde_json::Value,
    label: String,
    path: String,
    depth: usize,
    budget: &mut usize,
) -> TreeNode {
    *budget -= 1;
    let (kind, scalar, length) = match value {
        serde_json::Value::Object(v) => (TreeKind::Object, None, v.len()),
        serde_json::Value::Array(v) => (TreeKind::Array, None, v.len()),
        serde_json::Value::String(v) => (TreeKind::String, Some(bounded(v)), v.len()),
        serde_json::Value::Number(v) => (TreeKind::Number, Some(v.to_string()), 0),
        serde_json::Value::Bool(v) => (TreeKind::Boolean, Some(v.to_string()), 0),
        serde_json::Value::Null => (TreeKind::Null, Some("null".into()), 0),
    };
    let mut node = TreeNode {
        label: bounded(&label),
        path,
        kind,
        value: scalar,
        length,
        children: vec![],
        truncated: false,
    };
    let items: Box<dyn Iterator<Item = (String, &serde_json::Value)> + '_> = match value {
        serde_json::Value::Object(v) => Box::new(v.iter().map(|(k, v)| (k.clone(), v))),
        serde_json::Value::Array(v) => {
            Box::new(v.iter().enumerate().map(|(k, v)| (k.to_string(), v)))
        }
        _ => Box::new(std::iter::empty()),
    };
    for (key, child) in items {
        // Path bytes are bounded independently; never emit an inaccurate path.
        let path = format!(
            "{}/{}",
            node.path,
            key.replace('~', "~0").replace('/', "~1")
        );
        if *budget == 0 || depth >= MAX_DEPTH || path.len() > 2048 {
            node.truncated = true;
            break;
        }
        node.children
            .push(tree(child, key, path, depth + 1, budget));
    }
    if let serde_json::Value::String(v) = value {
        node.truncated = v.len() > MAX_VALUE;
    }
    node
}

fn toml_tree(
    value: &toml::Value,
    label: String,
    path: String,
    depth: usize,
    budget: &mut usize,
) -> TreeNode {
    *budget -= 1;
    let (kind, scalar, length) = match value {
        toml::Value::Table(v) => (TreeKind::Object, None, v.len()),
        toml::Value::Array(v) => (TreeKind::Array, None, v.len()),
        toml::Value::String(v) => (TreeKind::String, Some(v.clone()), v.len()),
        toml::Value::Integer(v) => (TreeKind::Number, Some(v.to_string()), 0),
        toml::Value::Float(v) => (
            TreeKind::Number,
            Some(if v.is_nan() {
                "nan".into()
            } else {
                v.to_string()
            }),
            0,
        ),
        toml::Value::Boolean(v) => (TreeKind::Boolean, Some(v.to_string()), 0),
        toml::Value::Datetime(v) => (TreeKind::String, Some(v.to_string()), 0),
    };
    let truncated = scalar.as_ref().is_some_and(|s| s.len() > MAX_VALUE);
    let mut node = TreeNode {
        label: bounded(&label),
        path,
        kind,
        value: scalar.as_deref().map(bounded),
        length,
        children: vec![],
        truncated,
    };
    let items: Box<dyn Iterator<Item = (String, &toml::Value)> + '_> = match value {
        toml::Value::Table(v) => Box::new(v.iter().map(|(k, v)| (k.clone(), v))),
        toml::Value::Array(v) => Box::new(v.iter().enumerate().map(|(k, v)| (k.to_string(), v))),
        _ => Box::new(std::iter::empty()),
    };
    for (key, child) in items {
        let path = format!(
            "{}/{}",
            node.path,
            key.replace('~', "~0").replace('/', "~1")
        );
        if *budget == 0 || depth >= MAX_DEPTH || path.len() > 2048 {
            node.truncated = true;
            break;
        }
        node.children
            .push(toml_tree(child, key, path, depth + 1, budget));
    }
    node
}

pub fn classify(bytes: &[u8], total_bytes: u64) -> StructuredOutput {
    let output = classify_inner(bytes, total_bytes);
    // Bound the presentation document as well as its input. Huge diffs or
    // adversarially long object paths retain their plain-text preview.
    if serde_json::to_vec(&output).map_or(true, |bytes| bytes.len() > 256 * 1024) {
        StructuredOutput::PlainText
    } else {
        output
    }
}
fn classify_inner(bytes: &[u8], total_bytes: u64) -> StructuredOutput {
    if total_bytes > MAX_OUTPUT as u64 || bytes.len() > MAX_OUTPUT {
        return StructuredOutput::PlainText;
    }
    let clean = strip_ansi(bytes);
    let text = clean.trim();
    if text.is_empty() {
        return StructuredOutput::PlainText;
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        return StructuredOutput::JsonTree(tree(
            &value,
            "$".into(),
            "".into(),
            0,
            &mut MAX_NODES.clone(),
        ));
    }
    if let Some(files) = diff(clean.trim_matches('\n')) {
        return StructuredOutput::UnifiedDiff(files);
    }
    if let Ok(value) = toml::from_str::<toml::Value>(text) {
        return StructuredOutput::TomlTree(toml_tree(
            &value,
            "$".into(),
            "".into(),
            0,
            &mut MAX_NODES.clone(),
        ));
    }
    StructuredOutput::PlainText
}
fn range(s: &str, prefix: char) -> Option<u32> {
    let s = s.strip_prefix(prefix)?;
    match s.split_once(',') {
        Some((start, count)) => {
            start.parse::<u32>().ok()?;
            count.parse().ok()
        }
        None => {
            s.parse::<u32>().ok()?;
            Some(1)
        }
    }
}
/// Accept text unified patches only when hunk counts match exactly. Binary,
/// combined, truncated and malformed patches retain their plain-text preview.
pub fn diff(text: &str) -> Option<Vec<DiffFile>> {
    let lines: Vec<&str> = text.lines().collect();
    let mut files = Vec::new();
    let mut i = 0;
    let mut budget = MAX_NODES;
    while i < lines.len() {
        if lines[i].starts_with("diff --git ")
            || lines[i].starts_with("index ")
            || lines[i].starts_with("new file mode ")
            || lines[i].starts_with("deleted file mode ")
            || lines[i].starts_with("old mode ")
            || lines[i].starts_with("new mode ")
        {
            i += 1;
            continue;
        }
        let old = lines[i].strip_prefix("--- ")?;
        i += 1;
        let new = lines.get(i)?.strip_prefix("+++ ")?;
        i += 1;
        let mut file = DiffFile {
            old_path: old.into(),
            new_path: new.into(),
            added: 0,
            removed: 0,
            hunks: vec![],
        };
        if old.len() > 2048 || new.len() > 2048 {
            return None;
        }
        while i < lines.len() && lines[i].starts_with("@@ ") {
            let header = lines[i];
            let mut parts = header.split_whitespace();
            parts.next()?;
            let mut old_left = range(parts.next()?, '-')?;
            let mut new_left = range(parts.next()?, '+')?;
            if parts.next()? != "@@" {
                return None;
            }
            let mut hunk = DiffHunk {
                header: bounded(header),
                lines: vec![],
            };
            i += 1;
            while old_left != 0 || new_left != 0 {
                if budget == 0 {
                    return None;
                }
                budget -= 1;
                let line = *lines.get(i)?;
                let (kind, text) = match line.as_bytes().first()? {
                    b'+' => {
                        new_left = new_left.checked_sub(1)?;
                        file.added += 1;
                        (DiffLineKind::Added, &line[1..])
                    }
                    b'-' => {
                        old_left = old_left.checked_sub(1)?;
                        file.removed += 1;
                        (DiffLineKind::Removed, &line[1..])
                    }
                    b' ' => {
                        old_left = old_left.checked_sub(1)?;
                        new_left = new_left.checked_sub(1)?;
                        (DiffLineKind::Context, &line[1..])
                    }
                    b'\\' if line == "\\ No newline at end of file" => {
                        (DiffLineKind::Metadata, line)
                    }
                    _ => return None,
                };
                if text.len() > 4096 {
                    return None;
                }
                hunk.lines.push(DiffLine {
                    kind,
                    text: text.into(),
                });
                i += 1;
            }
            if lines.get(i) == Some(&"\\ No newline at end of file") {
                i += 1;
            }
            file.hunks.push(hunk);
        }
        if file.hunks.is_empty() {
            return None;
        }
        files.push(file);
        if files.len() > 64 {
            return None;
        }
    }
    (!files.is_empty()).then_some(files)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colored_json_toml_and_paths() {
        let output = classify(b"\x1b[32m{\"a/b\": [true, null, 3, \"s\"]}\x1b[0m", 40);
        let StructuredOutput::JsonTree(node) = output else {
            panic!("json")
        };
        assert_eq!(node.children[0].path, "/a~1b");
        assert_eq!(node.children[0].children[3].path, "/a~1b/3");
        assert!(matches!(
            classify(b"\x1b[31m[package]\nname = 'tavo'\x1b[0m", 40),
            StructuredOutput::TomlTree(_)
        ));
        for text in [
            "{broken",
            "[package",
            "hello world",
            "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n+only",
        ] {
            assert_eq!(
                classify(text.as_bytes(), text.len() as u64),
                StructuredOutput::PlainText
            );
        }
    }
    #[test]
    fn colored_diff_validates_counts() {
        let text = "diff --git a/a b/a\nindex 0..1 100644\n--- a/a\n+++ b/a\n@@ -1,2 +1,2 @@\n context\n\x1b[31m-old\x1b[0m\n\x1b[32m+new\x1b[0m\n";
        let StructuredOutput::UnifiedDiff(files) = classify(text.as_bytes(), text.len() as u64)
        else {
            panic!("diff")
        };
        assert_eq!((files[0].added, files[0].removed), (1, 1));
        assert!(diff("--- a\n+++ b\n@@ -1 +1 @@\n-old\n+new\ntrailing").is_none());
    }
    #[test]
    fn node_depth_and_size_guards() {
        let text = format!("[{}]", vec!["1"; 1000].join(","));
        let StructuredOutput::JsonTree(tree) = classify(text.as_bytes(), text.len() as u64) else {
            panic!("json")
        };
        assert_eq!(tree.children.len(), MAX_NODES - 1);
        assert!(tree.truncated);
        assert_eq!(
            classify(b"{}", MAX_OUTPUT as u64 + 1),
            StructuredOutput::PlainText
        );
        let mut parser = crate::StreamParser::default();
        parser.push(b"\x1b]133;C\x07", 0);
        parser.push(&vec![b'x'; MAX_OUTPUT * 2], 1);
        parser.push(b"\x1b]133;D;0\x07", 2);
        let block = parser.snapshot().blocks.remove(0);
        assert!(block.truncated);
        assert_eq!(block.total_bytes, (MAX_OUTPUT * 2) as u64);
        assert_eq!(block.preview.len(), crate::stream::MAX_PREVIEW);
    }
}

#[cfg(test)]
mod whitespace_tests {
    use super::*;
    #[test]
    fn final_blank_context_line_keeps_its_diff_prefix() {
        let text = b"--- a/a\n+++ b/a\n@@ -1,2 +1,2 @@\n-old\n+new\n \n";
        assert!(matches!(
            classify(text, text.len() as u64),
            StructuredOutput::UnifiedDiff(_)
        ));
    }
}

#[cfg(test)]
mod toml_tests {
    use super::*;
    #[test]
    fn nonfinite_toml_numbers_and_dates_keep_their_values() {
        let text = b"n = nan\ni = inf\nt = 1979-05-27T07:32:00Z\n";
        let StructuredOutput::TomlTree(tree) = classify(text, text.len() as u64) else {
            panic!("toml");
        };
        let n = tree.children.iter().find(|n| n.label == "n").unwrap();
        assert_eq!(n.kind, TreeKind::Number);
        assert_eq!(n.value.as_deref(), Some("nan"));
        let t = tree.children.iter().find(|n| n.label == "t").unwrap();
        assert_eq!(t.kind, TreeKind::String);
        assert_eq!(t.value.as_deref(), Some("1979-05-27T07:32:00Z"));
    }
}

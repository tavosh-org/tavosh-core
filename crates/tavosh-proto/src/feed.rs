//! Adaptive feed schema v1. Additive optional fields are compatible; changes to
//! variant meaning require a new schema version. Paths use RFC 6901 pointers.
use serde::{Deserialize, Serialize};
pub const FEED_SCHEMA_VERSION: u16 = 1;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandBlock {
    pub schema_version: u16,
    pub id: u64,
    pub command: String,
    pub cwd: Option<String>,
    /// Monotonic milliseconds relative to session start.
    pub started_ms: u64,
    pub duration_ms: Option<u64>,
    pub state: BlockState,
    pub exit_code: Option<i32>,
    pub output: StructuredOutput,
    pub preview: String,
    pub truncated: bool,
    pub total_bytes: u64,
    pub total_lines: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockState {
    Running,
    Completed,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum StructuredOutput {
    PlainText,
    JsonTree(TreeNode),
    TomlTree(TreeNode),
    UnifiedDiff(Vec<DiffFile>),
    TuiSummary { duration_ms: u64 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    pub path: String,
    pub label: String,
    pub kind: TreeKind,
    pub value: Option<String>,
    pub length: usize,
    pub children: Vec<TreeNode>,
    pub truncated: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreeKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffFile {
    pub old_path: String,
    pub new_path: String,
    pub added: u32,
    pub removed: u32,
    pub hunks: Vec<DiffHunk>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
    Metadata,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewportMode {
    #[default]
    Feed,
    TuiCanvas,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgramState {
    Working,
    Done,
    Blocked,
    Error,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramStatusEvent {
    pub schema_version: u16,
    pub state: ProgramState,
    pub kind: String,
    pub app: String,
    pub message: Option<String>,
    pub title: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BlockEvent {
    Started(CommandBlock),
    OutputDelta(CommandBlock),
    Completed(CommandBlock),
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FeedSnapshot {
    pub schema_version: u16,
    pub generation: u64,
    pub blocks: Vec<CommandBlock>,
    pub viewport: ViewportMode,
    pub status: Option<ProgramStatusEvent>,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_all_variants() {
        let node = TreeNode {
            path: "/a~1b".into(),
            label: "a/b".into(),
            kind: TreeKind::String,
            value: Some("hello".into()),
            length: 0,
            children: vec![],
            truncated: false,
        };
        for output in [
            StructuredOutput::PlainText,
            StructuredOutput::JsonTree(node.clone()),
            StructuredOutput::TomlTree(node),
            StructuredOutput::UnifiedDiff(vec![DiffFile {
                old_path: "a/a".into(),
                new_path: "b/a".into(),
                added: 1,
                removed: 0,
                hunks: vec![DiffHunk {
                    header: "@@ -0,0 +1 @@".into(),
                    lines: vec![DiffLine {
                        kind: DiffLineKind::Added,
                        text: "hello".into(),
                    }],
                }],
            }]),
            StructuredOutput::TuiSummary { duration_ms: 42 },
        ] {
            let block = CommandBlock {
                schema_version: FEED_SCHEMA_VERSION,
                id: 1,
                command: "echo".into(),
                cwd: None,
                started_ms: 0,
                duration_ms: Some(42),
                state: BlockState::Completed,
                exit_code: Some(0),
                output,
                preview: String::new(),
                truncated: false,
                total_bytes: 0,
                total_lines: 0,
            };
            for event in [
                BlockEvent::Started(block.clone()),
                BlockEvent::OutputDelta(block.clone()),
                BlockEvent::Completed(block),
            ] {
                assert_eq!(
                    serde_json::from_slice::<BlockEvent>(&serde_json::to_vec(&event).unwrap())
                        .unwrap(),
                    event
                );
            }
        }
        for state in [
            ProgramState::Working,
            ProgramState::Done,
            ProgramState::Blocked,
            ProgramState::Error,
        ] {
            let snapshot = FeedSnapshot {
                schema_version: FEED_SCHEMA_VERSION,
                generation: 1,
                blocks: vec![],
                viewport: ViewportMode::TuiCanvas,
                status: Some(ProgramStatusEvent {
                    schema_version: FEED_SCHEMA_VERSION,
                    state,
                    kind: "build".into(),
                    app: "cargo".into(),
                    message: None,
                    title: Some("Build".into()),
                }),
            };
            assert_eq!(
                serde_json::from_str::<FeedSnapshot>(&serde_json::to_string(&snapshot).unwrap())
                    .unwrap(),
                snapshot
            );
        }
    }
}

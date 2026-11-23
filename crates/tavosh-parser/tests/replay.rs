use tavosh_parser::StreamParser;
use tavosh_proto::{feed::*, trace::Reader};
fn replay(bytes: &[u8], fragment: usize) -> FeedSnapshot {
    let mut reader = Reader::new(bytes).unwrap();
    let mut parser = StreamParser::default();
    let mut now = 0;
    let mut saw_tui = false;
    while let Some(record) = reader.read_record().unwrap() {
        now += u64::from(record.delta_ms);
        parser.tick(now);
        for chunk in record.payload.chunks(fragment) {
            parser.push(chunk, now);
        }
        saw_tui |= parser.snapshot().viewport == ViewportMode::TuiCanvas;
    }
    parser.tick(now + 100);
    let snapshot = parser.snapshot();
    assert!(saw_tui);
    assert_eq!(snapshot.viewport, ViewportMode::Feed);
    assert!(
        snapshot
            .blocks
            .iter()
            .all(|b| b.state == BlockState::Completed && !b.preview.contains("private TUI"))
    );
    snapshot
}
#[test]
fn osc_replay_is_independent_of_read_boundaries() {
    let bytes = include_bytes!("../../../fixtures/osc133_adaptive_session.trace");
    let expected = replay(bytes, usize::MAX);
    assert_eq!(expected.blocks.len(), 5);
    assert!(matches!(
        expected.blocks[0].output,
        StructuredOutput::JsonTree(_)
    ));
    assert!(matches!(
        expected.blocks[1].output,
        StructuredOutput::TomlTree(_)
    ));
    assert!(matches!(
        expected.blocks[2].output,
        StructuredOutput::UnifiedDiff(_)
    ));
    assert!(matches!(
        expected.blocks[3].output,
        StructuredOutput::TuiSummary { duration_ms: 1200 }
    ));
    assert_eq!(expected.blocks[4].exit_code, Some(1));
    assert_eq!(expected.status.as_ref().unwrap().state, ProgramState::Done);
    for size in [1, 2, 7, 31, 128] {
        let actual = replay(bytes, size);
        assert_eq!(actual.blocks, expected.blocks);
        assert_eq!(actual.status, expected.status);
    }
}
#[test]
fn heuristic_replay_slices_at_controlled_quiescence() {
    let bytes = include_bytes!("../../../fixtures/heuristic_fallback.trace");
    let snapshot = replay(bytes, 1);
    assert_eq!(snapshot.blocks.len(), 3);
    assert_eq!(snapshot.blocks[0].command, "echo hello");
    assert!(matches!(
        snapshot.blocks[1].output,
        StructuredOutput::JsonTree(_)
    ));
    assert!(matches!(
        snapshot.blocks[2].output,
        StructuredOutput::TuiSummary { .. }
    ));
}

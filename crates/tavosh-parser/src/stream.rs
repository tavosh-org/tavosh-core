//! Incremental, bounded native stream classifier. All clocks are supplied by
//! the worker: tests and replay never depend on wall-clock scheduling.
use std::collections::VecDeque;
use tavosh_proto::feed::*;

pub const MAX_OUTPUT: usize = 1024 * 1024;
pub const MAX_PREVIEW: usize = 16 * 1024;
pub const MAX_BLOCKS: usize = 32;
const MAX_CONTROL: usize = 4096;

#[derive(Clone, Debug)]
pub struct Config {
    pub quiescence_ms: u64,
    pub prompt_suffixes: Vec<String>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            quiescence_ms: 75,
            prompt_suffixes: ["$ ", "# ", "% ", "❯ "].map(String::from).to_vec(),
        }
    }
}
#[derive(Default)]
enum Escape {
    #[default]
    Ground,
    Esc,
    Csi(Vec<u8>),
    Osc(Vec<u8>, bool),
    DiscardOsc(bool),
    DiscardCsi,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Prompt,
    Input,
    Output,
}

pub struct StreamParser {
    config: Config,
    escape: Escape,
    phase: Phase,
    osc133: bool,
    snapshot: FeedSnapshot,
    events: VecDeque<BlockEvent>,
    active: Option<CommandBlock>,
    output: Vec<u8>,
    input: Vec<u8>,
    last_ms: u64,
    next_id: u64,
    tui_started: Option<u64>,
    tui_duration: Option<u64>,
    line_tail: Vec<u8>,
}
impl Default for StreamParser {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
impl StreamParser {
    pub fn new(mut config: Config) -> Self {
        config.quiescence_ms = config.quiescence_ms.clamp(50, 100);
        config.prompt_suffixes.truncate(16);
        config
            .prompt_suffixes
            .retain(|s| !s.is_empty() && s.len() <= 128);
        Self {
            config,
            escape: Escape::Ground,
            phase: Phase::Prompt,
            osc133: false,
            snapshot: FeedSnapshot {
                schema_version: FEED_SCHEMA_VERSION,
                ..Default::default()
            },
            events: VecDeque::new(),
            active: None,
            output: vec![],
            input: vec![],
            last_ms: 0,
            next_id: 1,
            tui_started: None,
            tui_duration: None,
            line_tail: vec![],
        }
    }
    pub fn is_osc133(&self) -> bool {
        self.osc133
    }
    pub fn snapshot(&self) -> FeedSnapshot {
        let mut snapshot = self.snapshot.clone();
        if let Some(block) = &self.active {
            snapshot.blocks.push(block.clone());
        }
        snapshot
    }
    pub fn generation(&self) -> u64 {
        self.snapshot.generation
    }
    pub fn drain_events(&mut self) -> impl Iterator<Item = BlockEvent> + '_ {
        self.events.drain(..)
    }
    fn event(&mut self, event: BlockEvent) {
        if self.events.len() == 64 {
            self.events.pop_front();
        }
        self.events.push_back(event);
        self.snapshot.generation += 1;
    }
    /// Register locally submitted text without exposing raw remote bytes to JS.
    /// OSC 133 input echo remains authoritative when hooks are available.
    pub fn submitted(&mut self, command: &str, now: u64) {
        if !self.osc133 && self.active.is_none() {
            self.start(command.trim().to_owned(), now);
            self.phase = Phase::Output;
        }
    }
    fn start(&mut self, command: String, now: u64) {
        if self.active.is_some() {
            self.complete(None, now);
        }
        self.output.clear();
        self.tui_duration = None;
        let block = CommandBlock {
            schema_version: FEED_SCHEMA_VERSION,
            id: self.next_id,
            command,
            cwd: None,
            started_ms: now,
            duration_ms: None,
            state: BlockState::Running,
            exit_code: None,
            output: StructuredOutput::PlainText,
            preview: String::new(),
            truncated: false,
            total_bytes: 0,
            total_lines: 0,
        };
        self.next_id += 1;
        self.event(BlockEvent::Started(block.clone()));
        self.active = Some(block);
    }
    fn complete(&mut self, exit: Option<i32>, now: u64) {
        self.refresh_preview();
        if !self.osc133 {
            let text = String::from_utf8_lossy(&self.output).into_owned();
            let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
            if let Some(last) = lines.last()
                && self
                    .config
                    .prompt_suffixes
                    .iter()
                    .any(|suffix| last.ends_with(suffix))
            {
                lines.pop();
            }
            if let Some(first) = lines.first().copied()
                && let Some((offset, suffix)) = self
                    .config
                    .prompt_suffixes
                    .iter()
                    .filter_map(|suffix| first.find(suffix).map(|offset| (offset, suffix)))
                    .min_by_key(|(offset, _)| *offset)
            {
                if let Some(block) = &mut self.active
                    && block.command.is_empty()
                {
                    block.command = first[offset + suffix.len()..].trim().to_owned();
                }
                lines.remove(0);
            }
            self.output = lines.concat().into_bytes();
            self.refresh_preview();
            if self.tui_duration.is_none()
                && self.output.is_empty()
                && self.active.as_ref().is_some_and(|b| b.command.is_empty())
            {
                self.active = None;
                return;
            }
        }
        if let Some(mut block) = self.active.take() {
            block.state = BlockState::Completed;
            block.exit_code = exit;
            block.duration_ms = Some(now.saturating_sub(block.started_ms));
            block.output = if let Some(duration_ms) = self.tui_duration.take() {
                StructuredOutput::TuiSummary { duration_ms }
            } else {
                classify(&self.output, block.total_bytes)
            };
            self.event(BlockEvent::Completed(block.clone()));
            if self.snapshot.blocks.len() >= MAX_BLOCKS - 1 {
                self.snapshot.blocks.remove(0);
            }
            self.snapshot.blocks.push(block);
            self.output.clear();
        }
    }
    pub fn tick(&mut self, now: u64) {
        if !self.osc133
            && self.snapshot.viewport == ViewportMode::Feed
            && self.active.is_some()
            && now.saturating_sub(self.last_ms) >= self.config.quiescence_ms
        {
            // A prompt confirms a boundary. Quiescence alone also allows output
            // from appliances without a recognizable prompt to become a card.
            self.complete(None, now);
            self.phase = Phase::Prompt;
        }
    }
    pub fn feed(&mut self, bytes: &[u8], now: u64) {
        self.last_ms = now;
        for &byte in bytes {
            self.byte(byte, now);
        }
        self.refresh_preview();
        if let Some(block) = &self.active {
            self.event(BlockEvent::OutputDelta(block.clone()));
        }
    }
    fn byte(&mut self, byte: u8, now: u64) {
        let state = std::mem::take(&mut self.escape);
        self.escape = match state {
            Escape::Ground => match byte {
                0x1b => Escape::Esc,
                _ => {
                    self.text(byte, now);
                    Escape::Ground
                }
            },
            Escape::Esc => match byte {
                b']' => Escape::Osc(vec![], false),
                b'[' => Escape::Csi(vec![]),
                0x1b => Escape::Esc,
                _ => Escape::Ground,
            },
            Escape::Csi(mut buf) => {
                if (0x40..=0x7e).contains(&byte) {
                    self.csi(&buf, byte, now);
                    Escape::Ground
                } else if buf.len() < MAX_CONTROL {
                    buf.push(byte);
                    Escape::Csi(buf)
                } else {
                    Escape::DiscardCsi
                }
            }
            Escape::Osc(mut buf, esc) => {
                if byte == 7 || (esc && byte == b'\\') {
                    self.osc(&buf, now);
                    Escape::Ground
                } else if buf.len() >= MAX_CONTROL {
                    Escape::DiscardOsc(byte == 0x1b)
                } else {
                    if esc {
                        buf.push(0x1b);
                    }
                    if byte != 0x1b {
                        buf.push(byte);
                    }
                    Escape::Osc(buf, byte == 0x1b)
                }
            }
            Escape::DiscardOsc(esc) => {
                if byte == 7 || (esc && byte == b'\\') {
                    Escape::Ground
                } else {
                    Escape::DiscardOsc(byte == 0x1b)
                }
            }
            Escape::DiscardCsi => {
                if (0x40..=0x7e).contains(&byte) {
                    Escape::Ground
                } else {
                    Escape::DiscardCsi
                }
            }
        };
    }
    fn text(&mut self, byte: u8, now: u64) {
        if self.snapshot.viewport == ViewportMode::TuiCanvas {
            return;
        }
        if self.osc133 && self.phase == Phase::Input {
            if self.input.len() < MAX_CONTROL {
                self.input.push(byte);
            }
            return;
        }
        if self.osc133 && self.phase != Phase::Output {
            return;
        }
        if !self.osc133 {
            if byte == b'\n' {
                self.line_tail.clear();
            } else if self.line_tail.len() < MAX_CONTROL {
                self.line_tail.push(byte);
            }
            if self.phase == Phase::Prompt {
                if byte == b'\r' || byte == b'\n' {
                    return;
                }
                self.start(String::new(), now);
                self.phase = Phase::Output;
            }
        }
        if let Some(block) = &mut self.active {
            block.total_bytes += 1;
            if byte == b'\n' {
                block.total_lines += 1;
            }
            if self.output.len() < MAX_OUTPUT {
                self.output.push(byte);
            }
            // Build UTF-8 only at chunk boundaries below, avoiding quadratic
            // per-byte conversion. This also preserves split multibyte scalars.
            block.truncated = block.total_bytes > MAX_PREVIEW as u64;
        }
    }
    fn osc(&mut self, buf: &[u8], now: u64) {
        let Ok(text) = std::str::from_utf8(buf) else {
            return;
        };
        if let Some(marker) = text.strip_prefix("133;") {
            let (marker, exit) = match marker.split_once(';') {
                Some((m, e)) => (m, e.parse().ok()),
                None => (marker, None),
            };
            if !matches!(marker, "A" | "B" | "C" | "D") {
                return;
            }
            if !self.osc133 {
                self.complete(None, now);
                self.osc133 = true;
            }
            match marker {
                "A" => {
                    self.complete(None, now);
                    self.phase = Phase::Prompt;
                    self.input.clear();
                }
                "B" => {
                    self.phase = Phase::Input;
                    self.input.clear();
                }
                "C" => {
                    self.start(String::from_utf8_lossy(&self.input).trim().to_owned(), now);
                    self.phase = Phase::Output;
                }
                "D" => {
                    self.complete(exit, now);
                    self.phase = Phase::Prompt;
                }
                _ => {}
            }
        } else if let Some(status) = text.strip_prefix("7501;").and_then(parse_status) {
            self.snapshot.status = Some(status);
            self.snapshot.generation += 1;
        }
    }
    fn csi(&mut self, params: &[u8], final_byte: u8, now: u64) {
        if !matches!(final_byte, b'h' | b'l') || params.first() != Some(&b'?') {
            return;
        }
        if !params[1..]
            .split(|&b| b == b';')
            .any(|p| matches!(p, b"1049" | b"1047" | b"47"))
        {
            return;
        }
        let next = if final_byte == b'h' {
            ViewportMode::TuiCanvas
        } else {
            ViewportMode::Feed
        };
        if next == self.snapshot.viewport {
            return;
        }
        if next == ViewportMode::TuiCanvas {
            if self.active.is_none() {
                self.start(String::new(), now);
            }
            self.tui_started = Some(now);
            self.output.clear();
            if let Some(block) = &mut self.active {
                block.preview.clear();
                block.total_bytes = 0;
                block.total_lines = 0;
            }
        } else {
            self.tui_duration = self.tui_started.take().map(|t| now.saturating_sub(t));
            // In OSC mode D supplies the authoritative exit status later.
            if !self.osc133 {
                self.complete(None, now);
                self.phase = Phase::Prompt;
            }
        }
        self.snapshot.viewport = next;
        self.snapshot.generation += 1;
    }
    fn refresh_preview(&mut self) {
        if let Some(block) = &mut self.active {
            let end = self.output.len().min(MAX_PREVIEW);
            block.preview = String::from_utf8_lossy(&self.output[..end]).into_owned();
        }
    }
    /// Feed with a preview refresh once per chunk, independent of byte count.
    pub fn push(&mut self, bytes: &[u8], now: u64) {
        self.feed(bytes, now);
        self.refresh_preview();
    }
}
use crate::sniff::classify;

pub fn parse_status(text: &str) -> Option<ProgramStatusEvent> {
    if text.len() > MAX_CONTROL {
        return None;
    }
    let mut values = std::collections::BTreeMap::new();
    for field in text.split(';') {
        let (key, value) = field.split_once('=')?;
        if values.insert(key, value).is_some() {
            return None;
        }
    }
    let state = match *values.get("state")? {
        "working" => ProgramState::Working,
        "done" => ProgramState::Done,
        "blocked" => ProgramState::Blocked,
        "error" => ProgramState::Error,
        _ => return None,
    };
    Some(ProgramStatusEvent {
        schema_version: FEED_SCHEMA_VERSION,
        state,
        kind: values.get("kind").copied().unwrap_or("").into(),
        app: values.get("app").copied().unwrap_or("").into(),
        message: values.get("message").map(|v| (*v).into()),
        title: values.get("title").map(|v| (*v).into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_fragment_boundary_and_terminator() {
        for end in ["\x07", "\x1b\\"] {
            let trace = format!(
                "\x1b]133;A{end}host$ \x1b]133;B{end}echo café\r\n\x1b]133;C{end}café\r\n\x1b]7501;state=done;app=test;kind=build{end}\x1b]133;D;7{end}"
            );
            for split in 0..=trace.len() {
                let mut parser = StreamParser::default();
                parser.push(&trace.as_bytes()[..split], 0);
                parser.push(&trace.as_bytes()[split..], 10);
                let snapshot = parser.snapshot();
                assert_eq!(snapshot.blocks.len(), 1, "split {split}");
                let block = &snapshot.blocks[0];
                assert_eq!(block.command, "echo café");
                assert_eq!(block.exit_code, Some(7));
                assert_eq!(block.preview, "café\r\n");
                assert_eq!(snapshot.status.unwrap().state, ProgramState::Done);
            }
        }
    }
    #[test]
    fn controlled_quiescence_and_promotion() {
        let mut parser = StreamParser::default();
        parser.push(b"user@host:~$ echo hello\r\nhello\r\nuser@host:~$ ", 0);
        parser.tick(74);
        assert_eq!(parser.snapshot().blocks[0].state, BlockState::Running);
        parser.tick(75);
        let snapshot = parser.snapshot();
        assert_eq!(snapshot.blocks[0].command, "echo hello");
        assert_eq!(snapshot.blocks[0].preview, "hello\r\n");
        parser.push(b"\x1b]133;A\x07", 100);
        assert!(parser.is_osc133());
        parser.push(b"\x1b]133;B\x07sleep 10\x1b]133;C\x07", 101);
        parser.tick(10000);
        assert_eq!(
            parser.snapshot().blocks.last().unwrap().state,
            BlockState::Running
        );
    }
    #[test]
    fn alt_modes_and_bounded_controls() {
        for mode in ["1049", "1047", "47"] {
            let mut parser = StreamParser::default();
            parser.push(format!("\x1b[?{mode}hsecret screen").as_bytes(), 10);
            assert_eq!(parser.snapshot().viewport, ViewportMode::TuiCanvas);
            parser.push(format!("\x1b[?{mode}l").as_bytes(), 110);
            assert_eq!(parser.snapshot().viewport, ViewportMode::Feed);
            assert_eq!(
                parser.snapshot().blocks[0].output,
                StructuredOutput::TuiSummary { duration_ms: 100 }
            );
            assert!(parser.snapshot().blocks[0].preview.is_empty());
        }
        let mut parser = StreamParser::default();
        parser.push(b"\x1b]", 0);
        parser.push(&vec![b'x'; MAX_CONTROL * 2], 0);
        parser.push(b"\x07\x1b]133;A\x07", 1);
        assert!(parser.is_osc133());
        assert!(parse_status("state=unknown").is_none());
        assert!(parse_status("state=done;state=error").is_none());
    }
}

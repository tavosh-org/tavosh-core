//! Terminal stream parsing and structured output detection.
#![forbid(unsafe_code)]

use tavosh_proto::{SMOKE_GRID_HEIGHT, SMOKE_GRID_WIDTH};

/// Validates the fixed-length M0 display snapshot.
pub fn is_smoke_grid(cells: &[u8]) -> bool {
    cells.len() == SMOKE_GRID_WIDTH * SMOKE_GRID_HEIGHT
        && cells.iter().all(|cell| matches!(cell, b' ' | b'#'))
}

pub mod stream;
pub use stream::{Config, StreamParser};
/// Shell-specific sources; callers must select the detected interactive shell.
pub const BASH_HOOK: &str = include_str!("hooks/bash.sh");
pub const ZSH_HOOK: &str = include_str!("hooks/zsh.sh");
pub const FISH_HOOK: &str = include_str!("hooks/fish.fish");

pub mod sniff;

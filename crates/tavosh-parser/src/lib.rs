//! Terminal stream parsing and structured output detection.
#![forbid(unsafe_code)]

use tavosh_proto::{SMOKE_GRID_HEIGHT, SMOKE_GRID_WIDTH};

/// Validates the fixed-length M0 display snapshot.
pub fn is_smoke_grid(cells: &[u8]) -> bool {
    cells.len() == SMOKE_GRID_WIDTH * SMOKE_GRID_HEIGHT
        && cells.iter().all(|cell| matches!(cell, b' ' | b'#'))
}

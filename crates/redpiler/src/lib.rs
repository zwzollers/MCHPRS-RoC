pub mod compile_graph;
pub mod passes;
pub mod ril;
pub mod string_replacer;
pub mod task_monitor;

use mchprs_blocks::blocks::Block;
use mchprs_blocks::BlockPos;
use mchprs_world::{for_each_block_mut_optimized, TickEntry, World};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, trace, warn};

pub use task_monitor::TaskMonitor;

use crate::{compile_graph::CompileGraph};

pub fn block_powered_mut(block: &mut Block) -> Option<&mut bool> {
    Some(match block {
        Block::Comparator(comparator) => &mut comparator.powered,
        Block::RedstoneTorch { lit } => lit,
        Block::RedstoneWallTorch { lit, .. } => lit,
        Block::Repeater(repeater) => &mut repeater.powered,
        Block::Lever { powered, .. } => powered,
        Block::StoneButton { powered, .. } => powered,
        Block::RedstoneLamp { lit } => lit,
        Block::IronTrapdoor { powered, .. } => powered,
        Block::NoteBlock { powered, .. } => powered,
        _ => return block.get_pressure_plate_powered(),
    })
}

#[derive(Default)]
pub struct CompilerOptions {
    options: Vec<String>,
}

impl CompilerOptions {
    pub fn check(&self, opt: String) -> bool {
        self.options.contains(&opt)
    }
}
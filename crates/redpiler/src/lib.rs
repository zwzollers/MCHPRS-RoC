pub mod compile_graph;
pub mod redpiler_graph;
pub mod passes;

use mchprs_blocks::blocks::Block;
use mchprs_blocks::BlockPos;
use mchprs_world::World;
use std::sync::Mutex;
use tracing::warn;


pub fn block_powered_mut(block: &mut Block) -> Option<&mut bool> {
    Some(match block {
        Block::RedstoneComparator { comparator } => &mut comparator.powered,
        Block::RedstoneTorch { lit } => lit,
        Block::RedstoneWallTorch { lit, .. } => lit,
        Block::RedstoneRepeater { repeater } => &mut repeater.powered,
        Block::Lever { lever } => &mut lever.powered,
        Block::StoneButton { button } => &mut button.powered,
        Block::StonePressurePlate { powered } => powered,
        Block::RedstoneLamp { lit } => lit,
        Block::IronTrapdoor { powered, .. } => powered,
        Block::NoteBlock { powered, .. } => powered,
        _ => return None,
    })
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct CompilerOptions {
    /// Enable optimization passes which may significantly increase compile times.
    pub optimize: bool,
    /// Export the graph to a binary format. See the [`redpiler_graph`] crate.
    pub export: bool,
    /// Only flush lamp, button, lever, pressure plate, or trapdoor updates.
    pub io_only: bool,
    /// Update all blocks in the input region after reset.
    pub update: bool,
    /// Export a dot file of the graph after backend compile (backend dependent)
    pub export_dot_graph: bool,
    /// Consider a redstone dot to be an output block (for color screens)
    pub wire_dot_out: bool,
    /// Compile only what is selected in the WorldEdit selection
    pub selection: bool,
    /// Run the verilog through compiler
    pub compile_verilog: bool,
    /// The backend variant to be used after compilation
    pub backend_variant: BackendVariant,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]

pub enum BackendVariant {
    #[default]
    Direct,
    FPGA,
}

impl CompilerOptions {
    pub fn parse(str: &str) -> CompilerOptions {
        let mut co: CompilerOptions = Default::default();
        let options = str.split_whitespace();
        for option in options {
            if option.starts_with("--") {
                match option {
                    "--optimize" => co.optimize = true,
                    "--export" => co.export = true,
                    "--io-only" => co.io_only = true,
                    "--update" => co.update = true,
                    "--export-dot" => co.export_dot_graph = true,
                    "--wire-dot-out" => co.wire_dot_out = true,
                    "--selection" => co.selection = true,
                    "--fpga" => co.backend_variant = BackendVariant::FPGA,
                    "--compile" => co.compile_verilog = true,
                    // FIXME: use actual error handling
                    _ => warn!("Unrecognized option: {}", option),
                }
            } else if let Some(str) = option.strip_prefix('-') {
                for c in str.chars() {
                    let lower = c.to_lowercase().to_string();
                    match lower.as_str() {
                        "o" => co.optimize = true,
                        "e" => co.export = true,
                        "i" => co.io_only = true,
                        "u" => co.update = true,
                        "d" => co.wire_dot_out = true,
                        "s" => co.selection = true,
                        "f" => co.backend_variant = BackendVariant::FPGA,
                        "c" => co.compile_verilog = true,
                        // FIXME: use actual error handling
                        _ => warn!("Unrecognized option: -{}", c),
                    }
                }
            } else {
                // FIXME: use actual error handling
                warn!("Unrecognized option: {}", option);
            }
        }
        co
    }

        pub fn to_str_vec(&self) -> Vec<String> {
        let mut flags = Vec::new();
        let backend = self.backend_variant;
        if self.optimize && backend == BackendVariant::Direct{
            flags.push("    §3- optimize".to_string());
        }
        if self.export && backend == BackendVariant::Direct{
            flags.push("    §3- export".to_string());
        }
        if self.io_only && backend == BackendVariant::Direct{
            flags.push("    §3- io only".to_string());
        }
        if self.update && backend == BackendVariant::Direct{
            flags.push("    §3- update".to_string());
        }
        if self.wire_dot_out && backend == BackendVariant::Direct{
            flags.push("    §3- wire dot out".to_string());
        }
        if self.selection && backend == BackendVariant::Direct{
            flags.push("    §3- selection only".to_string());
        }
        flags
    }

    pub fn fpga() -> CompilerOptions {
        let mut co: CompilerOptions = Default::default();
        co.backend_variant = BackendVariant::FPGA;
        co.compile_verilog = true;
        co.io_only = true;
        co.optimize = true;
        co.selection = true;

        co
    }
}


pub struct CompilerInput<'w, W: World> {
    pub world: &'w Mutex<W>,
    pub bounds: (BlockPos, BlockPos),
}

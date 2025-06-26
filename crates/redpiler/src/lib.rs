mod backend;
mod compile_graph;
mod task_monitor;
// mod debug_graph;
mod passes;

use backend::{BackendDispatcher, JITBackend};
use mchprs_blocks::blocks::Block;
use mchprs_blocks::BlockPos;
use mchprs_world::TickEntry;
use mchprs_world::{for_each_block_mut_optimized, World};
use passes::make_default_pass_manager;
use std::fs::read_dir;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, error, trace, warn};

pub use task_monitor::TaskMonitor;

fn block_powered_mut(block: &mut Block) -> Option<&mut bool> {
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

pub enum BackendStatus {
    Stopped,
    Redpiling,
    Compiling,
    Ready,
    Active
}

impl BackendStatus {
    pub fn to_str(&self) -> String {
        match self {
            BackendStatus::Stopped =>   "§c  Stopped".to_string(),
            BackendStatus::Redpiling => "§eRedpiling".to_string(),
            BackendStatus::Compiling => "§eCompiling".to_string(),
            BackendStatus::Ready =>     "§2    Ready".to_string(),
            BackendStatus::Active =>    "§a   Active".to_string(),
        }
    }
}

pub enum BackendMsg {
    BackendStatus{backend: String, status: BackendStatus}
}

pub struct Backend {
    is_active: bool,
    sender: Sender<BackendMsg>,
    name: String,
    jit: BackendDispatcher,
    options: CompilerOptions,
}

impl Backend {
    pub fn new <W: World>(
        sender: Sender<BackendMsg>,
        name: String,
        plot: String,
        world: &Mutex<W>,
        bounds: (BlockPos, BlockPos),
        options: CompilerOptions,
        ticks: Vec<TickEntry>,
    ) -> Backend{
        debug!("Starting compile");
        let start = Instant::now();

        let input = CompilerInput { world: world, bounds };
        let pass_manager = make_default_pass_manager::<W>();
        let graph = pass_manager.run_passes(&options, &input);

        let mut jit = match options.backend_variant {
            BackendVariant::Direct => BackendDispatcher::DirectBackend(Default::default()),
            BackendVariant::FPGA => BackendDispatcher::FPGABackend(Default::default())
        };

        _ = sender.send(BackendMsg::BackendStatus { backend: name.clone(), status: BackendStatus::Compiling });

        jit.compile(
            graph,
            ticks,
            format!("{plot}/{name}"),
            &options);

        _ = sender.send(BackendMsg::BackendStatus { backend: name.clone(), status: BackendStatus::Ready });
        debug!("Compile completed in {:?}", start.elapsed());

        Backend{ 
            is_active: false,
            sender: sender,
            name: name,
            jit: jit,
            options: options,
        }
    }

    pub fn load(plot: String) -> Vec<Backend> {
        let backend_list = Vec::new();
        let path_str = &format!("FPGA/bin/{plot}");
        let path = Path::new(path_str);

        if path.exists() {
            for backend in read_dir(path).unwrap() {
                
            }
        }
        backend_list
    }

    pub fn options(&self) -> &CompilerOptions {
        &self.options
    }

    pub fn reset<W: World>(&mut self, world: &mut W, bounds: (BlockPos, BlockPos)) {
        let io_only = self.options.io_only;
        self.backend().reset(world, io_only);

        if self.options.update {
            let (first_pos, second_pos) = bounds;
            for_each_block_mut_optimized(world, first_pos, second_pos, |world, pos| {
                let block = world.get_block(pos);
                mchprs_redstone::update(block, world, pos);
            });
        }
        self.options = Default::default();
    }

    fn backend(&mut self) -> &mut BackendDispatcher {
        assert!(
            self.is_active,
            "tried to get redpiler backend when inactive"
        );
        &mut self.jit
    }

    pub fn tick(&mut self) {
        self.backend().tick();
    }

    pub fn tickn(&mut self, ticks: u64) {
        self.backend().tickn(ticks);
    }

    pub fn on_use_block(&mut self, pos: BlockPos) {
        self.backend().on_use_block(pos);
    }

    pub fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool) {
        self.backend().set_pressure_plate(pos, powered);
    }

    pub fn flush<W: World>(&mut self, world: &mut W) {
        let io_only = self.options.io_only;
        self.backend().flush(world, io_only);
    }

    pub fn inspect(&mut self, pos: BlockPos) {
        self.backend().inspect(pos);
    }

    pub fn has_pending_ticks(&mut self) -> bool {
        self.backend().has_pending_ticks()
    }

    pub fn set_rtps(&mut self, rtps: u32) {
        self.backend().set_rtps(rtps);
    }
}

pub struct CompilerInput<'w, W: World> {
    pub world: &'w Mutex<W>,
    pub bounds: (BlockPos, BlockPos),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_options() {
        let input = "-io -u --export";
        let expected_options = CompilerOptions {
            io_only: true,
            optimize: true,
            export: true,
            update: true,
            export_dot_graph: false,
            wire_dot_out: false,
            selection: false,
            compile_verilog: false,
            backend_variant: BackendVariant::default(),
        };
        let options = CompilerOptions::parse(input);

        assert_eq!(options, expected_options);
    }
}

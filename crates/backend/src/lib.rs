pub mod direct;
pub mod fpga;

use mchprs_blocks::BlockPos;
use mchprs_world::TickEntry;
use mchprs_world::{for_each_block_mut_optimized, World};
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::time::Instant;
use tracing::debug;
use fpga::linker::Linker;


use mchprs_redpiler::{
    compile_graph::CompileGraph, 
    CompilerOptions, 
    passes::make_default_pass_manager,
    CompilerInput,
    BackendVariant,
};
use enum_dispatch::enum_dispatch;
use direct::DirectBackend;
use fpga::FPGABackend;

use crate::fpga::compiler::DeviceConfig;


#[enum_dispatch]
pub trait JITBackend {
    fn compile(
        &mut self,
        graph: CompileGraph,
        ticks: Vec<TickEntry>,
        plot: String,
        name: String,
        config: Option<DeviceConfig>,
        options: &CompilerOptions,  
    );
    fn run(&mut self);
    fn stop(&mut self);
    fn tick(&mut self);
    fn tickn(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.tick();
        }
    }
    fn on_use_block(&mut self, pos: BlockPos);
    fn set_pressure_plate(&mut self, pos: BlockPos, powered: bool);
    fn flush<W: World>(&mut self, world: &mut W, io_only: bool);
    fn reset<W: World>(&mut self, world: &mut W, io_only: bool);
    fn has_pending_ticks(&self) -> bool;
    fn inspect(&mut self, pos: BlockPos);
    fn set_rtps(&mut self, rtps: u32);
}

#[enum_dispatch(JITBackend)] 
pub enum BackendDispatcher {
    DirectBackend,
    FPGABackend,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
            BackendStatus::Stopped =>   "&cStopped".to_string(),
            BackendStatus::Redpiling => "&eRedpiling".to_string(),
            BackendStatus::Compiling => "&eCompiling".to_string(),
            BackendStatus::Ready =>     "&2Ready".to_string(),
            BackendStatus::Active =>    "&aActive".to_string(),
        }
    }
}

pub enum BackendMsg {
    BackendStatus{backend: String, status: BackendStatus},
    New{backend: String, options: CompilerOptions},
    Delete{backend: String}
}

pub struct Backend {
    is_active: bool,
    sender: Sender<BackendMsg>,
    pub name: String,
    jit: BackendDispatcher,
    options: CompilerOptions,
}

impl Backend {
    pub fn from_data(plot: (i32,i32), sender: Sender<BackendMsg>, config: DeviceConfig) -> Vec<Backend> {
        let mut backends: Vec<Backend> = Vec::new();
        let path_str = format!("FPGA/bin/{}-{}",plot.0,plot.1);
        let path = Path::new(&path_str);
        if path.is_dir() {

            for entry in fs::read_dir(path).unwrap() {
                let link_path = entry.unwrap().path().join("link.json");
                let links_str = fs::read_to_string(link_path).unwrap();
                let link: Linker = serde_json::from_str(&links_str).unwrap();

                let name = link.name.clone();

                let backend = FPGABackend::from_link_file(link, format!("{}-{}/{}",plot.0,plot.1,name), config.clone());
                let new_sender = sender.clone();
                _ = new_sender.send(BackendMsg::New { backend: name.clone(), options: CompilerOptions::fpga() });
                _ = new_sender.send(BackendMsg::BackendStatus { backend: name.clone(), status: BackendStatus::Ready });
                backends.push(Backend { 
                    is_active: false,
                    sender: sender.clone(),
                    name: name.clone(),
                    jit: BackendDispatcher::FPGABackend(backend),
                    options: CompilerOptions::fpga()  
                });
            }
        }

        backends
    }

    pub fn new <W: World>(
        sender: Sender<BackendMsg>,
        name: String,
        plot: String,
        config: Option<DeviceConfig>,
        world: &Mutex<W>,
        bounds: (BlockPos, BlockPos),
        options: CompilerOptions,
        ticks: Vec<TickEntry>,
    ) -> Backend{

        _ = sender.send(BackendMsg::New { backend: name.clone(), options: options.clone() });

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
            plot,
            name.clone(),
            config,
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
        &mut self.jit
    }

    pub fn run(&mut self) {
        self.backend().run();
        _ = self.sender.send(BackendMsg::BackendStatus { backend: self.name.clone(), status: BackendStatus::Active });
    }

    pub fn stop(&mut self) {
        self.backend().stop();
        _ = self.sender.send(BackendMsg::BackendStatus { backend: self.name.clone(), status: BackendStatus::Ready });
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


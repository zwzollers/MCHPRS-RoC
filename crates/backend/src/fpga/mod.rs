mod assembler;
mod node;
pub mod interface;
pub mod compiler;

use super::JITBackend;
use mchprs_redpiler::compile_graph::{CompileGraph, NodeType};
use crate::CompilerOptions;
use compiler::DeviceConfig;
use mchprs_blocks::blocks::Block;
use mchprs_blocks::BlockPos;
use mchprs_world::World;
use mchprs_world::TickEntry;
use node::{FPGAInputs, FPGAOutputs};
use std::path::Path;


use interface::{Interface, FPGACommand, BinaryIterator};

use std::fs::{remove_dir_all, copy};

#[derive(Default, Debug)]
pub struct FPGABackend {
    fpga: Interface,
    path: String,
    config: DeviceConfig,
    outputs: FPGAOutputs,
    inputs: FPGAInputs,
}

impl JITBackend for FPGABackend {
    fn inspect(&mut self, _pos: BlockPos) {}

    fn reset<W: World>(&mut self, _world: &mut W, _io_only: bool) {}

    fn on_use_block(&mut self, pos: BlockPos) {
        if let Some(input) = self.inputs.get_mut(pos) {
            input.set_state(!input.state);
            self.fpga.send_command(FPGACommand::SetInputs(input.id, 0, input.state));
        }  
    }

    fn set_pressure_plate(&mut self, _pos: BlockPos, _powered: bool) {}

    fn tick(&mut self) {}

    fn flush<W: World>(&mut self, world: &mut W, _io_only: bool) { 
        self.fpga.send_command(FPGACommand::Capture);
        self.fpga.send_command(FPGACommand::GetOutupts);
        let mut output_iter: BinaryIterator = BinaryIterator::new(self.fpga.outputs.clone());
        for (pos, block) in self.outputs.get_blocks_to_change(&mut output_iter) {
            world.set_block(pos, block);
        }
        for (pos, block) in self.inputs.get_blocks_to_change() {
            world.set_block(pos, block);
        }
    }

    fn compile(
        &mut self,
        graph: CompileGraph,
        _ticks: Vec<TickEntry>,
        path: String,
        config: Option<DeviceConfig>,
        _options: &CompilerOptions,
    ) {
        self.config = config.unwrap();
        self.path = path;
        for nodeid in graph.node_indices() {
            let node = &graph[nodeid];
            if let Some((pos, blockid)) = node.block {
                let block = Block::from_id(blockid);
                match node.ty
                {
                    NodeType::Lamp | NodeType::Trapdoor => 
                        self.outputs.add(block, pos),
                    NodeType::Lever | NodeType::Button | NodeType::PressurePlate => 
                        self.inputs.add(block, pos),
                    _ => ()
                }
            }
        }
        println!("generating");
        assembler::generate_verilog(&graph, Path::new(&format!("FPGA/bin/{}/redstone.sv", self.path)));
        println!("create_project");
        self.config.create_project(Path::new(&format!("FPGA/bin/{}/prj/prj.tcl",self.path)), self.outputs.bits as u32, self.inputs.num_inputs);
        println!("compile");
        self.config.compile(Path::new(&format!("FPGA/bin/{}/prj", self.path)));
        println!("done");
        _ = copy(Path::new(&format!("FPGA/bin/{}/prj/RoC.sof", self.path)), Path::new(&format!("FPGA/bin/{}/RoC.sof", self.path)));
        _ = remove_dir_all(Path::new(&format!("FPGA/bin/{}/prj", self.path)));  
    }

    fn run(&mut self) {
        println!("programming");
        self.config.program(Path::new(&format!("FPGA/bin/{}", self.path)));
        println!("serial start");
        self.fpga.outputs = vec![0; self.outputs.get_num_bytes()];
        self.fpga.serial_start(&self.config.command_com, 2500000);
        self.set_rtps(10);
    }

    fn set_rtps(&mut self, rtps: u32) {
        self.fpga.send_command(FPGACommand::SetRTPS(rtps));
    }

    fn has_pending_ticks(&self) -> bool {false}
}
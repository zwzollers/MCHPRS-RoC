mod assembler;
mod node;

use super::JITBackend;
use crate::compile_graph::{CompileGraph, NodeType};
use crate::CompilerOptions;
use fpga::compiler::DeviceConfig;
use mchprs_blocks::blocks::Block;
use mchprs_blocks::BlockPos;
use mchprs_world::World;
use mchprs_world::TickEntry;
use node::{FPGAInputs, FPGAOutputs};
use std::path::Path;


use fpga::interface::{Interface, FPGACommand};
use fpga::BinaryIterator;

use std::fs::{remove_dir_all, copy};

#[derive(Default, Debug)]
pub struct FPGABackend {
    fpga: Interface,
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
        options: &CompilerOptions,
    ) {
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
        assembler::generate_verilog(&graph, Path::new(&format!("FPGA/bin/{path}/redstone.sv")));
        println!("config");
        let config = DeviceConfig::read_config("DE1-SoC").unwrap();
        println!("create_project");
        config.create_project(Path::new(&format!("FPGA/bin/{path}/prj/prj.tcl")), self.outputs.bits as u32, self.inputs.num_inputs);
        println!("compile");
        config.compile(Path::new(&format!("FPGA/bin/{path}/prj")));
        println!("done");
        _ = copy(Path::new(&format!("FPGA/bin/{path}/prj/RoC.sof")), Path::new(&format!("FPGA/bin/{path}/RoC.sof")));
        _ = remove_dir_all(Path::new(&format!("FPGA/bin/{path}/prj")));

        // let parameters = format!("parameter\n\tROC_INPUTS={},\n\tROC_OUTPUTS={};", self.inputs.num_inputs, self.outputs.bits).to_owned();

        // let mut file = File::create(&format!("FPGA/bin/{path}/parameters.sv")).unwrap();
        // match file.write_all(parameters.as_bytes()) {
        //     Err(..) => println!("    Error Writing to file"),
        //     _ => ()
        // }
        
    }

    fn set_rtps(&mut self, rtps: u32) {
        self.fpga.send_command(FPGACommand::SetRTPS(rtps));
    }

    fn has_pending_ticks(&self) -> bool {false}
}
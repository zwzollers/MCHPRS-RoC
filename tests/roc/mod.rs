use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}, process::{Command, ExitCode}, str::FromStr, sync::Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use mchprs_backend::{fpga::{linker::{IntfBlock, Linker}, FPGABackend}, Backend, BackendDispatcher};
use mchprs_blocks::{block_entities::BlockEntity, BlockPos};
use mchprs_redpiler::CompilerOptions;
use mchprs_world::{storage::Chunk, TickEntry, TickPriority, World};
use mchprs_core::plot::worldedit::{schematic, WorldEditClipboard};
use serde::Deserialize;

pub fn run_simulations() {
    let paths = get_test_paths();

    for path_buf in paths {
        let path = path_buf.as_path();
        let link = generate_rs(path);
        let test = parse_test(path);
        for proc in &test.tests {
            let in_tbl = test.get_translation_table(IntfType::Input, &link.inputs);
            let out_tbl = test.get_translation_table(IntfType::Ouptut, &link.outputs);
            generate_tb(path, proc, in_tbl, out_tbl);
            assert!(run_sim(path));
        }
    }
}

fn get_test_paths() -> Vec<PathBuf> {
    let mut list = Vec::new();

    if let Ok(tests) = fs::read_dir("tests/roc/sim_tests") {
        for dir in tests {
            if let Ok(dir) = dir {
                let path = dir.path();
                if path.is_dir() {
                    list.push(path);
                }
            }
        }
    }

    list
}

fn run_sim(path: &Path) -> bool {

    let out = Command::new("cmd")
        .current_dir(path)
        .args(&["/C", "iverilog -g2005-sv -o sim tb.sv roc.sv ../../../../FPGA/src/redstone/components.sv"])
        .output()
        .unwrap();

    let out = Command::new("cmd")
        .current_dir(path)
        .args(&["/C", "vvp sim"])
        .output()
        .unwrap();
    println!("{:#?}", String::from_utf8_lossy(&out.stdout));

    out.status.success()
}

fn generate_tb(path: &Path, proc: &Procedure, in_tbl: Vec<usize>, out_tbl: Vec<usize>) {

    let mut proc_str = "".to_string();
    for step in &proc.procedure {
        proc_str.push_str(&parse_step(step, &in_tbl, &out_tbl));
    }

    let tb = format!("module tb;\n
    parameter OUTPUTS = {out_cnt};
    parameter INPUTS  = {in_cnt};

    //test 2

    reg tick = 0;
    wire[OUTPUTS-1:0] outputs;
    reg[INPUTS-1:0]   inputs = 0;

    RoC #(
        .OUTPUTS(OUTPUTS),
        .INPUTS(INPUTS)
    ) redstone (
        .tick(tick),
        .outputs(outputs),
        .inputs(inputs)
    );

    initial begin
        #2
        {proc}
    end

endmodule: tb",
    out_cnt = out_tbl.len(),
    in_cnt = in_tbl.len(),
    proc = proc_str);

    let mut file = File::create(path.join("tb.sv")).unwrap();
    match file.write(tb.as_bytes()) {
        Err(..) => println!("    Error Writing to file"),
        _ => ()
    }
}

fn parse_step(step: &String, in_tbl: &Vec<usize>, out_tbl: &Vec<usize>) -> String {
    let args: Vec<&str> = step.split(' ').collect();
    let mut input = "".to_string();
    for i in in_tbl {
        input.push(args[1].as_bytes()[*i] as char);
    }
    let mut output = "".to_string();
    for i in out_tbl {
        output.push(args[2].as_bytes()[*i] as char);
    }
    let step = format!("
        if (outputs !== {o_size}'b{o}) begin
            $display(\"FAILURE {o_size}'b%b != {o_size}'b{o}\", outputs);
            $fatal;
		end 
        else
			$display(\"OK\");
        inputs = {i_size}'b{i};
        #1
        tick = ~tick;
        #1
        tick = ~tick;
        ", o_size = out_tbl.len(), o = output, i_size = in_tbl.len(), i = input);

    step
}

fn generate_rs(path: &Path) -> Linker {
    let mut co = CompilerOptions::fpga();
    co.sim_test = true;
    let (sender, _receive) = mpsc::channel();
    let schem = schematic::load_schematic(&format!("{}/build.schem", path.to_str().unwrap()), true).unwrap();
    let mut world = TestWorld::new(10);
    world.paste_schem(&schem);
    let ticks = world.to_be_ticked.clone();
    let m_world = Mutex::new(world);
    let bounds = (BlockPos::new(0, 0, 0), BlockPos::new(32, 32, 32));
    
    let mut b = Backend::new(
        sender,
        path.to_str().unwrap().to_string(),
        "0,0".to_string(),
        None,
        &m_world,
        bounds,
        co.clone(),
        ticks,
    );

    match b.backend() {
        BackendDispatcher::FPGABackend(fpga) => {
            fpga.link.clone()
        }
        _ => panic!("no linker in backend")
    }
}

#[derive(Default, Debug, Deserialize)]
struct Test {
    name: String,
    description: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    tests: Vec<Procedure>,
}

enum IntfType {
    Input,
    Ouptut
}

impl Test {
    fn get_translation_table(&self, ty: IntfType, intf: &Vec<IntfBlock>) -> Vec<usize> {
        let mut table = Vec::new();
        for block in intf {
            let intf_pos = block.pos;
            let sim_blocks = match ty {
                IntfType::Input => &self.inputs,
                IntfType::Ouptut => &self.outputs,
            };
            for i in 0..sim_blocks.len() {
                let sim_pos = BlockPos::from_str(&sim_blocks[i]).unwrap();
                if sim_pos == intf_pos {
                    table.push(i);
                }
            }
        }
        table
    }
}

#[derive(Default, Debug, Deserialize)]
struct Procedure {
    name: String,
    description: String,
    procedure: Vec<String>,
}

fn parse_test(path: &Path) -> Test {
    let test_str = fs::read_to_string(path.join("test.json")).unwrap();
    let test: Test = serde_json::from_str(&test_str).unwrap();

    test
}

#[derive(Clone, Debug)]
pub struct TestWorld {
    chunks: Vec<Chunk>,
    to_be_ticked: Vec<TickEntry>,
    size: i32,
}

impl TestWorld {
    /// Create a new square world for testing with a size in chunks
    pub fn new(size: i32) -> TestWorld {
        let mut chunks = Vec::new();
        for x in 0..size {
            for z in 0..size {
                chunks.push(Chunk::empty(x, z, size as usize));
            }
        }
        TestWorld {
            chunks,
            to_be_ticked: Vec::new(),
            size,
        }
    }

    pub fn paste_schem(&mut self, cb: &WorldEditClipboard) {
        let offset_x = 0;
        let offset_y = 0;
        let offset_z = 0;
        let mut i = 0;
        // This can be made better, but right now it's not D:
        let x_range = offset_x..offset_x + cb.size_x as i32;
        let y_range = offset_y..offset_y + cb.size_y as i32;
        let z_range = offset_z..offset_z + cb.size_z as i32;

        let entries = cb.data.entries();
        // I have no clue if these clones are going to cost anything noticeable.
        'top_loop: for y in y_range {
            for z in z_range.clone() {
                for x in x_range.clone() {
                    if i >= entries {
                        break 'top_loop;
                    }
                    let entry = cb.data.get_entry(i);
                    i += 1;
                    if entry == 0 {
                        continue;
                    }
                    self.set_block_raw(BlockPos::new(x, y, z), entry);
                }
            }
        }

        for (pos, block_entity) in &cb.block_entities {
            let new_pos = BlockPos {
                x: pos.x + offset_x,
                y: pos.y + offset_y,
                z: pos.z + offset_z,
            };
            self.set_block_entity(new_pos, block_entity.clone());
        }
    }

    fn get_chunk_index_for_chunk(&self, chunk_x: i32, chunk_z: i32) -> usize {
        (chunk_x * self.size + chunk_z).unsigned_abs() as usize
    }

    fn get_chunk_index_for_block(&self, block_x: i32, block_z: i32) -> Option<usize> {
        let chunk_x = block_x >> 4;
        let chunk_z = block_z >> 4;
        if chunk_x >= self.size || chunk_z >= self.size || chunk_x < 0 || chunk_z < 0 {
            return None;
        }
        Some(((chunk_x * self.size) + chunk_z).unsigned_abs() as usize)
    }
}

impl World for TestWorld {
    /// Returns the block state id of the block at `pos`
    fn get_block_raw(&self, pos: BlockPos) -> u32 {
        let chunk_index = match self.get_chunk_index_for_block(pos.x, pos.z) {
            Some(idx) => idx,
            None => return 0,
        };
        let chunk = &self.chunks[chunk_index];
        chunk.get_block((pos.x & 0xF) as u32, pos.y as u32, (pos.z & 0xF) as u32)
    }

    /// Sets a block in storage. Returns true if a block was changed.
    fn set_block_raw(&mut self, pos: BlockPos, block: u32) -> bool {
        let chunk_index = match self.get_chunk_index_for_block(pos.x, pos.z) {
            Some(idx) => idx,
            None => return false,
        };

        // Check to see if block is within height limit
        if pos.y >= self.size * 16 || pos.y < 0 {
            return false;
        }

        let chunk = &mut self.chunks[chunk_index];
        chunk.set_block(
            (pos.x & 0xF) as u32,
            pos.y as u32,
            (pos.z & 0xF) as u32,
            block,
        )
    }

    fn delete_block_entity(&mut self, pos: BlockPos) {
        let chunk_index = match self.get_chunk_index_for_block(pos.x, pos.z) {
            Some(idx) => idx,
            None => return,
        };
        let chunk = &mut self.chunks[chunk_index];
        chunk.delete_block_entity(BlockPos::new(pos.x & 0xF, pos.y, pos.z & 0xF));
    }

    fn get_block_entity(&self, pos: BlockPos) -> Option<&BlockEntity> {
        let chunk_index = match self.get_chunk_index_for_block(pos.x, pos.z) {
            Some(idx) => idx,
            None => return None,
        };
        let chunk = &self.chunks[chunk_index];
        chunk.get_block_entity(BlockPos::new(pos.x & 0xF, pos.y, pos.z & 0xF))
    }

    fn set_block_entity(&mut self, pos: BlockPos, block_entity: BlockEntity) {
        let chunk_index = match self.get_chunk_index_for_block(pos.x, pos.z) {
            Some(idx) => idx,
            None => return,
        };
        let chunk = &mut self.chunks[chunk_index];
        chunk.set_block_entity(BlockPos::new(pos.x & 0xF, pos.y, pos.z & 0xF), block_entity);
    }

    fn get_chunk(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.chunks.get(self.get_chunk_index_for_chunk(x, z))
    }

    fn get_chunk_mut(&mut self, x: i32, z: i32) -> Option<&mut Chunk> {
        let chunk_idx = self.get_chunk_index_for_chunk(x, z);
        self.chunks.get_mut(chunk_idx)
    }

    fn schedule_tick(&mut self, pos: BlockPos, delay: u32, priority: TickPriority) {
        self.to_be_ticked.push(TickEntry {
            pos,
            ticks_left: delay,
            tick_priority: priority,
        });
    }

    fn pending_tick_at(&mut self, pos: BlockPos) -> bool {
        self.to_be_ticked.iter().any(|e| e.pos == pos)
    }
}
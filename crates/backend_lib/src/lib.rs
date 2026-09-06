use mchprs_save_data::plot_data::Tps;
pub use mchprs_redpiler::*;
pub use mchprs_blocks::*;
pub use mchprs_redstone::*;
pub use mchprs_world::*;

use std::any::Any;

#[enum_delegate::register]
pub trait Backend {
    fn init(&mut self) {}
    fn heartbeat(&mut self) {}
    fn delete(&mut self) {}
    fn init_compile_cb(&mut self) -> Option<InitCompileFn> {
        None
    }
    fn compile(&mut self, inputs: &Option<Box<ThreadAny>>, step: &mut CompileStep);

    fn tick(&mut self);
    fn tickn(&mut self, ticks: usize) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    fn run(&mut self) {}
    fn rtps(&mut self, rtps: u32) {}
    fn stop(&mut self) {}

    fn status(&self) -> String;

    fn flush(&mut self) -> Vec<WorldDiff>;
    fn edit(&mut self, edits: Vec<WorldDiff>) -> (Vec<WorldDiff>, bool);

    fn reset(&mut self) {}
    // fn can_edit(&self) -> EditMode;
    // fn edit(Vec<WorldDiff>) -> Bool;
    // fn set_options(&mut self, options: Options);
    // fn save(&mut self, path: &Path);
    // fn load(&mut self, path: &Path) -> bool;
}

pub type ThreadAny = dyn Any + Send + Sync;

pub type InitCompileFn = fn(&dyn World) -> Box<ThreadAny>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendStatus {
    Reset,
    Compiling,
    Stopped,
    Running,
    Error(String),
}

pub enum BackendMessage {
    DeleteAck(String),
    InitCompile(String, InitCompileFn),
    Status(String, BackendStatus),
    Flush(Vec<WorldDiff>),
}

pub enum PlotMessage {
    Delete,
    Run,
    Stop,
    RTPS(Tps),
    Compile(Option<Box<ThreadAny>>),
    Edit(Vec<WorldEdit>),
    Status,
    Reset,
    Flush,
}

pub struct CompileStep {
    pub cur: u32,
    pub total: Option<u32>,
}

pub struct WorldDiff {
    pub pos: BlockPos,
    pub id: u32,
}

pub enum WorldEdit {
    Place { id: u32 },
    Break,
    Use,
}

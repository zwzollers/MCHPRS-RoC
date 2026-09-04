pub use mchprs_world::World;
pub use mchprs_blocks::{BlockPos, blocks::Block};
use std::any::Any;

#[enum_delegate::register]
pub trait Backend {
    fn init(&mut self) {}
    fn heartbeat(&mut self) {}
    fn delete(&mut self) {}
    fn init_compile_cb(&mut self) -> Option<InitCompileFn> {
        None
    }
    fn compile(&mut self, step: Option<usize>) -> (usize, usize);

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

    // fn reset(&mut self);
    // fn can_edit(&self) -> EditMode;
    // fn edit(Vec<WorldDiff>) -> Bool;
    // fn set_options(&mut self, options: Options);
    // fn save(&mut self, path: &Path);
    // fn load(&mut self, path: &Path) -> bool;
}

pub type ThreadAny = dyn Any + Send + Sync;

pub type InitCompileFn = fn(&dyn World) -> Box<ThreadAny>;

pub enum BackendStatus {
    Reset,
    Compiling,
    Stopped,
    Running,
    Error,
}

pub enum BackendMessage {
    Heartbeat,
    Delete(String),
    Compile(Option<Box<ThreadAny>>),
    InitCompile(String, InitCompileFn),
    GetStatus(String),
    Status(String, String),
    Reset,
    GetFlush,
    Flush(Vec<WorldDiff>),
}

pub struct CompileStep {
    pub cur: u32,
    pub total: Option<u32>,
}

pub struct WorldDiff {
    pub pos: BlockPos,
    pub id: u32,
}

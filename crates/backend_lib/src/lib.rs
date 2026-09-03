
#[enum_delegate::register]
pub trait Backend {
    fn init(&mut self) {}
    fn heartbeat(&mut self) {}
    fn delete(&mut self) {}
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

    // fn reset(&mut self);
    //fn can_edit(&self) -> EditMode;
    // fn edit(Vec<WorldDiff>) -> Bool;
    // fn flush() -> Vec<WorldDiff>;
    // fn set_options(&mut self, options: Options);
    //fn save(&mut self, path: &Path);
    //fn load(&mut self, path: &Path) -> bool;
}

pub enum BackendStatus {
    Reset,
    Compiling,
    Stopped,
    Running,
    Error,
}

pub enum BackendMessage {
    Heartbeat,
    Delete,
    Compile,
    GetStatus(String),
    Status(String, String)
}


use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};

use backend_redpiler::{Backend1, Backend2};
use mchprs_backend_lib::*;

pub struct PlotBackend {
    pub manager_thread: JoinHandle<()>,
    pub area: u32,
    pub name: String,
    pub tx: Sender<BackendMessage>,
    pub compile_init_fn: Option<InitCompileFn>,
}

impl PlotBackend {
    pub fn new(name: String, ty: String, bknd_tx: Sender<BackendMessage>) -> Self {
        let bknd_tx = bknd_tx;
        let (tx, bknd_rx) = channel();
        let manager_thread = BackendManager::new(ty, name.clone(), (bknd_tx, bknd_rx));

        PlotBackend {
            manager_thread,
            area: 0,
            name,
            tx,
            compile_init_fn: None,
        }
    }
}

#[enum_delegate::implement(Backend,
    trait Backend {
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
)]
enum Backends {
    Backend1(backend_redpiler::Backend1),
    Backend2(backend_redpiler::Backend2),
}
impl Backends {
    fn new(name: &str) -> Option<Self> {
        match name {
            "Backend1" => Some(Backends::from(Backend1::default())),
            "Backend2" => Some(Backends::from(Backend2::default())),
            _ => None,
        }
    }
}

struct BackendManager {
    bknd: Backends,
    name: String,
    status: BackendStatus,
    channel: (Sender<BackendMessage>, Receiver<BackendMessage>),
    alive: bool,
    compile_step: CompileStep,
    compile_input: Option<Box<ThreadAny>>,
}

impl BackendManager {
    pub fn new(
        ty: String,
        name: String,
        chnl: (Sender<BackendMessage>, Receiver<BackendMessage>),
    ) -> JoinHandle<()> {
        let handle = thread::spawn(move || {
            if let Some(mut bknd) = BackendManager::init(ty.as_str(), name, chnl) {
                while bknd.alive {
                    bknd.update();
                }
            }
        });
        handle
    }

    fn init(
        ty: &str,
        name: String,
        chnl: (Sender<BackendMessage>, Receiver<BackendMessage>),
    ) -> Option<Self> {
        if let Some(mut bknd) = Backends::new(ty) {
            bknd.init();

            // send compile init callback to the plot if the backend needs one
            if let Some(compile_cb) = bknd.init_compile_cb() {
                let _ = chnl
                    .0
                    .send(BackendMessage::InitCompile(name.clone(), compile_cb));
            }

            Some(BackendManager {
                bknd,
                name,
                status: BackendStatus::Reset,
                channel: chnl,
                alive: true,
                compile_step: CompileStep {
                    cur: 0,
                    total: None,
                },
                compile_input: None,
            })
        } else {
            // failed to create backend; let the plot know that it can be deleted
            let _ = chnl.0.send(BackendMessage::Delete(name));
            None
        }
    }

    fn update(&mut self) {
        if let Ok(msg) = self.channel.1.recv() {
            self.process_message(msg);
        }
    }

    fn process_message(&mut self, msg: BackendMessage) {
        match msg {
            BackendMessage::Heartbeat => {
                self.bknd.heartbeat();
            }
            BackendMessage::Delete(_) => {
                self.bknd.delete();
                self.alive = false;
            }
            BackendMessage::Compile(data) => {
                self.status = BackendStatus::Compiling;
                self.compile_step.cur = 0;
                self.compile_step.total = None;
                self.compile_input = data;
                println!("{:?}", self.compile_input);
            }
            BackendMessage::GetStatus(uname) => {
                let _ = self
                    .channel
                    .0
                    .send(BackendMessage::Status(uname, self.bknd.status()));
            }
            _ => (),
        }
    }
}

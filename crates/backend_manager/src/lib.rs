use std::{sync::mpsc::{Receiver, Sender, channel}, thread::{self, JoinHandle}};

use backend_redpiler::{Backend1, Backend2};
use mchprs_backend_lib::{Backend, BackendMessage, BackendStatus};

pub struct PlotBackend {
    pub manager_thread: JoinHandle<()>,
    pub area: u32,
    pub name: String,
    pub tx: Sender<BackendMessage>,
}

impl PlotBackend {
    pub fn new(name: String, ty: String, bknd_tx: Sender<BackendMessage>) -> Self {
        let bknd_tx = bknd_tx;
        let (tx, bknd_rx) = channel();
        let manager_thread = BackendManager::new(ty, (bknd_tx, bknd_rx));

        PlotBackend { manager_thread, area: 0, name, tx }
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
            _ => None
        }
    }
}

struct BackendManager {
    bknd: Backends,
    status: BackendStatus,
    channel: (Sender<BackendMessage>, Receiver<BackendMessage>),
    alive: bool,
}

impl BackendManager {
    pub fn new(ty: String, chnl: (Sender<BackendMessage>, Receiver<BackendMessage>)) -> JoinHandle<()> {
        let handle = thread::spawn(move || {
            let mut bknd = BackendManager::init(ty, chnl);

            while bknd.alive {
                if let Ok(msg) = bknd.channel.1.recv() {
                    bknd.process_message(msg);
                }
            }
            let _ = bknd.channel.0.send(BackendMessage::Delete);
        });

        handle
    }

    fn init(ty: String, chnl: (Sender<BackendMessage>, Receiver<BackendMessage>)) -> Self {
        let mut bknd = match ty.as_str() {
            "Backend1" => Backends::from(Backend1::default()),
            "Backend2" => Backends::from(Backend2::default()),
            _          => Backends::from(Backend1::default()),
        };
        bknd.init();

        let status = BackendStatus::Reset;

        BackendManager { 
            bknd, 
            status, 
            channel: chnl,
            alive: true,
        }
    }

    fn process_message(&mut self, msg: BackendMessage) {
        match msg {
            BackendMessage::Heartbeat => {
                self.bknd.heartbeat();
            }
            BackendMessage::Delete => {
                self.bknd.delete();
                self.alive = false;
            }
            BackendMessage::Compile => {
                let cur_step = None;

                let (cur_step, total_steps) = self.bknd.compile(cur_step);
            }
            BackendMessage::GetStatus(uname) => {
                let _ = self.channel.0.send(BackendMessage::Status(uname, self.bknd.status()));
            }
            _ => ()
        }
    }
}
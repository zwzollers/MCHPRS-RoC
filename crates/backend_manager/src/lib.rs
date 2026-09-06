use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{
    default,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use backend_redpiler::{Backend1, Backend2};

use mchprs_backend_lib::*;
use mchprs_save_data::plot_data::Tps;

pub struct PlotBackend {
    pub area: u32,
    pub name: String,
    pub status: BackendStatus,
    pub tx: Sender<PlotMessage>,
    pub compile_init_fn: Option<InitCompileFn>,
}

impl PlotBackend {
    pub fn new(name: String, ty: String, bknd_tx: Sender<BackendMessage>) -> Self {
        let (plot_tx, bknd_rx) = unbounded();
        BackendManager::new(ty, name.clone(), bknd_tx, bknd_rx);

        PlotBackend {
            area: 0,
            name,
            status: BackendStatus::Reset,
            tx: plot_tx,
            compile_init_fn: None,
        }
    }
}

#[enum_delegate::implement(Backend,
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
)]
enum Backends {
    Backend1(backend_redpiler::Backend1),
    Backend2(backend_redpiler::Backend2),
}
impl Backends {
    fn new(name: &str) -> Option<Self> {
        match name {
            "rp" => Some(Backends::from(Backend1::default())),
            "roc" => Some(Backends::from(Backend2::default())),
            _ => None,
        }
    }
}

struct BackendManager {
    // generic
    bknd: Backends,
    name: String,
    status: BackendStatus,
    alive: bool,
    // plot channel
    tx: Sender<BackendMessage>,
    rx: Receiver<PlotMessage>,
    // Compile info
    compile_step: CompileStep,
    compile_input: Option<Box<ThreadAny>>,
    // Run info
    next_tick: Instant,
    tick_time: Option<Duration>,
}

impl BackendManager {
    pub fn new(
        ty: String,
        name: String,
        tx: Sender<BackendMessage>,
        rx: Receiver<PlotMessage>,
    ) -> Option<JoinHandle<()>> {
        if let Some(mut bknd) = Backends::new(ty.as_str()) {
            bknd.init();

            // send compile init callback to the plot if the backend needs one
            if let Some(compile_cb) = bknd.init_compile_cb() {
                let _ = tx.send(BackendMessage::InitCompile(name.clone(), compile_cb));
            }

            let mut bknd_mgr = BackendManager {
                bknd,
                name,
                status: BackendStatus::Reset,
                tx,
                rx,
                alive: true,
                compile_step: CompileStep {
                    cur: 0,
                    total: None,
                },
                compile_input: None,
                next_tick: Instant::now(),
                tick_time: None,
            };

            let handle = thread::spawn(move || {
                while bknd_mgr.alive {
                    bknd_mgr.update();
                }
            });
            Some(handle)
        } else {
            None
        }
    }

    fn update(&mut self) {
        match &self.status {
            BackendStatus::Reset | BackendStatus::Stopped => {
                if let Ok(msg) = self.rx.recv() {
                    self.process_message(msg);
                }
            }
            BackendStatus::Compiling => {
                if Some(self.compile_step.cur) == self.compile_step.total {
                    self.status = BackendStatus::Stopped;
                } else {
                    self.bknd
                        .compile(&self.compile_input, &mut self.compile_step);
                }
            }
            BackendStatus::Running => {
                if let Some(tick_time) = self.tick_time {
                    // process as many message as possible before the next ticks
                    // at least 1 message with get processed here so there is no potiental of starving the channel
                    while let Ok(msg) = self.rx.recv_deadline(self.next_tick) {
                        self.process_message(msg);
                    }

                    self.bknd.tick();

                    let now = Instant::now();
                    self.next_tick = if now + tick_time < self.next_tick {
                        // we cannot keep up with RTPS make sure the next_tick time doesnt fall too far behind
                        now
                    } else {
                        self.next_tick + tick_time
                    }
                } else {
                    if let Ok(msg) = self.rx.recv() {
                        self.process_message(msg);
                    }
                }
            }
            BackendStatus::Error(msg) => {}
        }
    }

    fn process_message(&mut self, msg: PlotMessage) {
        match msg {
            PlotMessage::Delete => {
                self.bknd.delete();
                self.alive = false;
            }
            PlotMessage::Compile(data) => {
                self.status = BackendStatus::Compiling;
                self.compile_step.cur = 0;
                self.compile_step.total = None;
                self.compile_input = data;
            }
            PlotMessage::Status => {
                let _ = self.tx.send(BackendMessage::Status(
                    self.name.clone(),
                    self.status.clone(),
                ));
            }
            PlotMessage::Flush => {
                let diff = self.bknd.flush();
                let _ = self.tx.send(BackendMessage::Flush(diff));
            }
            PlotMessage::RTPS(tps) => {
                self.next_tick = Instant::now();
                match tps {
                    Tps::Limited(rtps) => {
                        self.tick_time = Some(Duration::from_nanos((1000000000 / rtps) as u64));
                    }
                    Tps::Unlimited => {
                        self.tick_time = Some(Duration::default());
                    }
                }
            }
            PlotMessage::Run => {
                if self.status == BackendStatus::Stopped {
                    self.next_tick = Instant::now();
                    self.status = BackendStatus::Running;
                }
            }
            PlotMessage::Stop => {
                if self.status == BackendStatus::Running {
                    self.status = BackendStatus::Stopped;
                }
            }
            PlotMessage::Reset => {
                self.bknd.reset();
            }
            _ => (),
        }
    }
}

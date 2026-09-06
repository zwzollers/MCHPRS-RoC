use mchprs_backend_lib::*;

#[derive(Default)]
pub struct Backend1 {
    placed: bool,
}
impl Backend for Backend1 {
    fn init(&mut self) {
        self.placed = true
    }
    fn heartbeat(&mut self) {
        println!("Backend: heartbeat");
    }

    fn delete(&mut self) {
        println!("Backend: delete");
    }

    fn init_compile_cb(&mut self) -> Option<InitCompileFn> {
        println!("test");
        Some(|w: &dyn World| -> Box<ThreadAny> {
            println!("Running Callback");
            Box::new(5)
        })
    }

    fn compile(&mut self, inputs: &Option<Box<ThreadAny>>, step: &mut CompileStep) {
        step.total = Some(0);
    }

    fn tick(&mut self) {
        self.placed = !self.placed;
        //println!("ticked");
    }

    fn status(&self) -> String {
        "hello from backend1".into()
    }

    fn flush(&mut self) -> Vec<WorldDiff> {
        if self.placed {
            vec![WorldDiff {
                pos: BlockPos {
                    x: 50,
                    y: 50,
                    z: 50,
                },
                id: Block::GrayConcrete.get_id(),
            }]
        } else {
            vec![WorldDiff {
                pos: BlockPos {
                    x: 50,
                    y: 50,
                    z: 50,
                },
                id: Block::Air.get_id(),
            }]
        }
    }

    fn edit(&mut self, edits: Vec<WorldDiff>) -> (Vec<WorldDiff>, bool) {
        todo!()
    }
}

#[derive(Default)]
pub struct Backend2 {}
impl Backend for Backend2 {
    fn compile(&mut self, inputs: &Option<Box<ThreadAny>>, step: &mut CompileStep) {}

    fn tick(&mut self) {
        todo!()
    }

    fn status(&self) -> String {
        "hello from backend2".into()
    }

    fn flush(&mut self) -> Vec<WorldDiff> {
        todo!()
    }

    fn edit(&mut self, edits: Vec<WorldDiff>) -> (Vec<WorldDiff>, bool) {
        todo!()
    }
}

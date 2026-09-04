use mchprs_backend_lib::*;

#[derive(Default)]
pub struct Backend1 {}
impl Backend for Backend1 {
    fn init(&mut self) {}
    fn heartbeat(&mut self) {
        println!("Backend: heartbeat");
    }

    fn delete(&mut self) {
        println!("Backend: delete");
    }

    fn init_compile_cb(&mut self) -> Option<InitCompileFn> {
        Some(|w: &dyn World| -> Box<ThreadAny> {
            println!("Running Callback");
            Box::new(5)
        })
    }

    fn compile(&mut self, step: Option<usize>) -> (usize, usize) {
        if step.is_none() {
            return (0, 10);
        }

        match step.unwrap() {
            0 => (0, 10),
            _ => (0, 0),
        }
    }

    fn tick(&mut self) {
        todo!()
    }

    fn status(&self) -> String {
        "hello from backend1".into()
    }
}

#[derive(Default)]
pub struct Backend2 {}
impl Backend for Backend2 {
    fn compile(&mut self, step: Option<usize>) -> (usize, usize) {
        if step.is_none() {
            return (0, 10);
        }

        match step.unwrap() {
            0 => (0, 10),
            _ => (0, 0),
        }
    }

    fn tick(&mut self) {
        todo!()
    }

    fn status(&self) -> String {
        "hello from backend2".into()
    }
}

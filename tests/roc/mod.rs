use std::{fs, path::{Path, PathBuf}, process::{Command, ExitCode}};

use mchprs_backend::Backend;

pub fn run_simulations() {
    println!("hello_world");
    assert!(true);

    let path = std::env::current_dir().unwrap();
    println!("The current directory is {}", path.display());

    let paths = get_test_paths();

    for path_buf in paths {
        let path = path_buf.as_path();
        println!("{path:?}");
        generate_rs(path);
        //let test = parse_test(path);
        //generate_tb(path, test);
        assert!(run_sim(path));
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
        .args(&["/C", "iverilog -o sim tb.sv"])
        .output()
        .unwrap();
    println!("{:?}", &out.status);
    println!("{:?}", String::from_utf8_lossy(&out.stdout));
    println!("{:?}", String::from_utf8_lossy(&out.stderr));

    let out = Command::new("cmd")
        .current_dir(path)
        .args(&["/C", "vvp sim"])
        .output()
        .unwrap();
    println!("{:?}", out.status);
    println!("{:?}", String::from_utf8_lossy(&out.stdout));
    println!("{:?}", String::from_utf8_lossy(&out.stderr));

    out.status.success() 
}

fn generate_tb(path: &Path, test: Test) {

}

fn generate_rs(path: &Path) {

    
}

struct Test {
    name: String,
    desc: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
    tests: Vec<Procedure>,
}

struct Procedure {
    name: String,
    desc: String,
    proc: Vec<String>,
}

// fn parse_test(path: &Path) -> Test {

// }
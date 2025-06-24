// This build script is used to move all fpga related files to the release folder

use std::{env, fs, path::PathBuf};
use serde_json::{from_str, Value};

const COPY_DIR: &'static str = "FPGA/";

fn main() {
    // Read the release_manifest.json file
    let manifest_file = fs::read_to_string("FPGA/release_manifest.json").unwrap();
    let manifest_json: Value = from_str(&manifest_file).expect("Bad JSON");
    let manifest = manifest_json.get("manifest").unwrap().as_array().unwrap();

    // Request the output directory
    let out_dir = env::var("PROFILE").unwrap();
    let out_dir = PathBuf::from(format!("target/{}/{}", out_dir, COPY_DIR));

    let from =  PathBuf::from(COPY_DIR);

    for file in manifest {
        let path = file.as_str().unwrap();
        let file_path = out_dir.join(std::path::Path::new(path));

        if path.ends_with("/") {
            std::fs::create_dir_all(file_path).unwrap();
        } 
        else {
            let prefix = file_path.parent().unwrap();
            let from_path = from.join(std::path::Path::new(path));

            //copy files in the manifest to the realease folder
            std::fs::create_dir_all(prefix).unwrap();
            fs::copy(&from_path, file_path).unwrap();
        }  
    }
}
use fs_extra::{dir, move_items};

pub fn move_verilog_to_release() {
    let options = dir::CopyOptions::new();

    let mut from_paths = Vec::new();
    from_paths.push("source/dir1");
    from_paths.push("source/file.txt");
    move_items(&from_paths, "target", &options);
}
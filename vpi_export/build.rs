extern crate bindgen;

use std::path::PathBuf;
use std::process::Command;

fn main() {
    Command::new("sh")
        .current_dir("iverilog")
        .arg("autoconf.sh")
        .output()
        .expect("failed to execute process");

    Command::new("./configure")
        .current_dir("iverilog")
        .output()
        .expect("failed to execute process");

    let out_path = PathBuf::from("./src/");
    let svdpi_bindings = bindgen::Builder::default()
        .header("iverilog/vpi_user.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_default(true)
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");

    svdpi_bindings
        .write_to_file(out_path.join("vpi_user.rs"))
        .expect("Couldn't write bindings!");
}

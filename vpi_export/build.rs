use std::{
    env,
    path::{Path, PathBuf},
};

use git2::Repository;

#[cfg(feature = "iverilog-vpi-user")]
fn iverilog(out_dir: &Path) {
    use std::process::Command;
    let url = "https://github.com/steveicarus/iverilog.git";
    let path = out_dir.join("iverilog");

    if !path.is_dir() {
        match Repository::clone(url, &path) {
            Err(e) => panic!("failed to clone: {}", e),
            _ => (),
        };
    }

    Command::new("sh")
        .args(["autoconf.sh"])
        .current_dir(&path)
        .output()
        .unwrap();

    Command::new("./configure")
        .current_dir(&path)
        .output()
        .unwrap();

    let bindings = bindgen::Builder::default()
        .header(path.join("vpi_user.h").to_string_lossy())
        .use_core()
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("vpi_user.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(not(feature = "iverilog-vpi-user"))]
fn ghdl(out_dir: &Path) {
    let url = "https://github.com/ghdl/ghdl.git";
    let path = out_dir.join("ghdl");

    if !path.is_dir() {
        if let Err(e) = Repository::clone(url, &path) {
            panic!("failed to clone: {}", e);
        };
    }

    let bindings = bindgen::Builder::default()
        .header(path.join("src/grt/vpi_user.h").to_string_lossy())
        .use_core()
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("vpi_user.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    #[cfg(not(feature = "iverilog-vpi-user"))]
    ghdl(&out_dir);
    #[cfg(feature = "iverilog-vpi-user")]
    iverilog(&out_dir);
}

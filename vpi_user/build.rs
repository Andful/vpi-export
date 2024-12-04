use std::{
    env,
    path::{Path, PathBuf},
};

fn iverilog(out_dir: &Path) {
    let repo = PathBuf::from("repos/iverilog");
    std::fs::copy(repo.join("vpi_user.h"), out_dir.join("vpi_user.h")).unwrap();
    std::fs::copy(repo.join("_pli_types.h.in"), out_dir.join("_pli_types.h")).unwrap();
    let bindings = bindgen::Builder::default()
        .header(out_dir.join("vpi_user.h").to_string_lossy())
        .use_core()
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("vpi_user.rs"))
        .expect("Couldn't write bindings!");
}

fn verilator(out_dir: &Path) {
    let bindings = bindgen::Builder::default()
        .header("repos/verilator/include/vltstd/vpi_user.h")
        .use_core()
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("vpi_user.rs"))
        .expect("Couldn't write bindings!");
}

fn ghdl(out_dir: &Path) {
    let bindings = bindgen::Builder::default()
        .header("repos/ghdl/src/grt/vpi_user.h")
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
    if cfg!(feature = "ghdl") {
        ghdl(&out_dir);
    } else if cfg!(feature = "iverilog") {
        iverilog(&out_dir);
    } else {
        verilator(&out_dir);
    }
}

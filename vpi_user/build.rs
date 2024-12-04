use std::{
    env,
    path::{Path, PathBuf},
};

fn iverilog(out_dir: &Path) {
    use std::process::Command;

    let o = Command::new("sh")
        .args(["autoconf.sh"])
        .current_dir(std::env::current_dir().unwrap().join("iverilog"))
        .output()
        .unwrap();
    
    assert!(o.status.success(), "{}", String::from_utf8(o.stderr).unwrap());

        let o = Command::new("./configure")
        .current_dir(std::env::current_dir().unwrap().join("iverilog"))
        .output()
        .unwrap();

    assert!(o.status.success(), "{}", String::from_utf8(o.stderr).unwrap());

    let bindings = bindgen::Builder::default()
        .header("iverilog/vpi_user.h")
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
        .header("verilator/include/vltstd/vpi_user.h")
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
        .header("ghdl/src/grt/vpi_user.h")
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
    if cfg!(feature = "verilator") {
        verilator(&out_dir);
    } else if cfg!(feature = "iverilog") {
        iverilog(&out_dir);
    } else {
        ghdl(&out_dir);
    }
}

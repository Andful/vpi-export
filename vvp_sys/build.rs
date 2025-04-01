#![feature(exit_status_error)]

use std::{env, path::PathBuf, process::Command, thread::available_parallelism};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    if !out_dir.join("iverilog").is_dir() {
        copy_dir::copy_dir("iverilog", out_dir.join("iverilog")).unwrap();
    }

    Command::new("sh")
        .args(["autoconf.sh"])
        .current_dir(out_dir.join("iverilog"))
        .output()
        .expect("failed to run autoconf.sh")
        .status
        .exit_ok()
        .unwrap();

    Command::new("./configure")
        .args(["--enable-libvvp"])
        .current_dir(out_dir.join("iverilog"))
        .output()
        .expect("failed to run ./configure")
        .status
        .exit_ok()
        .unwrap();

    Command::new("make")
        .args(["-j", &available_parallelism().unwrap().to_string()])
        .current_dir(out_dir.join("iverilog"))
        .output()
        .expect("failed to run make")
        .status
        .exit_ok()
        .unwrap();

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("iverilog/vvp").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=dylib=vvp");
    //println!("cargo:rustc-link-lib=static=stdc++");
    let bindings = bindgen::Builder::default()
        .header(
            out_dir
                .join(out_dir.join("iverilog/vvp/libvvp.h"))
                .to_string_lossy(),
        )
        .use_core()
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("libvvp.rs"))
        .expect("Couldn't write bindings!");
}

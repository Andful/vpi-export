#![no_std]

use core::ffi::CStr;
use core::panic::PanicInfo;

extern crate vpi_export;
use vpi_export::{println, vpi_export};

#[vpi_export]
fn print_rust(s: &CStr) {
    println(s);
}

/*#[no_mangle]
pub extern "C" fn rust_eh_personality() {
    vpi_export::eh_personality();
}*/

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

//#![no_std]

extern crate vpi_export;
use vpi_export::vpi_export;

#[vpi_export]
fn test1(i: i32) {
    println!("hi {i}")
}

#[vpi_export]
fn test2(i: i32) {
    println!("hi {i}")
}

use vpi_export::vpi_export;

#[vpi_export]
fn test1(i: i32) {
    println!("hi {i}")
}

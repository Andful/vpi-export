extern crate vpi_export;
use vpi_export::{vpi_export, println, BitArray};

#[vpi_export]
fn bitvec(s: BitArray::<[u32, 3]>) {
    println!("{:?}", s);
}


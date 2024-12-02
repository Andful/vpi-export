extern crate vpi_export;
use vpi_export::{bitvec, vpi_task, BitVector, InputOutput, Output};

#[vpi_task]
fn bitvec(mut s: InputOutput<BitVector<32>>, mut i: Output<i32>) {
    //println!("{:?}", s.0.len());
    println!("{:?}", *s);
    *s = bitvec!("30'b001").concat(bitvec!("2'b10")).into();
    *i = 55;
}

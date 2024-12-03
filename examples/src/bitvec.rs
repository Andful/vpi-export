extern crate vpi_export;
use vpi_export::{bitvec, vpi_task, BitVector, InputOutput, Output};

#[vpi_task]
fn test2(mut s: InputOutput<BitVector<32>>, mut i: Output<i32>) {
    println!("{:?}", *s);
    *s = bitvec!("30'b001").concat(bitvec!("2'b10")).into();
    *i = 55;
}

#[vpi_task]
fn bitvec(mut a: Output<BitVector<30>>, mut b: Output<BitVector<2>>, mut conc: Output<BitVector<32>>) {
    *a = bitvec!("30'b001");
    *b = bitvec!("2'b10");
    *conc = (*a).clone().concat((*b).clone()).into();
}

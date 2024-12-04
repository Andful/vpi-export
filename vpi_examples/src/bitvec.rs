use vpi_export::{bitvec, vpi_task, BitVector, Output};

#[vpi_task]
fn bitvec(
    mut a: Output<BitVector<30>>,
    mut b: Output<BitVector<2>>,
    mut conc: Output<BitVector<32>>,
) {
    *a = bitvec!("30'b001");
    *b = bitvec!("2'b10");
    *conc = (*a).clone().concat((*b).clone()).into();
}

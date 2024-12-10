use vpi_export::{bitvec, vpi_task, BitVector, Handle};

#[vpi_task]
fn bitvec(
    mut a: Handle<BitVector<30>>,
    mut b: Handle<BitVector<2>>,
    mut conc: Handle<BitVector<32>>,
) -> vpi_export::Result<()> {
    *a.as_mut().unwrap() = bitvec!("30'b001");
    *b.as_mut().unwrap() = bitvec!("2'b10");
    *conc.as_mut().unwrap() = (*a.as_ref().unwrap())
        .clone()
        .concat((*b.as_ref().unwrap()).clone())
        .into();
    vpi_export::on_value_change(&b, move || {
        *a.as_mut().unwrap() = bitvec!("30'b010");
        println!("Hello");
    });
    Ok(())
}

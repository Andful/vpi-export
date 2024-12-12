use vpi_export::{bitvec, finish, get_time, on_delay, vpi_module, vpi_task, BitVector, Handle};

#[vpi_module(main)]
fn top(clk: Handle<BitVector<1>>) {
    on_delay(100, || {
        looping(0, clk);
    });
}

fn looping(i: u64, mut clk: Handle<BitVector<1>>) {
    if i > 100 {
        finish();
    }
    {
        *clk.borrow_mut().unwrap() = if i % 2 == 0 {
            bitvec!("1'b0")
        } else {
            bitvec!("1'b1")
        };
    }
    println!("Tick {}", get_time());
    on_delay(100, move || {
        looping(i + 1, clk);
    });
}

#[vpi_task]
fn bitvec(
    mut a: Handle<BitVector<30>>,
    mut b: Handle<BitVector<2>>,
    mut conc: Handle<BitVector<32>>,
) -> vpi_export::Result<()> {
    *a.borrow_mut()? = bitvec!("30'b001");
    *b.borrow_mut()? = bitvec!("2'b10");
    *conc.borrow_mut()? = (*a.borrow()?).clone().concat((*b.borrow()?).clone()).into();
    vpi_export::on_value_change(b.clone(), move || {
        *a.borrow_mut().unwrap() = bitvec!("30'b010");
        println!("Hello");
    });
    Ok(())
}

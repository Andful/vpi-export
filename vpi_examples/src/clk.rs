use vpi_export::{finish, get_time, vpi_module, Clk, Handle};

#[vpi_module(main)]
fn top(clk: Handle<Clk>) {
    Clk::on_posedge(clk.clone(), || {
        if get_time() >= 100000 {
            finish();
        }
        println!("Hi from Rust at {}", get_time());
    });
    Clk::start(clk, 10, 100).unwrap();
}

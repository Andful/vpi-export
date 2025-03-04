#![feature(coroutines, coroutine_trait, stmt_expr_attributes)]

use vpi_export::{finish, vpi_top, Clk, Handle};

use std::ops::{Coroutine, CoroutineState};
use std::pin::pin;

#[vpi_top]
fn top(clk: Handle<Clk>, mut rst: Handle<bool>, mut en: Handle<bool>, count: Handle<u32>) {
    let mut coroutine = #[coroutine]
    move || {
        *rst.borrow_mut().unwrap() = true;
        *en.borrow_mut().unwrap() = false;
        yield;
        *rst.borrow_mut().unwrap() = false;
        yield;
        println!("count: {}", *count.borrow().unwrap());
        yield;
        *en.borrow_mut().unwrap() = true;
        println!("count: {}", *count.borrow().unwrap());
        yield;
        println!("count: {}", *count.borrow().unwrap());
        yield;
        for _ in 0..100 {
            println!("count: {}", *count.borrow().unwrap());
            yield;
        }
    };
    Clk::on_posedge(clk.clone(), move || match pin!(&mut coroutine).resume(()) {
        CoroutineState::Yielded(()) => {}
        CoroutineState::Complete(()) => {
            finish();
        }
    });
    Clk::start(clk, 10, 100).unwrap();
}

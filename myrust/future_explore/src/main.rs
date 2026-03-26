// future_explore — Idiomatic async Rust, then explore via compiler output
//
// Write normal async code. Use `make all-ir` to see what the compiler
// actually generates (state machines, poll loops, discriminants).
//
// Build: make run | make all-ir (see Makefile)

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// ── Idiomatic async functions — just normal Rust ──

async fn add_one(x: u32) -> u32 {
    x + 1
}

async fn double(x: u32) -> u32 {
    x * 2
}

// Single await: compiler generates a 2-state machine
async fn compute(x: u32) -> u32 {
    let a = add_one(x).await;
    double(a).await
}

// Multiple awaits: compiler generates a multi-state machine
async fn pipeline(x: u32) -> String {
    let step1 = add_one(x).await;
    let step2 = double(step1).await;
    let step3 = add_one(step2).await;
    format!("pipeline({}) = {}", x, step3)
}

// Nested async: compiler flattens into one state machine
async fn nested(x: u32) -> u32 {
    let a = compute(x).await;
    let b = compute(a).await;
    a + b
}

// ── Minimal block_on executor ──

fn block_on<F: Future>(mut f: F) -> F::Output {
    const VTABLE: RawWakerVTable =
        RawWakerVTable::new(|p| RawWaker::new(p, &VTABLE), |_| {}, |_| {}, |_| {});
    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
    let mut cx = Context::from_waker(&waker);
    let mut f = unsafe { Pin::new_unchecked(&mut f) };
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => {}
        }
    }
}

// ── Main: run and introspect ──

fn main() {
    // Run the async functions
    println!("compute(5)  = {}", block_on(compute(5)));
    println!("pipeline(3) = {}", block_on(pipeline(3)));
    println!("nested(5)   = {}", block_on(nested(5)));

    // Introspect: how big are these state machines?
    println!("\n--- Future sizes (state machine enum size) ---");
    println!("add_one(0)  = {} bytes", std::mem::size_of_val(&add_one(0)));
    println!("double(0)   = {} bytes", std::mem::size_of_val(&double(0)));
    println!("compute(0)  = {} bytes", std::mem::size_of_val(&compute(0)));
    println!("pipeline(0) = {} bytes", std::mem::size_of_val(&pipeline(0)));
    println!("nested(0)   = {} bytes", std::mem::size_of_val(&nested(0)));

    println!("\n--- What to explore next ---");
    println!("Run `make all-ir` to see compiler output:");
    println!("  build/future_explore.expanded.rs  — async fn desugaring");
    println!("  build/future_explore.hir          — .await → loop+poll");
    println!("  build/future_explore.mir          — real state machine");
}

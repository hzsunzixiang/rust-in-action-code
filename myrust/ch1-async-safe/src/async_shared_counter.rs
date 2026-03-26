// =============================================================================
// async_shared_counter.rs — Shared mutable state across async tasks
//
// Key concepts:
//   - Arc<Mutex<T>>: the standard pattern for shared mutable state
//   - mutex.lock().await: async-aware locking (won't block the thread)
//   - tokio::spawn + move: transfer ownership into async tasks
//
// Run: cargo run --bin async_shared_counter
// =============================================================================

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;

// -----------------------------------------------------------------------------
// Why tokio::sync::Mutex instead of std::sync::Mutex?
//
// std::sync::Mutex::lock() blocks the OS thread while waiting.
// tokio::sync::Mutex::lock().await yields to the async runtime.
//
// Rule of thumb:
//   - Short critical sections → std::sync::Mutex is fine (even in async)
//   - Lock held across .await points → MUST use tokio::sync::Mutex
// -----------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  async Shared Counter — Arc<Mutex<T>>           ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    // Arc: shared ownership (atomic reference counting)
    // Mutex: interior mutability with exclusive access
    let counter = Arc::new(Mutex::new(0i64));

    let mut handles = vec![];

    // Spawn 10 async tasks, each incrementing the counter 100 times
    for task_id in 0..10 {
        // Clone the Arc (cheap: just increments refcount)
        let counter_clone = Arc::clone(&counter);

        let handle = task::spawn(async move {
            // `move` transfers counter_clone into this async block
            for _ in 0..100 {
                // .lock().await — async lock acquisition
                // Returns a MutexGuard that auto-unlocks on drop (RAII)
                let mut num = counter_clone.lock().await;
                *num += 1;
                // MutexGuard dropped here → lock released
            }
            println!("  Task {} finished (100 increments)", task_id);
        });

        handles.push(handle);
    }

    // .await each JoinHandle to wait for task completion
    for handle in handles {
        handle.await.unwrap();
    }

    // Final read — also needs .lock().await
    let final_value = *counter.lock().await;
    println!("\n  Final counter value: {} (expected: 1000)", final_value);
    assert_eq!(final_value, 1000);

    println!("\n══════════════════════════════════════════════════");
    println!("Key takeaways:");
    println!("  • Arc::clone() is cheap — just bumps a refcount");
    println!("  • mutex.lock().await — non-blocking async lock");
    println!("  • MutexGuard auto-releases on drop (RAII)");
    println!("  • You CANNOT access the data without calling .lock()");
    println!("  • Result is always 1000 — no data race, guaranteed!");
    println!("══════════════════════════════════════════════════");
}

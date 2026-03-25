use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{sleep, Duration};

// =============================================================================
// Rust: async / Tasks + No Data Races — Absolute Safety
//
// In C/C++, sharing mutable data across threads is a minefield:
//   - No compiler checks for data races
//   - You manually manage mutexes (and pray you don't forget)
//   - Forgetting a lock = silent corruption, Heisenbugs, undefined behavior
//
// In Rust, the compiler PREVENTS data races at compile time:
//   - `Send` trait: type can be transferred across threads
//   - `Sync` trait: type can be shared (via &T) across threads
//   - Borrow checker: ensures exclusive access to mutable data
//   - Arc<Mutex<T>>: the safe pattern for shared mutable state
// =============================================================================

// ---------------------------------------------------------------------------
// Example 1: Multiple async tasks safely sharing a counter
//
// C++ equivalent would need: std::mutex + std::lock_guard + pray you don't
// forget to lock. In Rust, the compiler won't even let you access the data
// without locking first!
// ---------------------------------------------------------------------------
async fn shared_counter_demo() {
    println!("=== Example 1: Shared Counter (Arc<Mutex<T>>) ===\n");

    // Arc  = Atomic Reference Counting (like std::shared_ptr in C++)
    // Mutex = Mutual Exclusion lock (like std::mutex in C++)
    // Combined: safe shared mutable state across async tasks
    let counter = Arc::new(Mutex::new(0i64));

    let mut handles = vec![];

    // Spawn 10 tasks, each incrementing the counter 100 times
    for task_id in 0..10 {
        let counter_clone = Arc::clone(&counter);

        let handle = task::spawn(async move {
            for _ in 0..100 {
                // In C++: you could forget the lock and cause a data race
                // In Rust: you CANNOT access the data without .lock()
                let mut num = counter_clone.lock().await;
                *num += 1;
                // Lock is automatically released here (RAII, just like C++)
            }
            println!("  Task {} finished", task_id);
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    let final_value = *counter.lock().await;
    println!("\n  Final counter value: {} (expected: 1000)", final_value);
    assert_eq!(final_value, 1000);
    println!("  ✅ No data race! Always exactly 1000.\n");
}

// ---------------------------------------------------------------------------
// Example 2: Producer-Consumer with async channels
//
// Like a thread-safe queue, but the compiler guarantees safety.
// In C++, you'd manually build this with condition_variable + mutex.
// ---------------------------------------------------------------------------
async fn channel_demo() {
    println!("=== Example 2: Async Channel (Producer-Consumer) ===\n");

    // mpsc = Multi-Producer, Single-Consumer channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);

    // Spawn 3 producer tasks
    for producer_id in 0..3 {
        let tx_clone = tx.clone();
        task::spawn(async move {
            for i in 0..3 {
                let msg = format!("Producer {} -> Message {}", producer_id, i);
                // send() takes ownership of msg — no use-after-send possible!
                tx_clone.send(msg).await.unwrap();
                sleep(Duration::from_millis(50)).await;
            }
            println!("  Producer {} done sending", producer_id);
        });
    }

    // Drop the original sender so the channel can close
    // when all cloned senders are dropped
    drop(tx);

    // Consumer: receive all messages
    let mut received = 0;
    while let Some(msg) = rx.recv().await {
        println!("  Received: {}", msg);
        received += 1;
    }

    println!("\n  Total messages received: {} (expected: 9)", received);
    assert_eq!(received, 9);
    println!("  ✅ All messages delivered safely!\n");
}

// ---------------------------------------------------------------------------
// Example 3: Concurrent computation without shared state (the ideal pattern)
//
// Best approach: avoid shared mutable state entirely!
// Each task owns its data, results are collected via join.
// This is IMPOSSIBLE to get wrong — no locks, no races, by design.
// ---------------------------------------------------------------------------
async fn independent_tasks_demo() {
    println!("=== Example 3: Independent Tasks (No Shared State) ===\n");

    let mut handles = vec![];

    // Spawn independent computation tasks
    let datasets = vec![
        vec![1, 2, 3, 4, 5],
        vec![10, 20, 30, 40, 50],
        vec![100, 200, 300, 400, 500],
    ];

    for (i, data) in datasets.into_iter().enumerate() {
        // `data` is MOVED into the task — no aliasing possible!
        let handle = task::spawn(async move {
            // Simulate some async computation
            sleep(Duration::from_millis(100)).await;
            let sum: i64 = data.iter().sum();
            println!("  Task {} computed sum = {}", i, sum);
            sum  // return the result
        });
        handles.push(handle);
    }

    // Collect results from all tasks
    let mut total = 0i64;
    for handle in handles {
        total += handle.await.unwrap();
    }

    println!("\n  Grand total: {} (expected: 1665)", total);
    assert_eq!(total, 1665);
    println!("  ✅ Zero shared state, zero possibility of data race!\n");
}

// ---------------------------------------------------------------------------
// Example 4: Why Rust prevents data races at COMPILE TIME
//
// These commented-out examples show what the compiler REFUSES to compile.
// In C/C++, all of these would compile fine... and crash at runtime.
// ---------------------------------------------------------------------------
fn _compile_time_safety_demo() {
    println!("=== Example 4: Compile-Time Safety (won't compile!) ===\n");

    // --- Case A: Cannot send non-Send types across tasks ---
    // Rc<T> is NOT thread-safe (no atomic refcount), so Rust forbids it:
    //
    //   use std::rc::Rc;
    //   let data = Rc::new(42);
    //   task::spawn(async move {
    //       println!("{}", data);  // ❌ ERROR: Rc<i32> cannot be sent between threads safely
    //   });
    //
    // Fix: use Arc<T> instead (atomic reference counting)

    // --- Case B: Cannot share &mut across tasks ---
    //
    //   let mut data = vec![1, 2, 3];
    //   let r = &mut data;
    //   task::spawn(async move {
    //       r.push(4);  // ❌ ERROR: borrowed data cannot be sent across tasks
    //   });
    //   data.push(5);   // would be a data race if allowed!
    //
    // Fix: use Arc<Mutex<Vec<i32>>>

    // --- Case C: Cannot access Mutex data without locking ---
    //
    //   let m = Mutex::new(vec![1, 2, 3]);
    //   m.push(4);  // ❌ ERROR: no method `push` on Mutex<Vec<i32>>
    //                //    You MUST call m.lock().await.push(4)

    println!("  (See source code comments for examples that won't compile)\n");
}

// ---------------------------------------------------------------------------
// Example 5: RwLock — multiple readers OR one writer (async version)
//
// Like C++ std::shared_mutex, but ENFORCED by the compiler.
// You literally cannot read the data without acquiring a read lock.
// ---------------------------------------------------------------------------
async fn rwlock_demo() {
    println!("=== Example 5: Async RwLock (Multiple Readers / One Writer) ===\n");

    let data = Arc::new(tokio::sync::RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // Spawn multiple readers
    for reader_id in 0..5 {
        let data_clone = Arc::clone(&data);
        handles.push(task::spawn(async move {
            let read_guard = data_clone.read().await;
            println!("  Reader {} sees: {:?}", reader_id, *read_guard);
            // Multiple readers can hold read locks simultaneously
            sleep(Duration::from_millis(50)).await;
        }));
    }

    // Spawn one writer (will wait until all readers release)
    let data_clone = Arc::clone(&data);
    handles.push(task::spawn(async move {
        let mut write_guard = data_clone.write().await;
        write_guard.push(4);
        write_guard.push(5);
        println!("  Writer added [4, 5] -> {:?}", *write_guard);
    }));

    for handle in handles {
        handle.await.unwrap();
    }

    let final_data = data.read().await;
    println!("\n  Final data: {:?}", *final_data);
    println!("  ✅ Readers and writer coordinated safely!\n");
}

// =============================================================================
// Main: Run all demos
// =============================================================================
#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   Rust: async / Tasks + No Data Races — Absolute Safety    ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  In C/C++: data races are undefined behavior (silent bugs) ║");
    println!("║  In Rust:  data races are COMPILE ERRORS (impossible bugs) ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    shared_counter_demo().await;
    channel_demo().await;
    independent_tasks_demo().await;
    _compile_time_safety_demo();
    rwlock_demo().await;

    println!("══════════════════════════════════════════════════════════════");
    println!("Summary: Rust's type system guarantees:");
    println!("  1. No data races (enforced at compile time)");
    println!("  2. No use-after-free across tasks (ownership transfer)");
    println!("  3. No forgotten locks (can't access data without locking)");
    println!("  4. No wrong lock type (Rc vs Arc caught at compile time)");
    println!("  5. Deadlock-free? No — but data-race-free? ABSOLUTELY.");
    println!("══════════════════════════════════════════════════════════════");
}

// =============================================================================
// async_rwlock.rs — Async RwLock: Multiple Readers OR One Writer
//
// Key concepts:
//   - tokio::sync::RwLock: async read-write lock
//   - rwlock.read().await: acquire shared read lock (multiple allowed)
//   - rwlock.write().await: acquire exclusive write lock (only one)
//   - Readers and writer are mutually exclusive
//
// Run: cargo run --bin async_rwlock
// =============================================================================

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  async RwLock — Multiple Readers / One Writer   ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    let data = Arc::new(RwLock::new(vec![1, 2, 3]));
    let mut handles = vec![];

    // --- Spawn 5 reader tasks ---
    // Multiple readers can hold read locks simultaneously
    for reader_id in 0..5 {
        let data_clone = Arc::clone(&data);
        handles.push(tokio::spawn(async move {
            // .read().await — acquire shared read lock
            // Multiple tasks can hold this simultaneously
            let guard = data_clone.read().await;
            println!("  📖 Reader {} sees: {:?}", reader_id, *guard);

            // Simulate reading takes some time
            sleep(Duration::from_millis(50)).await;
            println!("  📖 Reader {} done", reader_id);
            // RwLockReadGuard dropped here → read lock released
        }));
    }

    // --- Spawn 1 writer task ---
    // Writer must wait until ALL readers release their locks
    let data_clone = Arc::clone(&data);
    handles.push(tokio::spawn(async move {
        // Small delay to let readers start first
        sleep(Duration::from_millis(10)).await;

        // .write().await — acquire exclusive write lock
        // Blocks until all read locks are released
        let mut guard = data_clone.write().await;
        guard.push(4);
        guard.push(5);
        println!("\n  ✏️  Writer modified: {:?}", *guard);
        // RwLockWriteGuard dropped here → write lock released
    }));

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Final read
    let final_data = data.read().await;
    println!("\n  Final data: {:?}", *final_data);

    println!("\n══════════════════════════════════════════════════");
    println!("Key takeaways:");
    println!("  • rwlock.read().await  — shared lock (many readers OK)");
    println!("  • rwlock.write().await — exclusive lock (one writer only)");
    println!("  • Readers and writer are mutually exclusive");
    println!("  • Guards auto-release on drop (RAII)");
    println!("  • Use RwLock when reads >> writes for better concurrency");
    println!("══════════════════════════════════════════════════");
}

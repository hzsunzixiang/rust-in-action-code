// =============================================================================
// async_basic.rs — Fundamental async/await patterns in Rust
//
// Key concepts:
//   - async fn: declares an async function (returns a Future)
//   - .await:   suspends execution until the Future completes
//   - tokio::spawn: runs a Future as an independent task
//   - tokio::join!: runs multiple Futures concurrently, waits for ALL
//   - tokio::select!: runs multiple Futures concurrently, waits for FIRST
//
// Run: cargo run --bin async_basic
// =============================================================================

use tokio::time::{sleep, Duration};

// -----------------------------------------------------------------------------
// 1. Basic async fn + .await
//
// An async fn doesn't execute immediately — it returns a Future.
// The Future only runs when you .await it.
// Think of it like a lazy evaluation: nothing happens until you ask for it.
// -----------------------------------------------------------------------------
async fn fetch_data(id: u32) -> String {
    println!("  [Task {}] Starting fetch...", id);
    sleep(Duration::from_millis(100 * id as u64)).await; // <-- yields control here
    println!("  [Task {}] Fetch complete!", id);
    format!("Data from task {}", id)
}

// -----------------------------------------------------------------------------
// 2. Sequential vs Concurrent execution
// -----------------------------------------------------------------------------
async fn sequential_demo() {
    println!("\n--- Sequential execution (one after another) ---");
    let start = std::time::Instant::now();

    // Each .await blocks until complete before starting the next
    let a = fetch_data(1).await;
    let b = fetch_data(2).await;
    let c = fetch_data(3).await;

    println!("  Results: {}, {}, {}", a, b, c);
    println!("  Time: {:?} (≈600ms, because 100+200+300)\n", start.elapsed());
}

async fn concurrent_demo() {
    println!("--- Concurrent execution with tokio::join! ---");
    let start = std::time::Instant::now();

    // join! runs all three concurrently, waits for ALL to complete
    let (a, b, c) = tokio::join!(
        fetch_data(1),
        fetch_data(2),
        fetch_data(3),
    );

    println!("  Results: {}, {}, {}", a, b, c);
    println!("  Time: {:?} (≈300ms, because max(100,200,300))\n", start.elapsed());
}

// -----------------------------------------------------------------------------
// 3. tokio::spawn — fire-and-forget tasks (like threads, but lightweight)
// -----------------------------------------------------------------------------
async fn spawn_demo() {
    println!("--- tokio::spawn: independent tasks ---");

    // spawn returns a JoinHandle — you can .await it to get the result
    let handle1 = tokio::spawn(async {
        sleep(Duration::from_millis(200)).await;
        42 // return a value from the spawned task
    });

    let handle2 = tokio::spawn(async {
        sleep(Duration::from_millis(100)).await;
        "hello from task" // different return type is fine
    });

    // .await the handles to get results
    let result1 = handle1.await.unwrap(); // unwrap the JoinError
    let result2 = handle2.await.unwrap();
    println!("  Spawned task results: {}, {}\n", result1, result2);
}

// -----------------------------------------------------------------------------
// 4. tokio::select! — race multiple futures, take the FIRST one
// -----------------------------------------------------------------------------
async fn slow_operation() -> &'static str {
    sleep(Duration::from_millis(500)).await;
    "slow result"
}

async fn fast_operation() -> &'static str {
    sleep(Duration::from_millis(100)).await;
    "fast result"
}

async fn select_demo() {
    println!("--- tokio::select!: race futures, take first ---");

    // select! cancels the loser when the winner completes
    let winner = tokio::select! {
        val = slow_operation() => { format!("Slow won: {}", val) }
        val = fast_operation() => { format!("Fast won: {}", val) }
    };

    println!("  Winner: {}", winner);
    println!("  (slow_operation was cancelled automatically)\n");
}

// -----------------------------------------------------------------------------
// 5. async closures & async blocks
// -----------------------------------------------------------------------------
async fn async_block_demo() {
    println!("--- Async blocks (inline futures) ---");

    // An async block creates an anonymous Future
    let future = async {
        sleep(Duration::from_millis(50)).await;
        "result from async block"
    };

    // The block hasn't run yet! It only runs when we .await it:
    let result = future.await;
    println!("  Got: {}\n", result);
}

// =============================================================================
// Main
// =============================================================================
#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  async/await Basics — Core Patterns             ║");
    println!("╚══════════════════════════════════════════════════╝");

    sequential_demo().await;
    concurrent_demo().await;
    spawn_demo().await;
    select_demo().await;
    async_block_demo().await;

    println!("══════════════════════════════════════════════════");
    println!("Key takeaways:");
    println!("  • async fn → returns Future (lazy, does nothing until .await)");
    println!("  • .await   → suspends current task, yields to runtime");
    println!("  • join!    → run concurrently, wait for ALL");
    println!("  • select!  → run concurrently, wait for FIRST (cancel rest)");
    println!("  • spawn    → run as independent task (like a green thread)");
    println!("══════════════════════════════════════════════════");
}

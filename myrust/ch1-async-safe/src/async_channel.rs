// =============================================================================
// async_channel.rs — Async channels: Producer-Consumer pattern
//
// Key concepts:
//   - tokio::sync::mpsc: Multi-Producer, Single-Consumer async channel
//   - tx.send(msg).await: async send (backpressure when buffer full)
//   - rx.recv().await: async receive (suspends until message available)
//   - Ownership transfer: send() takes ownership, no use-after-send
//   - Channel close: when all senders drop, recv() returns None
//
// Run: cargo run --bin async_channel
// =============================================================================

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  async Channel — Producer-Consumer              ║");
    println!("╚══════════════════════════════════════════════════╝\n");

    // Create a bounded channel with buffer size 32
    // Bounded = backpressure: if buffer is full, send().await suspends
    let (tx, mut rx) = mpsc::channel::<String>(32);

    // --- Spawn 3 producer tasks ---
    for producer_id in 0..3 {
        // Clone the sender — each producer gets its own handle
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            for i in 0..3 {
                let msg = format!("Producer {} → Message {}", producer_id, i);

                // .send(msg).await:
                //   - Takes ownership of msg (moved, not copied)
                //   - Suspends if channel buffer is full (backpressure)
                //   - Returns Err if receiver is dropped
                tx_clone.send(msg).await.unwrap();

                // Simulate some async work between sends
                sleep(Duration::from_millis(50)).await;
            }
            println!("  📤 Producer {} done sending", producer_id);
            // tx_clone is dropped here → one sender gone
        });
    }

    // IMPORTANT: drop the original sender!
    // Otherwise the channel never closes (rx.recv() would hang forever)
    drop(tx);

    // --- Consumer: receive all messages ---
    println!("  Waiting for messages...\n");
    let mut received = 0;

    // rx.recv().await:
    //   - Returns Some(msg) when a message arrives
    //   - Returns None when ALL senders are dropped (channel closed)
    while let Some(msg) = rx.recv().await {
        println!("  📥 Received: {}", msg);
        received += 1;
    }

    println!("\n  Total messages: {} (expected: 9)", received);
    assert_eq!(received, 9);

    println!("\n══════════════════════════════════════════════════");
    println!("Key takeaways:");
    println!("  • mpsc::channel(N) — bounded async channel with backpressure");
    println!("  • tx.send(msg).await — async send, suspends if buffer full");
    println!("  • rx.recv().await — async receive, returns None when closed");
    println!("  • send() takes ownership → no use-after-send bugs");
    println!("  • Drop all senders → channel closes → recv returns None");
    println!("══════════════════════════════════════════════════");
}

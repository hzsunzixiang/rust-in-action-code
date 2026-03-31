// tokio_debug_explore — Why LLDB shows "loop first, then your code"
//
//   #[tokio::main]
//   async fn main() {
//       println!("Hello");  // ← you think this is the "first line"
//       loop { ... }        // ← your event loop
//   }
//
// But #[tokio::main] transforms it into:
//
//   fn main() {
//       runtime.block_on(async { println!("Hello"); loop { ... } });
//   }
//
// And block_on() has its own poll LOOP:
//
//   fn block_on(future) {
//       loop {                  // ← LLDB sees THIS first
//           match future.poll() {
//               Ready(v) => return v,
//               Pending  => park(),
//           }
//       }
//   }
//
// So in LLDB backtrace:
//   frame #0: your println!("Hello")   ← you are HERE
//   frame #1: Future::poll()           ← poll call
//   frame #2: block_on() poll loop     ← THE RUNTIME LOOP
//   frame #3: real fn main()           ← macro-generated

use std::io::{self, Write};

#[tokio::main]
async fn main() {
    println!("Hello");  // ← "first line", but INSIDE tokio's poll loop

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        println!("Echo: {}", input.trim());
        break;
    }
}

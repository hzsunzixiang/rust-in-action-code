# =============================================================================
# tokio_debug_explore — Makefile
# Demonstrate why LLDB shows "loop first, then your code" with #[tokio::main]
#
# The key insight: #[tokio::main] is a proc macro that transforms:
#   async fn main() { ... }
# Into:
#   fn main() { runtime.block_on(async { ... }); }
#
# The block_on() contains a poll loop. Your async code runs INSIDE that loop.
# So LLDB sees: real main → runtime → poll loop → your code
# =============================================================================

CARGO = cargo
SRC = src/main.rs
TARGET = target/debug/tokio_debug_explore
NIGHTLY = rustup run nightly cargo

.PHONY: all run build clean debug expanded lldb-demo help

# Default: build and run
all: run

# Build with debug symbols
build:
	$(CARGO) build
	@echo "✅ Built: $(TARGET)"

# Run the program
run: build
	$(CARGO) run

# ====== LLDB debugging demo ======
# This is the main point: observe execution order in LLDB

debug: build
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║  Starting LLDB — observe the execution order!               ║"
	@echo "╠══════════════════════════════════════════════════════════════╣"
	@echo "║  Try these commands in LLDB:                                ║"
	@echo "║                                                             ║"
	@echo "║  (lldb) b tokio_debug_explore::main                         ║"
	@echo "║  (lldb) r                                                   ║"
	@echo "║  (lldb) bt          ← see the call stack!                   ║"
	@echo "║  (lldb) n           ← step over                             ║"
	@echo "║  (lldb) s           ← step into                             ║"
	@echo "║  (lldb) c           ← continue                              ║"
	@echo "║                                                             ║"
	@echo "║  Key observation in backtrace (bt):                         ║"
	@echo "║    frame #0: your async code (the closure)                  ║"
	@echo "║    frame #1: Future::poll()                                 ║"
	@echo "║    frame #2: block_on() poll loop  ← THE LOOP               ║"
	@echo "║    frame #3: real fn main()        ← MACRO GENERATED        ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@echo ""
	lldb $(TARGET)

# ====== Show macro expansion (nightly required) ======
# This reveals what #[tokio::main] actually generates
expanded:
	@echo "Expanding #[tokio::main] macro (requires nightly)..."
	@echo "─────────────────────────────────────────────────────"
	$(NIGHTLY) expand 2>/dev/null || (echo ""; echo "⚠️  Install cargo-expand: cargo install cargo-expand"; echo "⚠️  Install nightly: rustup install nightly")

# ====== LLDB batch demo: automated breakpoint + backtrace ======
lldb-demo: build
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║  Automated LLDB demo — showing backtrace at async main      ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "b tokio_debug_explore::main" > /tmp/lldb_commands.txt
	@echo "r" >> /tmp/lldb_commands.txt
	@echo "bt" >> /tmp/lldb_commands.txt
	@echo "q" >> /tmp/lldb_commands.txt
	lldb -s /tmp/lldb_commands.txt $(TARGET) 2>&1 | head -80
	@rm -f /tmp/lldb_commands.txt

# Clean build artifacts
clean:
	$(CARGO) clean

# Show help
help:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║  tokio_debug_explore — LLDB + #[tokio::main] Execution Order║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  The Problem:"
	@echo "    In LLDB, #[tokio::main] async fn main() appears to run"
	@echo "    the poll loop BEFORE your first line of code."
	@echo ""
	@echo "  The Reason:"
	@echo "    #[tokio::main] expands to: fn main() { rt.block_on(async{...}) }"
	@echo "    Your code is INSIDE the poll loop, not before it."
	@echo ""
	@echo "  Commands:"
	@echo "    make run       - Build and run the demo"
	@echo "    make debug     - Launch LLDB with instructions"
	@echo "    make lldb-demo - Automated LLDB backtrace demo"
	@echo "    make expanded  - Show macro expansion (nightly + cargo-expand)"
	@echo "    make clean     - Remove build artifacts"
	@echo ""
	@echo "  LLDB Quick Reference:"
	@echo "    b <symbol>  - set breakpoint"
	@echo "    r           - run"
	@echo "    bt          - backtrace (shows call stack)"
	@echo "    n           - step over"
	@echo "    s           - step into"
	@echo "    c           - continue"
	@echo "    q           - quit"


# =============================================================================
# Makefile for ch1-async-safe — Debug all async examples
#
# Usage:
#   make run BIN=async_channel       — Run any bin by name
#   make debug BIN=async_channel     — Debug any bin with lldb
#   make run-channel                 — Shortcut for async_channel
#   make debug-channel               — Debug async_channel (break at main)
# =============================================================================

# Default binary (override with: make run BIN=async_channel)
BIN        ?= async_basic
TARGET_DIR := target/debug
BINARY      = $(TARGET_DIR)/$(BIN)

# All available binaries
BINS := async_basic async_shared_counter async_channel async_rwlock

# Default target
.PHONY: help
help:
	@echo "╔══════════════════════════════════════════════════════════════╗"
	@echo "║  ch1-async-safe Makefile                                   ║"
	@echo "╚══════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Generic (use BIN=<name> to switch target, default: async_basic):"
	@echo "  make build            — Build debug binary"
	@echo "  make run              — Build and run"
	@echo "  make debug            — Build and launch lldb"
	@echo "  make debug-break      — Build, launch lldb, break at main"
	@echo "  make release          — Build release binary"
	@echo "  make run-release      — Build and run release binary"
	@echo "  make expand           — Expand macros (requires cargo-expand)"
	@echo "  make asm              — Show disassembly (first 200 lines)"
	@echo ""
	@echo "  Example: make run BIN=async_channel"
	@echo "  Example: make debug BIN=async_channel"
	@echo ""
	@echo "Shortcuts — async_basic:"
	@echo "  make run-basic        — Run async_basic"
	@echo "  make debug-basic      — Debug async_basic (break at main)"
	@echo ""
	@echo "Shortcuts — async_channel:"
	@echo "  make run-channel      — Run async_channel"
	@echo "  make debug-channel    — Debug async_channel (break at main)"
	@echo "  make debug-channel-send — Debug: break at mpsc send"
	@echo "  make debug-channel-recv — Debug: break at mpsc recv loop"
	@echo ""
	@echo "Shortcuts — async_shared_counter:"
	@echo "  make run-counter        — Run async_shared_counter"
	@echo "  make debug-counter      — Debug async_shared_counter (break at main)"
	@echo "  make debug-counter-lock — Debug: break at mutex lock"
	@echo "  make debug-counter-inc  — Debug: break at counter increment"
	@echo ""
	@echo "Shortcuts — async_rwlock:"
	@echo "  make run-rwlock         — Run async_rwlock"
	@echo "  make debug-rwlock       — Debug async_rwlock (break at main)"
	@echo "  make debug-rwlock-read  — Debug: break at read lock"
	@echo "  make debug-rwlock-write — Debug: break at write lock"
	@echo ""
	@echo "Utilities:"
	@echo "  make check            — Check compilation (all bins)"
	@echo "  make clean            — Clean build artifacts"
	@echo "  make build-all        — Build all bins"
	@echo "  make run-all          — Run all bins sequentially"

# -----------------------------------------------------------------------------
# Build
# -----------------------------------------------------------------------------
.PHONY: build
build:
	cargo build --bin $(BIN)

.PHONY: build-all
build-all:
	@for b in $(BINS); do echo "=== Building $$b ==="; cargo build --bin $$b; done

.PHONY: release
release:
	cargo build --release --bin $(BIN)

.PHONY: check
check:
	cargo check

.PHONY: clean
clean:
	cargo clean

# -----------------------------------------------------------------------------
# Run
# -----------------------------------------------------------------------------
.PHONY: run
run:
	cargo run --bin $(BIN)

.PHONY: run-release
run-release:
	cargo run --release --bin $(BIN)

.PHONY: run-all
run-all:
	@for b in $(BINS); do echo "\n=== Running $$b ===\n"; cargo run --bin $$b; done

# -----------------------------------------------------------------------------
# Debug with lldb (generic)
# -----------------------------------------------------------------------------
.PHONY: debug
debug: build
	rust-lldb $(BINARY)

.PHONY: debug-break
debug-break: build
	rust-lldb $(BINARY) -o "b $(BIN)::main" -o "run"

# -----------------------------------------------------------------------------
# Macro expansion (requires: cargo install cargo-expand)
# -----------------------------------------------------------------------------
.PHONY: expand
expand:
	cargo expand --bin $(BIN)

# -----------------------------------------------------------------------------
# Disassembly
# -----------------------------------------------------------------------------
.PHONY: asm
asm: build
	objdump -d $(BINARY) | head -200

# =============================================================================
# Shortcuts — async_basic
# =============================================================================
.PHONY: run-basic debug-basic
run-basic:
	cargo run --bin async_basic

debug-basic:
	cargo build --bin async_basic
	rust-lldb $(TARGET_DIR)/async_basic -o "b async_basic::main" -o "run"

# =============================================================================
# Shortcuts — async_channel
# =============================================================================
.PHONY: run-channel debug-channel debug-channel-send debug-channel-recv

run-channel:
	cargo run --bin async_channel

# Break at main — see the channel creation and producer spawning
debug-channel:
	cargo build --bin async_channel
	rust-lldb $(TARGET_DIR)/async_channel -o "b async_channel::main" -o "run"

# Break at the send() call inside the producer task
# Useful for inspecting: msg content, tx_clone state, channel buffer
debug-channel-send:
	cargo build --bin async_channel
	rust-lldb $(TARGET_DIR)/async_channel \
		-o "b async_channel::main" \
		-o "run" \
		-o 'br set -f async_channel.rs -p "tx_clone.send"'

# Break at the recv() loop in the consumer
# Useful for inspecting: received messages, channel close behavior
debug-channel-recv:
	cargo build --bin async_channel
	rust-lldb $(TARGET_DIR)/async_channel \
		-o "b async_channel::main" \
		-o "run" \
		-o 'br set -f async_channel.rs -p "rx.recv"'

# =============================================================================
# Shortcuts — async_shared_counter
# =============================================================================
.PHONY: run-counter debug-counter debug-counter-lock debug-counter-inc

run-counter:
	cargo run --bin async_shared_counter

# Break at main — see Arc<Mutex<T>> creation and task spawning
debug-counter:
	cargo build --bin async_shared_counter
	rust-lldb $(TARGET_DIR)/async_shared_counter -o "b async_shared_counter::main" -o "run"

# Break at the lock().await call — inspect lock contention
# Useful for: seeing which task acquires the lock, MutexGuard state
debug-counter-lock:
	cargo build --bin async_shared_counter
	rust-lldb $(TARGET_DIR)/async_shared_counter \
		-o "b async_shared_counter::main" \
		-o "run" \
		-o 'br set -f async_shared_counter.rs -p "counter_clone.lock"'

# Break at the increment — inspect counter value changes
# Useful for: verifying no data race, watching counter grow
debug-counter-inc:
	cargo build --bin async_shared_counter
	rust-lldb $(TARGET_DIR)/async_shared_counter \
		-o "b async_shared_counter::main" \
		-o "run" \
		-o 'br set -f async_shared_counter.rs -p "\*num \+= 1"'

# =============================================================================
# Shortcuts — async_rwlock
# =============================================================================
.PHONY: run-rwlock debug-rwlock debug-rwlock-read debug-rwlock-write

run-rwlock:
	cargo run --bin async_rwlock

# Break at main — see RwLock creation and task spawning
debug-rwlock:
	cargo build --bin async_rwlock
	rust-lldb $(TARGET_DIR)/async_rwlock -o "b async_rwlock::main" -o "run"

# Break at read().await — inspect concurrent readers
# Useful for: verifying multiple readers hold lock simultaneously
debug-rwlock-read:
	cargo build --bin async_rwlock
	rust-lldb $(TARGET_DIR)/async_rwlock \
		-o "b async_rwlock::main" \
		-o "run" \
		-o 'br set -f async_rwlock.rs -p "data_clone.read"'

# Break at write().await — inspect exclusive writer
# Useful for: seeing writer wait for readers, then modify data
debug-rwlock-write:
	cargo build --bin async_rwlock
	rust-lldb $(TARGET_DIR)/async_rwlock \
		-o "b async_rwlock::main" \
		-o "run" \
		-o 'br set -f async_rwlock.rs -p "data_clone.write"'

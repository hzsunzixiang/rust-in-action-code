
# =============================================================================
# Makefile for ch1-async-safe — focused on async_basic.rs debugging
# =============================================================================

BIN_NAME   := async_basic
TARGET_DIR := target/debug
BINARY     := $(TARGET_DIR)/$(BIN_NAME)

# Default target
.PHONY: help
help:
	@echo "Usage:"
	@echo "  make build       — Build debug binary"
	@echo "  make run         — Build and run async_basic"
	@echo "  make debug       — Build and launch lldb debugger"
	@echo "  make debug-break — Build and launch lldb, break at main"
	@echo "  make release     — Build release binary"
	@echo "  make run-release — Build and run release binary"
	@echo "  make clean       — Clean build artifacts"
	@echo "  make check       — Check compilation without building"
	@echo "  make expand      — Expand macros (requires cargo-expand)"
	@echo "  make asm         — Show disassembly of the binary"
	@echo ""
	@echo "Other bins:"
	@echo "  make run-counter — Run async_shared_counter"
	@echo "  make run-channel — Run async_channel"
	@echo "  make run-rwlock  — Run async_rwlock"

# -----------------------------------------------------------------------------
# Build
# -----------------------------------------------------------------------------
.PHONY: build
build:
	cargo build --bin $(BIN_NAME)

.PHONY: release
release:
	cargo build --release --bin $(BIN_NAME)

.PHONY: check
check:
	cargo check --bin $(BIN_NAME)

.PHONY: clean
clean:
	cargo clean

# -----------------------------------------------------------------------------
# Run
# -----------------------------------------------------------------------------
.PHONY: run
run:
	cargo run --bin $(BIN_NAME)

.PHONY: run-release
run-release:
	cargo run --release --bin $(BIN_NAME)

# -----------------------------------------------------------------------------
# Debug with lldb
# -----------------------------------------------------------------------------
.PHONY: debug
debug: build
	rust-lldb $(BINARY)

.PHONY: debug-break
debug-break: build
	rust-lldb $(BINARY) -o "b async_basic::main" -o "run"

# -----------------------------------------------------------------------------
# Macro expansion (requires: cargo install cargo-expand)
# -----------------------------------------------------------------------------
.PHONY: expand
expand:
	cargo expand --bin $(BIN_NAME)

# -----------------------------------------------------------------------------
# Disassembly
# -----------------------------------------------------------------------------
.PHONY: asm
asm: build
	objdump -d $(BINARY) | head -200

# -----------------------------------------------------------------------------
# Other bins in this crate
# -----------------------------------------------------------------------------
.PHONY: run-counter
run-counter:
	cargo run --bin async_shared_counter

.PHONY: run-channel
run-channel:
	cargo run --bin async_channel

.PHONY: run-rwlock
run-rwlock:
	cargo run --bin async_rwlock

# =============================================================================
# Rust WebAssembly (WASI) Demo — Makefile
#
# Compile Rust to WebAssembly using wasm32-wasip1 target
#
# Pipeline:
#   .rs → (rustc/cargo) → .wasm → wasmtime (WASI runtime)
#
# Prerequisites:
#   rustup target add wasm32-wasip1
#   brew install wasmtime   (or: curl https://wasmtime.dev/install.sh -sSf | bash)
#
# Quick start:
#   make setup    — install wasm32-wasip1 target + wasmtime (one-time)
#   make          — compile Rust to WASM
#   make run      — run WASM via wasmtime
#   make clean    — remove build artifacts
# =============================================================================

CARGO = cargo
TARGET = wasm32-wasip1
BUILD_DIR = build
BIN_NAME = rust_wasm_main
WASM_FILE = target/$(TARGET)/release/$(BIN_NAME).wasm
WASM_COPY = $(BUILD_DIR)/$(BIN_NAME).wasm

.PHONY: all clean run setup help wat size inspect check native

# ====== Default: compile to WASM ======
all: $(WASM_COPY)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(WASM_COPY): src/main.rs Cargo.toml | $(BUILD_DIR)
	$(CARGO) build --release --target $(TARGET)
	cp $(WASM_FILE) $(WASM_COPY)
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  ✅ Rust → WASM (WASI) compilation successful!                 ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  Output: $(WASM_COPY)"
	@ls -lh $(WASM_COPY)

# ====== Run WASM via wasmtime ======
run: $(WASM_COPY)
	@echo ""
	@echo "▶ Running WASM via wasmtime..."
	@echo "─────────────────────────────────"
	wasmtime $(WASM_COPY)

# ====== Compile and run as native binary (for comparison) ======
native:
	$(CARGO) run --release
	@echo ""
	@echo "(Above output is from native binary, not WASM)"

# ====== Type-check without building ======
check:
	$(CARGO) check --target $(TARGET)

# ====== Generate WAT (WebAssembly Text Format) for inspection ======
wat: $(WASM_COPY)
	@if command -v wasm2wat > /dev/null 2>&1; then \
		wasm2wat $(WASM_COPY) -o $(BUILD_DIR)/$(BIN_NAME).wat; \
		echo "✅ WAT output: $(BUILD_DIR)/$(BIN_NAME).wat"; \
		echo "   Lines: $$(wc -l < $(BUILD_DIR)/$(BIN_NAME).wat)"; \
	else \
		echo "❌ wasm2wat not found. Install: brew install wabt"; \
	fi

# ====== Show WASM binary size breakdown ======
size: $(WASM_COPY)
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  📊 WASM Binary Size Analysis                                  ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@ls -lh $(WASM_COPY)
	@echo ""
	@if command -v wasm-objdump > /dev/null 2>&1; then \
		echo "  Sections:"; \
		wasm-objdump -h $(WASM_COPY); \
	else \
		echo "  WASM size: $$(wc -c < $(WASM_COPY)) bytes"; \
		echo "  (install wabt for detailed section info: brew install wabt)"; \
	fi

# ====== Inspect exported functions ======
inspect: $(WASM_COPY)
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  🔍 WASM Module Inspection                                     ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@if command -v wasm-objdump > /dev/null 2>&1; then \
		echo "  Exports:"; \
		wasm-objdump -x $(WASM_COPY) | grep -E "export|^ - " | head -30; \
		echo ""; \
		echo "  Imports (WASI syscalls):"; \
		wasm-objdump -x $(WASM_COPY) | grep -E "import|^ - " | head -30; \
	else \
		echo "  (install wabt: brew install wabt)"; \
	fi

# ====== Strip WASM binary (remove debug info, reduce size) ======
strip: $(WASM_COPY)
	@if command -v wasm-strip > /dev/null 2>&1; then \
		echo "Before strip:"; \
		ls -lh $(WASM_COPY); \
		wasm-strip $(WASM_COPY); \
		echo "After strip:"; \
		ls -lh $(WASM_COPY); \
	else \
		echo "❌ wasm-strip not found. Install: brew install wabt"; \
	fi

# ====== Install prerequisites (one-time setup) ======
setup:
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  🔧 Setting up Rust WASM (WASI) toolchain                     ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Step 1: Adding wasm32-wasip1 target..."
	rustup target add $(TARGET)
	@echo ""
	@echo "Step 2: Installing wasmtime runtime..."
	@if command -v wasmtime > /dev/null 2>&1; then \
		echo "  wasmtime already installed: $$(wasmtime --version)"; \
	else \
		echo "  Installing via Homebrew..."; \
		brew install wasmtime; \
	fi
	@echo ""
	@echo "Step 3 (optional): Installing wabt tools..."
	@if command -v wasm2wat > /dev/null 2>&1; then \
		echo "  wabt already installed."; \
	else \
		echo "  Installing via Homebrew..."; \
		brew install wabt; \
	fi
	@echo ""
	@echo "✅ Setup complete! Run 'make' to compile, 'make run' to execute."

# ====== Clean build artifacts ======
clean:
	$(CARGO) clean
	rm -rf $(BUILD_DIR)
	@echo "✅ Cleaned."

# ====== Help ======
help:
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  Rust WebAssembly (WASI) Demo                                  ║"
	@echo "║  Compile Rust to WASM, run via wasmtime                        ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  Pipeline: .rs → cargo build --target wasm32-wasip1 → .wasm → wasmtime"
	@echo ""
	@echo "  setup     — Install wasm32-wasip1 target + wasmtime + wabt"
	@echo "  all       — Compile Rust to WASM (default)"
	@echo "  run       — Run WASM via wasmtime"
	@echo "  native    — Compile & run as native binary (for comparison)"
	@echo "  check     — Type-check for WASM target"
	@echo "  wat       — Generate WAT (text format) for inspection"
	@echo "  size      — Show WASM binary size analysis"
	@echo "  inspect   — Inspect exports & imports"
	@echo "  strip     — Strip debug info from WASM binary"
	@echo "  clean     — Remove build artifacts"
	@echo "  help      — Show this help"
	@echo ""
	@echo "  Prerequisites: run 'make setup' first (one-time)"

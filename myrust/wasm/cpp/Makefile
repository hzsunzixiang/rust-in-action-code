# =============================================================================
# C++ WebAssembly Demo — Makefile
#
# Compile C++ to WebAssembly using Emscripten (emcc)
#
# Pipeline:
#   .cpp → (emcc) → .wasm + .js (glue code) → browser / Node.js
#
# Prerequisites:
#   brew install emscripten   (or: git clone emsdk && ./emsdk install latest)
#
# Quick start:
#   make setup    — install emscripten via brew (one-time)
#   make          — compile C++ to WASM
#   make run      — run in Node.js (CLI)
#   make serve    — start HTTP server and open browser
#   make clean    — remove build artifacts
# =============================================================================

EMCC = emcc
SRC = src/main.cpp
BUILD_DIR = build
WEB_DIR = web
TARGET_JS = $(BUILD_DIR)/main.js
TARGET_WASM = $(BUILD_DIR)/main.wasm

# Emscripten compile flags
EMFLAGS = \
	-O2 \
	-s WASM=1 \
	-s EXPORTED_RUNTIME_METHODS='["ccall","cwrap","UTF8ToString","stringToUTF8","lengthBytesUTF8"]' \
	-s ALLOW_MEMORY_GROWTH=1 \
	-s EXPORTED_FUNCTIONS='["_main","_add","_multiply","_fibonacci","_create_array","_free_array","_array_sum","_reverse_string","_is_prime","_count_primes","_malloc","_free"]' \
	-s NO_EXIT_RUNTIME=1

.PHONY: all clean run serve setup help wat size inspect

# ====== Default: compile to WASM ======
all: $(TARGET_JS)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(TARGET_JS): $(SRC) | $(BUILD_DIR)
	$(EMCC) $(SRC) -o $(TARGET_JS) $(EMFLAGS)
	@cp $(TARGET_JS) $(WEB_DIR)/main.js
	@cp $(TARGET_WASM) $(WEB_DIR)/main.wasm
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  ✅ C++ → WASM compilation successful!                         ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  Output files:"
	@echo "    $(TARGET_WASM)  — WebAssembly binary"
	@echo "    $(TARGET_JS)    — JavaScript glue code"
	@echo ""
	@ls -lh $(TARGET_WASM) $(TARGET_JS)

# ====== Run in Node.js (CLI mode) ======
run: $(TARGET_JS)
	@echo ""
	@echo "▶ Running WASM in Node.js..."
	@echo "─────────────────────────────"
	node $(TARGET_JS)

# ====== Serve in browser ======
serve: $(TARGET_JS)
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  🌐 Starting HTTP server at http://localhost:8080              ║"
	@echo "║  Open browser to see the demo. Press Ctrl+C to stop.          ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@cd $(WEB_DIR) && python3 -m http.server 8080

# ====== Generate WAT (WebAssembly Text Format) for inspection ======
wat: $(TARGET_JS)
	@if command -v wasm2wat > /dev/null 2>&1; then \
		wasm2wat $(TARGET_WASM) -o $(BUILD_DIR)/main.wat; \
		echo "✅ WAT output: $(BUILD_DIR)/main.wat"; \
		echo "   Lines: $$(wc -l < $(BUILD_DIR)/main.wat)"; \
	else \
		echo "❌ wasm2wat not found. Install: brew install wabt"; \
	fi

# ====== Show WASM binary size breakdown ======
size: $(TARGET_JS)
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  📊 WASM Binary Size Analysis                                  ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@ls -lh $(TARGET_WASM) $(TARGET_JS)
	@echo ""
	@echo "  WASM sections:"
	@if command -v wasm-objdump > /dev/null 2>&1; then \
		wasm-objdump -h $(TARGET_WASM); \
	else \
		echo "  (install wabt for detailed section info: brew install wabt)"; \
		echo "  WASM size: $$(wc -c < $(TARGET_WASM)) bytes"; \
	fi

# ====== Inspect exported functions ======
inspect: $(TARGET_JS)
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  🔍 Exported WASM Functions                                    ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@if command -v wasm-objdump > /dev/null 2>&1; then \
		wasm-objdump -x $(TARGET_WASM) | grep -E "^ - func|export" | head -30; \
	else \
		echo "  (install wabt: brew install wabt)"; \
	fi

# ====== Compile with debug info (larger, but debuggable) ======
debug: $(SRC) | $(BUILD_DIR)
	$(EMCC) $(SRC) -o $(BUILD_DIR)/main-debug.js \
		-g \
		-s WASM=1 \
		-s EXPORTED_RUNTIME_METHODS='["ccall","cwrap","UTF8ToString","stringToUTF8","lengthBytesUTF8"]' \
		-s ALLOW_MEMORY_GROWTH=1 \
		-s EXPORTED_FUNCTIONS='["_main","_add","_multiply","_fibonacci","_create_array","_free_array","_array_sum","_reverse_string","_is_prime","_count_primes","_malloc","_free"]' \
		-s NO_EXIT_RUNTIME=1 \
		-s ASSERTIONS=2
	@echo "✅ Debug build: $(BUILD_DIR)/main-debug.js"

# ====== Install Emscripten (one-time setup) ======
setup:
	@echo "Installing Emscripten via Homebrew..."
	brew install emscripten
	@echo ""
	@echo "✅ Emscripten installed. Verify with: emcc --version"

# ====== Clean build artifacts ======
clean:
	rm -rf $(BUILD_DIR)
	rm -f $(WEB_DIR)/main.js $(WEB_DIR)/main.wasm
	@echo "✅ Cleaned."

# ====== Help ======
help:
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  C++ WebAssembly Demo                                          ║"
	@echo "║  Compile C++ to WASM using Emscripten (emcc)                   ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  Pipeline: .cpp → emcc → .wasm + .js → browser / Node.js"
	@echo ""
	@echo "  setup     — Install Emscripten (brew install emscripten)"
	@echo "  all       — Compile C++ to WASM (default)"
	@echo "  run       — Run WASM in Node.js"
	@echo "  serve     — Start HTTP server and open in browser"
	@echo "  debug     — Compile with debug info and assertions"
	@echo "  wat       — Generate WAT (text format) for inspection"
	@echo "  size      — Show WASM binary size analysis"
	@echo "  inspect   — List exported WASM functions"
	@echo "  clean     — Remove build artifacts"
	@echo "  help      — Show this help"
	@echo ""
	@echo "  Prerequisites: emcc (run 'make setup' to install)"
	@echo "  Optional:      wabt (for wat/inspect: brew install wabt)"

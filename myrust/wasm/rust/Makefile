# Rust WebAssembly Demo - Makefile
# ================================

# Configuration
WASM_PACK := wasm-pack
CARGO := cargo
PORT := 8080

.PHONY: all build serve clean setup check help

# Default target
all: build

# ---------------------------------------------------------------
# Setup: Install required tools
# ---------------------------------------------------------------
setup:
	@echo "🔧 Installing required tools..."
	@command -v wasm-pack >/dev/null 2>&1 || { \
		echo "📦 Installing wasm-pack..."; \
		curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh; \
	}
	@rustup target add wasm32-unknown-unknown 2>/dev/null || true
	@echo "✅ Setup complete!"

# ---------------------------------------------------------------
# Build: Compile Rust to WebAssembly
# ---------------------------------------------------------------
build:
	@echo "🦀 Building Rust to WebAssembly..."
	$(WASM_PACK) build --target web --out-dir pkg
	@echo "✅ Build complete! Output in ./pkg/"

# Build in release mode with optimizations
build-release:
	@echo "🦀 Building Rust to WebAssembly (release)..."
	$(WASM_PACK) build --target web --out-dir pkg --release
	@echo "✅ Release build complete!"

# ---------------------------------------------------------------
# Serve: Start a local HTTP server
# ---------------------------------------------------------------
serve: build
	@echo "🌐 Starting HTTP server at http://localhost:$(PORT)"
	@echo "   Press Ctrl+C to stop"
	@python3 -m http.server $(PORT)

# Serve without rebuilding
serve-only:
	@echo "🌐 Starting HTTP server at http://localhost:$(PORT)"
	@echo "   Press Ctrl+C to stop"
	@python3 -m http.server $(PORT)

# ---------------------------------------------------------------
# Check: Run cargo check and clippy
# ---------------------------------------------------------------
check:
	@echo "🔍 Running cargo check..."
	$(CARGO) check --target wasm32-unknown-unknown
	@echo "🔍 Running clippy..."
	$(CARGO) clippy --target wasm32-unknown-unknown -- -D warnings
	@echo "✅ All checks passed!"

# ---------------------------------------------------------------
# Test: Run tests
# ---------------------------------------------------------------
test:
	@echo "🧪 Running tests..."
	$(CARGO) test
	@echo "✅ Tests passed!"

# ---------------------------------------------------------------
# Clean: Remove build artifacts
# ---------------------------------------------------------------
clean:
	@echo "🧹 Cleaning build artifacts..."
	$(CARGO) clean
	rm -rf pkg/
	@echo "✅ Clean complete!"

# ---------------------------------------------------------------
# Help: Show available targets
# ---------------------------------------------------------------
help:
	@echo ""
	@echo "🦀 Rust WebAssembly Demo - Available Commands"
	@echo "=============================================="
	@echo ""
	@echo "  make setup          Install required tools (wasm-pack, wasm target)"
	@echo "  make build          Build WASM module (debug)"
	@echo "  make build-release  Build WASM module (release, optimized)"
	@echo "  make serve          Build and start HTTP server (port $(PORT))"
	@echo "  make serve-only     Start HTTP server without rebuilding"
	@echo "  make check          Run cargo check and clippy"
	@echo "  make test           Run cargo tests"
	@echo "  make clean          Remove all build artifacts"
	@echo "  make help           Show this help message"
	@echo ""

# Rust WebAssembly Demo - Makefile
# ================================

# Configuration
WASM_PACK := wasm-pack
CARGO := cargo
PORT := 8081

.PHONY: all build gen-run-script run serve clean setup check help kill-port

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
# Generate: Create run_node.mjs for Node.js execution
# ---------------------------------------------------------------
# The base64 string below encodes the run_node.mjs script content.
# To update: modify run_node.mjs, then run: base64 -i run_node.mjs | tr -d '\n'
RUN_NODE_B64 := Ly8gcnVuX25vZGUubWpzIC0gUnVuIFJ1c3QgV0FTTSBkaXJlY3RseSBpbiBOb2RlLmpzIChubyBicm93c2VyIG5lZWRlZCkKaW1wb3J0IHsgcmVhZEZpbGVTeW5jIH0gZnJvbSAnbm9kZTpmcyc7CmltcG9ydCB7IGZpbGVVUkxUb1BhdGggfSBmcm9tICdub2RlOnVybCc7CmltcG9ydCB7IGRpcm5hbWUsIGpvaW4gfSBmcm9tICdub2RlOnBhdGgnOwoKLy8gTG9hZCB0aGUgV0FTTSBtb2R1bGUgYW5kIEpTIGdsdWUgY29kZQpjb25zdCBfX2Rpcm5hbWUgPSBkaXJuYW1lKGZpbGVVUkxUb1BhdGgoaW1wb3J0Lm1ldGEudXJsKSk7CmNvbnN0IHdhc21QYXRoID0gam9pbihfX2Rpcm5hbWUsICdwa2cnLCAncnVzdF93YXNtX2RlbW9fYmcud2FzbScpOwpjb25zdCB3YXNtQnl0ZXMgPSByZWFkRmlsZVN5bmMod2FzbVBhdGgpOwoKLy8gSW1wb3J0IHRoZSBnbHVlIGNvZGUgKGluaXRTeW5jIGFsbG93cyBzeW5jaHJvbm91cyBpbml0aWFsaXphdGlvbiB3aXRob3V0IGZldGNoKQppbXBvcnQgeyBpbml0U3luYywgZ3JlZXQsIGZpYm9uYWNjaSwgc29ydF9udW1iZXJzIH0gZnJvbSAnLi9wa2cvcnVzdF93YXNtX2RlbW8uanMnOwoKLy8gSW5pdGlhbGl6ZSBXQVNNIHN5bmNocm9ub3VzbHkgd2l0aCB0aGUgcmF3IGJ5dGVzCmluaXRTeW5jKHsgbW9kdWxlOiBuZXcgV2ViQXNzZW1ibHkuTW9kdWxlKHdhc21CeXRlcykgfSk7CgovLyA9PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09Ci8vIE5vdyBjYWxsIFJ1c3QgZnVuY3Rpb25zIGRpcmVjdGx5IGZyb20gTm9kZS5qcyEKLy8gPT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PT09PQoKY29uc29sZS5sb2coJz09PSBSdXN0IFdBU00gcnVubmluZyBpbiBOb2RlLmpzID09PVxuJyk7CgovLyBFeGFtcGxlIDE6IGdyZWV0CmNvbnN0IGdyZWV0aW5nID0gZ3JlZXQoJ05vZGUuanMnKTsKY29uc29sZS5sb2coYGdyZWV0OiAke2dyZWV0aW5nfWApOwpjb25zb2xlLmxvZygpOwoKLy8gRXhhbXBsZSAyOiBmaWJvbmFjY2kKZm9yIChjb25zdCBuIG9mIFswLCAxLCA1LCAxMCwgMjAsIDUwXSkgewogICAgY29uc3QgcmVzdWx0ID0gZmlib25hY2NpKG4pOwogICAgY29uc29sZS5sb2coYGZpYm9uYWNjaSgke259KSA9ICR7cmVzdWx0fWApOwp9CmNvbnNvbGUubG9nKCk7CgovLyBFeGFtcGxlIDM6IHNvcnRfbnVtYmVycwpjb25zdCBpbnB1dCA9ICc0MiwgNywgMTMsIDk5LCAxLCA1NiwgMjMnOwpjb25zdCBzb3J0ZWQgPSBzb3J0X251bWJlcnMoaW5wdXQpOwpjb25zb2xlLmxvZyhgc29ydF9udW1iZXJzKCIke2lucHV0fSIpYCk7CmNvbnNvbGUubG9nKGAgID0+ICR7c29ydGVkfWApOwpjb25zb2xlLmxvZygpOwoKY29uc29sZS5sb2coJz09PSBEb25lISA9PT0nKTsK

gen-run-script:
	@echo "📝 Generating run_node.mjs..."
	@echo "$(RUN_NODE_B64)" | base64 -d > run_node.mjs
	@echo "✅ run_node.mjs generated!"

# ---------------------------------------------------------------
# Run: Run WASM in Node.js (no browser needed)
# ---------------------------------------------------------------
run: build gen-run-script
	@echo "🚀 Running WASM in Node.js..."
	@node run_node.mjs

# Run without rebuilding
run-only:
	@echo "🚀 Running WASM in Node.js..."
	@node run_node.mjs

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
# Kill-port: Kill process occupying the server port
# ---------------------------------------------------------------
kill-port:
	@echo "🔪 Killing process on port $(PORT)..."
	@lsof -ti:$(PORT) | xargs kill -9 2>/dev/null && echo "✅ Port $(PORT) freed!" || echo "ℹ️  No process found on port $(PORT)"

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
	@echo "  make gen-run-script Generate run_node.mjs script"
	@echo "  make run            Build and run WASM in Node.js (no browser)"
	@echo "  make run-only       Run WASM in Node.js without rebuilding"
	@echo "  make serve          Build and start HTTP server (port $(PORT))"
	@echo "  make serve-only     Start HTTP server without rebuilding"
	@echo "  make check          Run cargo check and clippy"
	@echo "  make test           Run cargo tests"
	@echo "  make kill-port      Kill process occupying port $(PORT)"
	@echo "  make clean          Remove all build artifacts"
	@echo "  make help           Show this help message"
	@echo ""

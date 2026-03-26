# =============================================================================
# future_explore — Makefile
# Explore how Rust compiler transforms async/await into Future state machines
#
# NO cargo, NO external dependencies — pure rustc compilation.
#
# Compilation Pipeline:
#   Source (.rs) → AST → Expanded → HIR → MIR → LLVM IR → Assembly → Binary
#                  ↑         ↑        ↑     ↑      ↑          ↑
#               ast      expanded   hir    mir   llvm-ir     asm
#
# The most interesting stages for async/await:
#   - expanded: shows how async fn is desugared (the state machine skeleton)
#   - hir:      shows the desugared Future trait implementation
#   - mir:      shows the actual state machine states and transitions
# =============================================================================

RUSTC = rustc
NIGHTLY = rustup run nightly rustc
SRC = src/main.rs
OUT_DIR = build
TARGET = $(OUT_DIR)/future_explore
BIN = future_explore

.PHONY: all clean ast expanded hir mir llvm-ir asm asm-opt all-ir run help

# Build the binary (default)
all: $(TARGET)

$(OUT_DIR):
	mkdir -p $(OUT_DIR)

# Compile to executable
$(TARGET): $(SRC) | $(OUT_DIR)
	$(RUSTC) --edition 2021 $(SRC) -o $(TARGET)

# ====== Pipeline Stage 1: AST (before macro expansion, nightly only) ======
# Shows the raw abstract syntax tree before any macro processing
ast: $(SRC) | $(OUT_DIR)
	$(NIGHTLY) --edition 2021 $(SRC) -Z unpretty=ast-tree > $(OUT_DIR)/$(BIN).ast 2>&1 || true
	@echo "✅ AST output: $(OUT_DIR)/$(BIN).ast"

# ====== Pipeline Stage 2: Expanded (after macro expansion, nightly only) ======
# ⭐ KEY STAGE: Shows how async fn is transformed!
# Look for: from_coroutine, poll, Pin, Context
expanded: $(SRC) | $(OUT_DIR)
	$(NIGHTLY) --edition 2021 $(SRC) -Z unpretty=expanded > $(OUT_DIR)/$(BIN).expanded.rs 2>&1 || true
	@echo "✅ Expanded output: $(OUT_DIR)/$(BIN).expanded.rs"
	@echo "   ⭐ This is the most important file for understanding Future transformation!"

# ====== Pipeline Stage 3: HIR (High-level IR, desugared form, nightly only) ======
# Shows the desugared form with explicit Future trait usage
# ⭐ KEY STAGE: Shows how .await is desugared into loop { match poll() { ... } }
hir: $(SRC) | $(OUT_DIR)
	$(NIGHTLY) --edition 2021 $(SRC) -Z unpretty=hir > $(OUT_DIR)/$(BIN).hir 2>&1 || true
	@echo "✅ HIR output: $(OUT_DIR)/$(BIN).hir"
	@echo "   ⭐ Look for .await desugared into loop+poll!"

# ====== Pipeline Stage 4: MIR (Mid-level IR, control-flow graph) ======
# ⭐ KEY STAGE: Shows the actual state machine with numbered states!
# Look for: discriminant, switchInt, variant, Poll::Pending/Ready
mir: $(SRC) | $(OUT_DIR)
	$(RUSTC) --edition 2021 $(SRC) --emit mir -o $(OUT_DIR)/$(BIN).mir
	@echo "✅ MIR output: $(OUT_DIR)/$(BIN).mir"
	@echo "   ⭐ Look for 'discriminant' and state transitions!"

# ====== Pipeline Stage 5: LLVM IR (.ll file) ======
llvm-ir: $(SRC) | $(OUT_DIR)
	$(RUSTC) --edition 2021 $(SRC) --emit llvm-ir -o $(OUT_DIR)/$(BIN).ll
	@echo "✅ LLVM IR output: $(OUT_DIR)/$(BIN).ll"

# ====== Pipeline Stage 6: Assembly (.s file) ======
asm: $(SRC) | $(OUT_DIR)
	$(RUSTC) --edition 2021 $(SRC) --emit asm -o $(OUT_DIR)/$(BIN).s
	@echo "✅ Assembly output: $(OUT_DIR)/$(BIN).s"

# ====== Assembly with optimizations ======
asm-opt: $(SRC) | $(OUT_DIR)
	$(RUSTC) --edition 2021 $(SRC) --emit asm -C opt-level=3 -o $(OUT_DIR)/$(BIN)-opt.s
	@echo "✅ Optimized assembly output: $(OUT_DIR)/$(BIN)-opt.s"

# ====== Generate ALL intermediate representations ======
all-ir: ast expanded hir mir llvm-ir asm
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  All pipeline intermediate files generated in $(OUT_DIR)/       ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  Pipeline: .rs → .ast → .expanded.rs → .hir → .mir → .ll → .s"
	@echo ""
	@echo "  ⭐ Start reading from these files:"
	@echo "     1. $(OUT_DIR)/$(BIN).expanded.rs  — async fn → state machine"
	@echo "     2. $(OUT_DIR)/$(BIN).hir          — .await → loop+poll"
	@echo "     3. $(OUT_DIR)/$(BIN).mir          — state machine states"
	@echo ""
	@ls -lh $(OUT_DIR)/$(BIN).ast $(OUT_DIR)/$(BIN).expanded.rs $(OUT_DIR)/$(BIN).hir \
	        $(OUT_DIR)/$(BIN).mir $(OUT_DIR)/$(BIN).ll $(OUT_DIR)/$(BIN).s 2>/dev/null || true

# ====== Quick peek: extract Future-related code from expanded output ======
peek-future: expanded
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  Extracted Future-related code from expanded output             ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@grep -n -A 5 "impl.*Future\|fn poll\|Pin\|Context\|coroutine\|__awaitee\|yield\|from_coroutine" \
		$(OUT_DIR)/$(BIN).expanded.rs 2>/dev/null || echo "  (run 'make expanded' first)"

# ====== Quick peek: extract state machine from MIR ======
peek-states: mir
	@echo ""
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  State machine transitions from MIR                             ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@grep -n "discriminant\|switchInt\|Unresumed\|Returned\|Panicked\|Suspend\|Poll" \
		$(OUT_DIR)/$(BIN).mir 2>/dev/null | head -60 || echo "  (run 'make mir' first)"

# Run the program
run: $(TARGET)
	./$(TARGET)

# Clean build artifacts
clean:
	rm -rf $(OUT_DIR)

# Show help
help:
	@echo "╔══════════════════════════════════════════════════════════════════╗"
	@echo "║  Rust Async/Future Compilation Pipeline Explorer                ║"
	@echo "║  Pure rustc — no cargo, no external dependencies               ║"
	@echo "╚══════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "  Source (.rs) → AST → Expanded → HIR → MIR → LLVM IR → Asm → Binary"
	@echo ""
	@echo "  all          - Build the binary (default)"
	@echo "  ast          - [Stage 1] AST tree (nightly)              → .ast"
	@echo "  expanded     - [Stage 2] ⭐ Macro-expanded (nightly)     → .expanded.rs"
	@echo "  hir          - [Stage 3] ⭐ HIR - .await desugared       → .hir"
	@echo "  mir          - [Stage 4] ⭐ MIR - state machine          → .mir"
	@echo "  llvm-ir      - [Stage 5] LLVM IR                         → .ll"
	@echo "  asm          - [Stage 6] Assembly                        → .s"
	@echo "  asm-opt      - [Stage 6] Optimized Assembly (-O3)        → -opt.s"
	@echo "  all-ir       - Generate ALL stages"
	@echo "  peek-future  - Extract Future-related code from expanded"
	@echo "  peek-states  - Extract state machine from MIR"
	@echo "  run          - Build and run"
	@echo "  clean        - Remove build artifacts"
	@echo ""
	@echo "  ⭐ = Most interesting for understanding async/Future"
	@echo ""
	@echo "  Requires: rustup install nightly (for ast, expanded, hir)"

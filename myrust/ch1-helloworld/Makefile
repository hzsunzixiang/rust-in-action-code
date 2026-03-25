# Rust Hello World - Makefile
# Generates binary and all compilation pipeline intermediate files
#
# Compilation Pipeline:
#   Source (.rs) → AST → Expanded (macro-expanded) → HIR → MIR → LLVM IR → Assembly → Binary
#                  ↑         ↑                         ↑     ↑      ↑          ↑
#               ast      expanded                    hir    mir   llvm-ir     asm

RUSTC = rustc
NIGHTLY = rustup run nightly rustc
SRC = src/main.rs
OUT_DIR = build
TARGET = $(OUT_DIR)/hello

.PHONY: all clean ast expanded hir mir llvm-ir asm asm-opt all-ir run help

# Build the binary (default)
all: $(TARGET)

$(OUT_DIR):
	mkdir -p $(OUT_DIR)

# Compile to executable
$(TARGET): $(SRC) | $(OUT_DIR)
	$(RUSTC) $(SRC) -o $(TARGET)

# ====== Pipeline Stage 1: AST (before macro expansion, nightly only) ======
ast: $(SRC) | $(OUT_DIR)
	$(NIGHTLY) $(SRC) -Z unpretty=ast-tree > $(OUT_DIR)/hello.ast 2>&1 || true
	@echo "AST output: $(OUT_DIR)/hello.ast"

# ====== Pipeline Stage 2: Expanded (after macro expansion, nightly only) ======
expanded: $(SRC) | $(OUT_DIR)
	$(NIGHTLY) $(SRC) -Z unpretty=expanded > $(OUT_DIR)/hello.expanded.rs 2>&1 || true
	@echo "Expanded output: $(OUT_DIR)/hello.expanded.rs"

# ====== Pipeline Stage 3: HIR (High-level IR, desugared form, nightly only) ======
hir: $(SRC) | $(OUT_DIR)
	$(NIGHTLY) $(SRC) -Z unpretty=hir > $(OUT_DIR)/hello.hir 2>&1 || true
	@echo "HIR output: $(OUT_DIR)/hello.hir"

# ====== Pipeline Stage 4: MIR (Mid-level IR, control-flow graph) ======
mir: $(SRC) | $(OUT_DIR)
	$(RUSTC) $(SRC) --emit mir -o $(OUT_DIR)/hello.mir
	@echo "MIR output: $(OUT_DIR)/hello.mir"

# ====== Pipeline Stage 5: LLVM IR (.ll file) ======
llvm-ir: $(SRC) | $(OUT_DIR)
	$(RUSTC) $(SRC) --emit llvm-ir -o $(OUT_DIR)/hello.ll
	@echo "LLVM IR output: $(OUT_DIR)/hello.ll"

# ====== Pipeline Stage 6: Assembly (.s file) ======
asm: $(SRC) | $(OUT_DIR)
	$(RUSTC) $(SRC) --emit asm -o $(OUT_DIR)/hello.s
	@echo "Assembly output: $(OUT_DIR)/hello.s"

# ====== Assembly with optimizations (to compare with unoptimized) ======
asm-opt: $(SRC) | $(OUT_DIR)
	$(RUSTC) $(SRC) --emit asm -C opt-level=3 -o $(OUT_DIR)/hello-opt.s
	@echo "Optimized assembly output: $(OUT_DIR)/hello-opt.s"

# ====== Generate ALL intermediate representations ======
all-ir: ast expanded hir mir llvm-ir asm
	@echo ""
	@echo "=== All pipeline intermediate files generated in $(OUT_DIR)/ ==="
	@echo "  Pipeline: .rs → .ast → .expanded.rs → .hir → .mir → .ll → .s"
	@echo ""
	@ls -lh $(OUT_DIR)/hello.ast $(OUT_DIR)/hello.expanded.rs $(OUT_DIR)/hello.hir \
	        $(OUT_DIR)/hello.mir $(OUT_DIR)/hello.ll $(OUT_DIR)/hello.s 2>/dev/null || true

# Run the program
run: $(TARGET)
	./$(TARGET)

# Clean build artifacts
clean:
	rm -rf $(OUT_DIR)

# Show help
help:
	@echo "Rust Compilation Pipeline Targets:"
	@echo ""
	@echo "  Source (.rs) → AST → Expanded → HIR → MIR → LLVM IR → Assembly → Binary"
	@echo ""
	@echo "  all       - Build the binary (default)"
	@echo "  ast       - [Stage 1] AST tree (nightly only)             → .ast"
	@echo "  expanded  - [Stage 2] Macro-expanded source (nightly only)→ .expanded.rs"
	@echo "  hir       - [Stage 3] HIR - High-level IR (nightly only)  → .hir"
	@echo "  mir       - [Stage 4] MIR - Mid-level IR                  → .mir"
	@echo "  llvm-ir   - [Stage 5] LLVM IR                             → .ll"
	@echo "  asm       - [Stage 6] Assembly                            → .s"
	@echo "  asm-opt   - [Stage 6] Optimized Assembly (-O3)            → -opt.s"
	@echo "  all-ir    - Generate ALL stages (ast→expanded→hir→mir→ll→s)"
	@echo "  run       - Build and run"
	@echo "  clean     - Remove build artifacts"
	@echo ""
	@echo "Note: ast, expanded, hir require nightly toolchain (rustup install nightly)"

# Ch1 - Hello World

Rust 入门第一个程序：Hello World。

本项目除了展示最基础的 Rust 程序外，还通过 Makefile 提供了 **Rust 完整编译管线** 各阶段中间产物的生成命令，帮助理解 Rust 从源码到可执行文件的编译过程。

## 源码

```rust
fn main() {
    println!("Hello, World!");
}
```

## Rust 编译管线

```
Source (.rs) → AST → Expanded → HIR → MIR → LLVM IR → Assembly → Binary
```

| 阶段 | 全称 | 生成者 | 说明 |
|------|------|--------|------|
| AST | Abstract Syntax Tree | rustc 前端 | 语法树（宏展开前） |
| Expanded | Macro-expanded Source | rustc 前端 | 宏展开后的源码 |
| HIR | High-level Intermediate Representation | rustc 前端 | 脱糖后的高级 IR，类型检查、trait 解析 |
| MIR | Mid-level Intermediate Representation | rustc 前端 | 控制流图，借用检查、生命周期验证 |
| LLVM IR | LLVM Intermediate Representation | rustc → LLVM | 平台无关优化（内联、常量传播等） |
| Assembly | Target Assembly | LLVM 后端 | 目标平台的机器指令 |

## 快速开始

### 编译并运行

```bash
# 使用 Makefile
make run

# 或直接使用 rustc
rustc src/main.rs -o hello && ./hello
```

### 生成所有中间产物

```bash
make all-ir
```

执行后会在 `build/` 目录下生成：

```
build/
├── hello.ast          # Stage 1: AST 语法树
├── hello.expanded.rs  # Stage 2: 宏展开后的源码
├── hello.hir          # Stage 3: HIR 高级中间表示
├── hello.mir          # Stage 4: MIR 中级中间表示
├── hello.ll           # Stage 5: LLVM IR
└── hello.s            # Stage 6: 汇编
```

### 单独生成某个阶段

```bash
make ast        # Stage 1: AST（nightly）
make expanded   # Stage 2: 宏展开（nightly）
make hir        # Stage 3: HIR（nightly）
make mir        # Stage 4: MIR
make llvm-ir    # Stage 5: LLVM IR
make asm        # Stage 6: 汇编
make asm-opt    # Stage 6: 优化汇编（-O3）
```

### 其他命令

```bash
make help       # 查看所有可用目标
make clean      # 清理构建产物
```

## 前置条件

- **Rust stable**：用于编译、生成 MIR / LLVM IR / Assembly
- **Rust nightly**：用于生成 AST / Expanded / HIR（`-Z unpretty` 仅 nightly 支持）

```bash
# 安装 nightly 工具链
rustup toolchain install nightly
```

## 项目结构

```
ch1-helloworld/
├── Cargo.toml         # 项目配置
├── Makefile           # 编译管线构建脚本
├── README.md          # 本文件
├── src/
│   └── main.rs        # 源码
└── build/             # 编译产物（git ignored）
```

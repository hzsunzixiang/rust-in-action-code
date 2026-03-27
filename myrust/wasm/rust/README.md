# 🦀 Rust WebAssembly 示例项目

一个演示如何将 Rust 编译为 WebAssembly 并在网页中与之交互的示例项目。

## 功能特性

| 示例 | 说明 |
|------|------|
| **Greeting（问候）** | JS 与 Rust 之间的基础字符串传递 |
| **Fibonacci（斐波那契）** | 在 WASM 中执行 CPU 密集型计算 |
| **Canvas Drawing（画布绘制）** | Rust 通过 `web-sys` 在 HTML Canvas 上绘图 |
| **DOM Manipulation（DOM 操作）** | Rust 动态创建并添加 DOM 元素 |
| **Array Sorting（数组排序）** | 在 Rust 中处理数据，将结果返回给 JS |

## 前置条件

- **Rust**（1.70+）：[https://rustup.rs](https://rustup.rs)
- **wasm-pack**：可通过 `make setup` 自动安装，或手动安装：
  ```bash
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
  ```
- **Python 3**：用于启动本地 HTTP 服务器（macOS/Linux 通常已预装）

## 快速开始

```bash
# 1. 安装工具（仅首次需要）
make setup

# 2. 编译并运行
make serve
```

然后在浏览器中打开 [http://localhost:8080](http://localhost:8080) 即可。

## 可用命令

```bash
make setup          # 安装所需工具（wasm-pack、wasm32 target）
make build          # 编译 WASM 模块（debug 模式）
make build-release  # 编译 WASM 模块（release 模式，已优化）
make serve          # 编译 + 启动 HTTP 服务器（端口 8080）
make serve-only     # 仅启动服务器（不重新编译）
make check          # 运行 cargo check 和 clippy 检查
make test           # 运行 cargo 测试
make clean          # 清理所有构建产物
make help           # 显示帮助信息
```

## 项目结构

```
.
├── Cargo.toml      # Rust 包配置文件
├── Makefile        # 编译与运行命令
├── README.md       # 本文件
├── index.html      # Web 前端页面
├── src/
│   └── lib.rs      # Rust 源码（WASM 导出函数）
└── pkg/            # 编译生成的 WASM 输出（编译后生成）
    ├── rust_wasm_demo.js
    ├── rust_wasm_demo_bg.wasm
    └── ...
```

## 工作原理

```
┌─────────────┐    wasm-pack     ┌──────────────┐
│  src/lib.rs  │ ──────────────> │  pkg/*.wasm   │
│  (Rust 代码) │      编译       │  pkg/*.js     │
└─────────────┘                  └──────┬───────┘
                                        │
                                        │ import 导入
                                        ▼
                                 ┌──────────────┐
                                 │  index.html   │
                                 │  (浏览器运行)  │
                                 └──────────────┘
```

1. **Rust 代码**（`src/lib.rs`）使用 `#[wasm_bindgen]` 宏导出函数
2. **wasm-pack** 将 Rust 编译为 `.wasm` 文件并生成 JS 胶水代码
3. **index.html** 导入 JS 模块并调用 Rust 函数

## 核心依赖

- [`wasm-bindgen`](https://github.com/nickel-org/rust-wasm-bindgen) — Rust/JS 互操作桥梁
- [`web-sys`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/) — Web API 绑定（DOM、Canvas、Console）

## 常见问题

**直接打开 `index.html` 时出现 MIME 类型错误：**
> WASM 文件必须通过 HTTP 协议提供服务。请使用 `make serve` 启动服务器，而不是直接打开文件。

**找不到 `wasm-pack` 命令：**
```bash
make setup
# 或者
cargo install wasm-pack
```

**端口 8080 已被占用：**
```bash
PORT=3000 make serve
```

# C++ WebAssembly 运行机制详解

> 本文档说明 C++ 如何通过 Emscripten 编译为 WebAssembly，以及 WASM 在浏览器和 Node.js 中的运行机制。

## 目录

1. [总览：编译与运行管线](#1-总览编译与运行管线)
2. [编译阶段：C++ → WASM](#2-编译阶段c--wasm)
3. [产物分析：.wasm 与 .js 的分工](#3-产物分析wasm-与-js-的分工)
4. [运行时机制：WASM 如何被加载和执行](#4-运行时机制wasm-如何被加载和执行)
5. [函数导出机制：C++ 函数如何暴露给 JS](#5-函数导出机制c-函数如何暴露给-js)
6. [内存模型：C++ 与 JS 如何共享数据](#6-内存模型c-与-js-如何共享数据)
7. [两种运行环境对比：Node.js vs 浏览器](#7-两种运行环境对比nodejs-vs-浏览器)
8. [性能特征](#8-性能特征)
9. [项目结构与命令速查](#9-项目结构与命令速查)

---

## 1. 总览：编译与运行管线

```
                        Emscripten (emcc)
                              │
  ┌──────────┐    ┌───────────┴───────────┐    ┌──────────────────┐
  │ main.cpp │───→│  Clang/LLVM → wasm    │───→│  main.wasm (14K) │  WebAssembly 二进制
  │ (C++)    │    │  backend              │    │  main.js   (15K) │  JS 胶水代码
  └──────────┘    └───────────────────────┘    └────────┬─────────┘
                                                        │
                              ┌──────────────────────────┤
                              │                          │
                       ┌──────▼──────┐           ┌───────▼───────┐
                       │  Node.js    │           │   Browser     │
                       │  (CLI 运行)  │           │  (HTTP 加载)   │
                       └─────────────┘           └───────────────┘
```

**核心思想**：Emscripten 不只是一个编译器，它是一个完整的工具链：
- **前端**：用 Clang 解析 C++ 代码
- **中端**：LLVM 优化
- **后端**：生成 `.wasm` 二进制 + `.js` 胶水代码（处理运行时环境差异）

---

## 2. 编译阶段：C++ → WASM

### 编译命令

```bash
emcc src/main.cpp -o build/main.js \
    -O2 \                                          # 优化级别
    -s WASM=1 \                                    # 输出 WASM（而非 asm.js）
    -s EXPORTED_FUNCTIONS='["_add",...]' \          # 导出哪些 C 函数
    -s EXPORTED_RUNTIME_METHODS='["ccall",...]' \   # 导出哪些运行时辅助方法
    -s ALLOW_MEMORY_GROWTH=1 \                     # 允许动态扩展内存
    -s NO_EXIT_RUNTIME=1                           # main() 结束后不销毁运行时
```

### 编译管线细节

```
main.cpp
  │
  ▼ Clang 前端
C++ AST
  │
  ▼ Clang → LLVM IR
LLVM IR (.ll)
  │
  ▼ LLVM 优化 Pass (-O2)
Optimized LLVM IR
  │
  ▼ LLVM wasm32 后端
main.wasm          ← WebAssembly 二进制（纯计算逻辑）
  +
main.js            ← JavaScript 胶水代码（环境适配 + 模块加载）
```

### 关键编译标志解释

| 标志 | 作用 |
|------|------|
| `-O2` | LLVM 优化：内联、死代码消除、循环优化等 |
| `-s WASM=1` | 输出 `.wasm` 格式（默认值，区别于旧的 `asm.js`） |
| `-s EXPORTED_FUNCTIONS` | 指定哪些 C 函数可被 JS 调用（函数名前加 `_`） |
| `-s EXPORTED_RUNTIME_METHODS` | 暴露 Emscripten 运行时辅助函数（如 `ccall`, `UTF8ToString`） |
| `-s ALLOW_MEMORY_GROWTH=1` | 允许 WASM 线性内存动态增长（`malloc` 需要） |
| `-s NO_EXIT_RUNTIME=1` | `main()` 返回后保持运行时存活，JS 仍可调用导出函数 |

---

## 3. 产物分析：.wasm 与 .js 的分工

### main.wasm — 纯计算引擎

```
┌─────────────────────────────────────────────────┐
│                 main.wasm (14K)                  │
├─────────────────────────────────────────────────┤
│  Type Section     — 函数签名类型定义              │
│  Import Section   — 从 JS 导入的函数（如 printf） │
│  Function Section — 函数索引表                    │
│  Memory Section   — 线性内存声明（初始 256 页）    │
│  Export Section   — 导出给 JS 的函数              │
│  Code Section     — 函数体的 WASM 字节码          │
│  Data Section     — 静态数据（字符串常量等）       │
└─────────────────────────────────────────────────┘
```

WASM 是一种**栈式虚拟机**的二进制指令格式。例如 `add(a, b)` 编译后大致为：

```wasm
;; WAT (WebAssembly Text Format) 伪代码
(func $add (param $a i32) (param $b i32) (result i32)
    local.get $a      ;; 将 a 压栈
    local.get $b      ;; 将 b 压栈
    i32.add           ;; 弹出两个值，相加，结果压栈
)
```

### main.js — 环境适配层

```
┌─────────────────────────────────────────────────┐
│                 main.js (15K)                    │
├─────────────────────────────────────────────────┤
│  Module 对象初始化                                │
│  ├── 内存管理（malloc/free 的 JS 包装）           │
│  ├── 文件系统模拟（printf → console.log）         │
│  ├── 环境检测（Node.js vs Browser）              │
│  ├── WASM 加载与实例化                            │
│  └── 导出函数绑定（Module._add 等）               │
└─────────────────────────────────────────────────┘
```

**为什么需要 JS 胶水代码？** 因为 WASM 本身：
- ❌ 不能直接访问 DOM
- ❌ 不能直接做 I/O（文件、网络、控制台）
- ❌ 不知道自己运行在 Node.js 还是浏览器
- ✅ 只能做纯计算 + 访问线性内存

JS 胶水代码负责桥接这些能力。

---

## 4. 运行时机制：WASM 如何被加载和执行

### 加载流程

```
                    Browser / Node.js
                          │
                    ┌─────▼─────┐
                    │ 加载 main.js │
                    └─────┬─────┘
                          │
                    ┌─────▼──────────────┐
                    │ fetch main.wasm     │  下载/读取 WASM 二进制
                    └─────┬──────────────┘
                          │
                    ┌─────▼──────────────┐
                    │ WebAssembly         │
                    │  .instantiate()     │  验证 + 编译 + 实例化
                    └─────┬──────────────┘
                          │
              ┌───────────┼───────────┐
              │           │           │
        ┌─────▼────┐ ┌───▼────┐ ┌───▼──────┐
        │ 验证字节码 │ │ JIT 编译│ │ 创建实例  │
        │ (类型安全) │ │ (→机器码)│ │ (内存+表) │
        └──────────┘ └────────┘ └──────────┘
                          │
                    ┌─────▼──────────────┐
                    │ 调用 _main()        │  执行 C++ 的 main 函数
                    └─────┬──────────────┘
                          │
                    ┌─────▼──────────────┐
                    │ Module 就绪         │  onRuntimeInitialized 回调
                    │ JS 可调用导出函数    │  Module._add(3, 4) → 7
                    └────────────────────┘
```

### 关键步骤详解

**Step 1: 验证（Validation）**
- 浏览器/Node.js 的 WASM 引擎检查字节码的类型安全性
- 确保没有越界访问、类型不匹配等问题
- 这是 WASM 安全性的基石：**即使 C++ 源码有 bug，WASM 沙箱也能防止逃逸**

**Step 2: 编译（Compilation）**
- WASM 字节码被 JIT 编译为本机机器码（x86/ARM）
- 因为 WASM 是强类型的，编译速度远快于 JavaScript
- V8 引擎使用 Liftoff（快速基线编译）+ TurboFan（优化编译）两层策略

**Step 3: 实例化（Instantiation）**
- 分配线性内存（`WebAssembly.Memory`）
- 绑定导入函数（JS → WASM 的桥接）
- 创建函数表（用于间接调用）

---

## 5. 函数导出机制：C++ 函数如何暴露给 JS

### C++ 侧：标记导出

```cpp
extern "C" {                    // ① 禁用 C++ name mangling
    EMSCRIPTEN_KEEPALIVE        // ② 防止链接器优化掉未引用的函数
    int add(int a, int b) {
        return a + b;
    }
}
```

两个关键要素：

| 机制 | 作用 | 没有会怎样 |
|------|------|-----------|
| `extern "C"` | 函数名保持为 `add`，不被 C++ mangling 为 `_Z3addii` | JS 找不到函数 |
| `EMSCRIPTEN_KEEPALIVE` | 告诉链接器：即使 main 没调用，也不要删除这个函数 | 函数被 dead code elimination 删掉 |

### 编译时：指定导出列表

```makefile
-s EXPORTED_FUNCTIONS='["_main","_add","_multiply","_fibonacci",...]'
```

函数名前加 `_` 是 C 的 ABI 约定（Emscripten 遵循此规则）。

### JS 侧：调用导出函数

```
Module._add(3, 4)                    ← 直接调用（简单类型）
Module.ccall('add', 'number',        ← ccall 方式（自动类型转换）
             ['number','number'],
             [3, 4])
```

### 调用链路

```
JS: Module._add(3, 4)
  │
  ▼
WASM Instance: exports.add(3, 4)
  │
  ▼ (已 JIT 编译为机器码)
Native: add 函数的 x86/ARM 指令
  │
  ▼
返回 i32: 7
  │
  ▼
JS: 收到 Number 7
```

---

## 6. 内存模型：C++ 与 JS 如何共享数据

### WASM 线性内存

WASM 使用一块**连续的字节数组**作为内存（线性内存），C++ 和 JS 都可以读写：

```
┌─────────────────────────────────────────────────────────────┐
│              WebAssembly.Memory (线性内存)                    │
│                                                             │
│  ┌──────┬──────────┬──────────┬──────────────────────────┐  │
│  │ 静态  │  栈空间   │  堆空间   │  可增长区域               │  │
│  │ 数据  │ (局部变量) │ (malloc) │  (ALLOW_MEMORY_GROWTH)  │  │
│  └──────┴──────────┴──────────┴──────────────────────────┘  │
│  0                                                    N bytes│
└─────────────────────────────────────────────────────────────┘
         ↑                    ↑
    C++ 直接访问          JS 通过 Module.HEAPU8 访问
```

### 字符串传递示例：reverse_string

这是本项目中最能体现内存共享机制的例子：

```
JS 侧                              WASM 线性内存                    C++ 侧
──────                              ──────────                      ──────

1. 计算字符串字节长度
   lengthBytesUTF8("Hello") → 5

2. 在 WASM 堆上分配内存
   Module._malloc(6) → ptr=1024     ┌─────────────────┐
                                    │ 1024: [空空空空空0] │
                                    └─────────────────┘

3. 将 JS 字符串写入 WASM 内存
   stringToUTF8("Hello", 1024, 6)   ┌─────────────────┐
                                    │ 1024: [H,e,l,l,o,\0] │
                                    └─────────────────┘

4. 调用 C++ 函数
   Module._reverse_string(1024)                              reverse_string(ptr=1024)
                                                             ├── strlen → 5
                                    ┌─────────────────┐      ├── swap in-place
                                    │ 1024: [o,l,l,e,H,\0] │ ← 直接修改同一块内存
                                    └─────────────────┘

5. 从 WASM 内存读回结果
   UTF8ToString(1024) → "olleH"     ┌─────────────────┐
                                    │ 1024: [o,l,l,e,H,\0] │
                                    └─────────────────┘

6. 释放内存
   Module._free(1024)
```

**关键点**：C++ 和 JS 操作的是**同一块内存**。没有序列化/反序列化，没有拷贝（除了 JS 字符串 ↔ WASM 内存的初始/最终转换）。

### 数组传递示例：array_sum

```javascript
// JS 侧
var ptr = Module._create_array(5);          // C++ malloc，返回指针
var arr = new Int32Array(Module.HEAP32.buffer, ptr, 5);  // 直接映射到 JS TypedArray
arr.set([10, 20, 30, 40, 50]);              // 写入数据（直接写 WASM 内存）
var sum = Module._array_sum(ptr, 5);        // C++ 读取同一块内存，计算求和
Module._free_array(ptr);                    // 释放
```

---

## 7. 两种运行环境对比：Node.js vs 浏览器

### Node.js 运行（`make run`）

```
node build/main.js
  │
  ├── 检测环境：process.versions.node 存在 → Node.js 模式
  ├── 读取 WASM：fs.readFileSync('main.wasm')
  ├── 实例化：WebAssembly.instantiate(buffer, imports)
  ├── printf 映射：fd_write → process.stdout.write
  ├── 调用 main()
  │     ├── printf("[C++ WASM] add(3, 4) = 7")     → stdout
  │     ├── printf("[C++ WASM] fibonacci(10) = 55") → stdout
  │     └── ...
  └── 运行时保持存活（NO_EXIT_RUNTIME=1）
```

### 浏览器运行（`make serve`）

```
Browser 加载 index.html
  │
  ├── 解析 <script> 定义 Module 对象（print/printErr 回调）
  ├── 加载 main.js（Emscripten 胶水代码）
  │     ├── 检测环境：typeof window !== 'undefined' → 浏览器模式
  │     ├── fetch('main.wasm')  → 下载 WASM 二进制
  │     ├── WebAssembly.instantiateStreaming()  → 流式编译（边下载边编译）
  │     ├── printf 映射：fd_write → Module.print → appendLog()
  │     └── 调用 main() → 输出到页面 Console 区域
  │
  ├── onRuntimeInitialized 触发
  │     └── 页面显示 "✅ WASM runtime initialized!"
  │
  └── 用户交互
        ├── 点击 "add(a,b)"      → runAdd()  → Module._add(42, 58)  → 100
        ├── 点击 "fibonacci(n)"   → runFib()  → Module._fibonacci(35) → 9227465
        ├── 点击 "count_primes"   → runPrimes() → Module._count_primes(100000)
        └── 点击 "reverse_string" → runReverse() → malloc → copy → call → read → free
```

### 环境差异对比

| 特性 | Node.js | 浏览器 |
|------|---------|--------|
| WASM 加载方式 | `fs.readFileSync` | `fetch` + `instantiateStreaming` |
| printf 输出 | `process.stdout` | `Module.print` 回调 |
| 文件系统 | 真实 fs | Emscripten 虚拟 FS |
| 内存限制 | V8 堆限制（~4GB） | 浏览器 tab 限制（~2-4GB） |
| 用途 | CLI 工具、测试 | 交互式 Web 应用 |

---

## 8. 性能特征

### WASM vs 原生 vs JavaScript

```
                    性能对比（概念性）
    ┌─────────────────────────────────────────┐
    │  Native C++ (clang -O2)    ████████████ │  100%
    │  WASM (emcc -O2)           █████████    │  ~80-95%
    │  JavaScript (V8 JIT)       ██████       │  ~50-70%
    │  JavaScript (解释执行)      ██           │  ~10-20%
    └─────────────────────────────────────────┘
```

### 为什么 WASM 快？

1. **强类型**：所有变量类型在编译期确定，无需运行时类型检查
2. **AOT 友好**：WASM 字节码接近机器码，JIT 编译极快
3. **无 GC 开销**：手动内存管理（malloc/free），无垃圾回收暂停
4. **可预测性能**：没有 JS 的 deoptimization、hidden class 变化等

### 本项目的性能验证点

| 函数 | 计算特征 | 适合 WASM 的原因 |
|------|---------|-----------------|
| `fibonacci(35)` | 递归密集计算 | 函数调用开销低，无 GC |
| `count_primes(100000)` | 循环密集计算 | 整数运算直接映射到机器指令 |
| `reverse_string` | 内存操作 | 直接指针操作，无字符串对象开销 |
| `array_sum` | 连续内存遍历 | 线性内存 = 缓存友好 |

---

## 9. 项目结构与命令速查

### 目录结构

```
wasm/cpp/
├── Makefile              # 编译、运行、分析的所有命令
├── src/
│   └── main.cpp          # C++ 源码（7 个导出函数 + main）
├── web/
│   ├── index.html        # 浏览器交互界面
│   ├── main.js           # (编译生成) JS 胶水代码
│   └── main.wasm         # (编译生成) WASM 二进制
└── build/
    ├── main.js           # (编译生成)
    └── main.wasm         # (编译生成)
```

### 命令速查

```bash
make              # 编译 C++ → WASM
make run          # Node.js 运行（CLI）
make serve        # 启动 HTTP 服务器，浏览器打开 http://localhost:8080
make debug        # 带调试信息编译（-g + ASSERTIONS=2）
make wat          # 生成 WAT 文本格式（需要 wabt）
make size         # 查看 WASM 二进制大小分析
make inspect      # 查看导出的 WASM 函数列表（需要 wabt）
make clean        # 清理编译产物
make help         # 显示帮助
```

### 导出函数一览

| C++ 函数 | JS 调用方式 | 功能 |
|----------|------------|------|
| `add(a, b)` | `Module._add(a, b)` | 整数加法 |
| `multiply(a, b)` | `Module._multiply(a, b)` | 浮点乘法 |
| `fibonacci(n)` | `Module._fibonacci(n)` | 递归斐波那契 |
| `is_prime(n)` | `Module._is_prime(n)` | 质数判断 |
| `count_primes(n)` | `Module._count_primes(n)` | 统计质数个数 |
| `create_array(size)` | `Module._create_array(n)` | 堆上分配数组 |
| `free_array(ptr)` | `Module._free_array(ptr)` | 释放数组 |
| `array_sum(arr, len)` | `Module._array_sum(ptr, n)` | 数组求和 |
| `reverse_string(str)` | `Module._reverse_string(ptr)` | 原地反转字符串 |

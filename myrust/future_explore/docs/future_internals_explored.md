# Future 的真面目：Rust async/await 编译器变换探索

> 写日常惯用的 async Rust 代码，然后通过编译器的"X光"来看内部机理。
> 所有中间产物均可通过 `make all-ir` 自行生成。
>
> **纯 rustc 编译，无任何外部依赖。~80 行惯用代码。**

## 目录

1. [源代码：日常写法](#1-源代码日常写法)
2. [编译管线总览](#2-编译管线总览)
3. [Stage 2 — Expanded：async fn 还在](#3-stage-2--expandedasync-fn-还在)
4. [Stage 3 — HIR：.await 变成了什么](#4-stage-3--hirawait-变成了什么)
5. [Stage 4 — MIR：状态机现形](#5-stage-4--mir状态机现形)
6. [还原：编译器到底生成了什么](#6-还原编译器到底生成了什么)
7. [运行输出与内存布局](#7-运行输出与内存布局)
8. [总结](#8-总结)

---

## 1. 源代码：日常写法

代码就是你日常怎么写就怎么写，没有任何手写 Future 或状态机：

```rust
async fn add_one(x: u32) -> u32 {
    x + 1
}

async fn double(x: u32) -> u32 {
    x * 2
}

// 2 个 await → 编译器生成 2 个挂起状态
async fn compute(x: u32) -> u32 {
    let a = add_one(x).await;
    double(a).await
}

// 3 个 await → 编译器生成 3 个挂起状态
async fn pipeline(x: u32) -> String {
    let step1 = add_one(x).await;
    let step2 = double(step1).await;
    let step3 = add_one(step2).await;
    format!("pipeline({}) = {}", x, step3)
}

// 嵌套 async → 编译器递归展开
async fn nested(x: u32) -> u32 {
    let a = compute(x).await;
    let b = compute(a).await;
    a + b
}
```

问题：这些 `async fn` 在编译器眼中到底变成了什么？

---

## 2. 编译管线总览

```
Source (.rs) → AST → Expanded → HIR → MIR → LLVM IR → Assembly → Binary
               ↑        ↑         ↑     ↑
            语法树   宏展开     脱糖   状态机
                     (async     (.await  (真正的
                      fn 还在)   变 loop)  enum!)
```

| 阶段 | 文件 | 看什么 |
|------|------|--------|
| **Expanded** | `.expanded.rs` | `async fn` 仍保留，只有宏被展开 |
| **HIR** | `.hir` | `.await` 被脱糖为 `loop { match poll() { ... } }` |
| **MIR** | `.mir` | 真正的状态机 enum，`discriminant` 状态切换 |

---

## 3. Stage 2 — Expanded：async fn 还在

运行 `make expanded`，查看 `build/future_explore.expanded.rs`：

```rust
// async fn 在 expanded 阶段完全保留！
async fn add_one(x: u32) -> u32 { x + 1 }
async fn double(x: u32) -> u32 { x * 2 }
async fn compute(x: u32) -> u32 { let a = add_one(x).await; double(a).await }
```

**发现**：expanded 阶段只展开了宏（`println!` → `_print(format_args!(...))`），`async fn` 和 `.await` 原封不动。真正的变换在下一阶段。

---

## 4. Stage 3 — HIR：.await 变成了什么

运行 `make hir`，查看 `build/future_explore.hir`。以 `compute` 为例：

```rust
// 原始代码：
//   async fn compute(x: u32) -> u32 {
//       let a = add_one(x).await;
//       double(a).await
//   }

// HIR 脱糖后：
async fn compute(x: u32)
    -> /*impl Trait*/ |mut _task_context: ResumeTy|
{
    let x = x;
    {
        let a =
            match into_future(add_one(x)) {       // 调用 add_one(x)，得到 Future
                mut __awaitee =>
                    loop {
                        match unsafe {
                            poll(new_unchecked(&mut __awaitee),  // Pin 住
                                 get_context(_task_context))     // 传入 Context
                        } {
                            Ready { 0: result } => break result, // ✅ 完成 → break
                            Pending {} => { }                    // ⏳ 没好 → 继续
                        }
                        _task_context = (yield ());              // 💤 让出控制权
                    },
            };

        // 第二个 .await 同理
        match into_future(double(a)) {
            mut __awaitee =>
                loop {
                    match unsafe {
                        poll(new_unchecked(&mut __awaitee),
                             get_context(_task_context))
                    } {
                        Ready { 0: result } => break result,
                        Pending {} => { }
                    }
                    _task_context = (yield ());
                },
        }
    }
}
```

**发现**：每个 `.await` 都被脱糖为同一个模式：

```
loop {
    match poll(pin(&mut future), context) {
        Ready(result) => break result,   // 拿到结果，继续执行
        Pending       => {}              // 还没好
    }
    yield ();                            // 让出 CPU，等待唤醒
}
```

这就是 `.await` 的全部秘密——一个 `loop + poll + yield`。

---

## 5. Stage 4 — MIR：状态机现形

运行 `make mir`，查看 `build/future_explore.mir`。这里终于看到了**真正的状态机**。

### 5.1 async fn 的返回值

```
fn compute(_1: u32) -> {async fn body of compute()} {
    bb0: {
        _0 = {coroutine@src/main.rs:23:33: 26:2} { x: copy _1 };
        return;
    }
}
```

**关键发现**：`compute(5)` 不执行任何计算！它只是创建一个 coroutine 对象，把参数 `x` 存进去，然后立即返回。真正的计算在 `poll()` 时才发生。

### 5.2 poll 函数：状态机入口

```
fn compute::{closure#0}(
    _1: Pin<&mut {async fn body of compute()}>,   // self（被 Pin 住）
    _2: &mut Context<'_>                           // waker/context
) -> Poll<u32>
{
    bb0: {
        _22 = discriminant((*_23));                // 读取当前状态
        switchInt(move _22) -> [
            0: bb1,     // State 0: 未开始（Unresumed）
            1: bb25,    // State 1: 已完成（Returned）→ panic
            2: bb24,    // State 2: 已 panic（Panicked）→ panic
            3: bb22,    // State 3: 在 add_one().await 处挂起
            4: bb23,    // State 4: 在 double().await 处挂起
        ];
    }
```

**关键发现**：`compute` 有 2 个 `.await`，编译器生成了 **5 个状态**（0=未开始, 1=完成, 2=panic, 3=第一个await挂起, 4=第二个await挂起）。

### 5.3 State 0 → 首次执行

```
bb1: {  // State 0: 第一次被 poll
    _3 = copy ((*_23).0: u32);                              // 读取参数 x
    _5 = add_one(copy _3) -> bb2;                           // 调用 add_one(x)
}

bb2: {
    _4 = into_future(move _5) -> bb3;                       // 转为 Future
}

bb3: {
    (((*_23) as variant#3).0: {async fn body of add_one()}) = move _4;  // 存入 State 3
}

bb5: {
    _6 = <add_one as Future>::poll(move _7, copy _9);       // poll 内部 Future
}

bb6: {
    _10 = discriminant(_6);
    switchInt(move _10) -> [0: bb9, 1: bb8];                // Ready or Pending?
}
```

### 5.4 Pending → 保存状态

```
bb8: {
    _0 = Poll::<u32>::Pending;       // 返回 Pending
    discriminant((*_23)) = 3;        // 设置状态为 3（在 add_one.await 处挂起）
    return;                          // 让出控制权
}
```

### 5.5 Ready → 继续执行下一个 await

```
bb9: {
    _11 = copy ((_6 as Ready).0: u32);                      // 取出 add_one 的结果 → a
    drop(add_one_future);                                    // 释放已完成的 Future
}

bb10: {
    _14 = double(copy _11) -> bb11;                          // 调用 double(a)
}

bb12: {
    (((*_23) as variant#4).0: {async fn body of double()}) = move _13;  // 存入 State 4
}

bb14: {
    _15 = <double as Future>::poll(move _16, copy _18);      // poll double
}
```

### 5.6 第二个 await 的 Pending/Ready

```
bb16: {  // Pending
    _0 = Poll::<u32>::Pending;
    discriminant((*_23)) = 4;        // 设置状态为 4（在 double.await 处挂起）
    return;
}

bb17: {  // Ready
    _20 = copy ((_15 as Ready).0: u32);    // 取出 double 的结果
}

bb18: {
    _0 = Poll::<u32>::Ready(copy _20);     // 返回最终结果
    discriminant((*_23)) = 1;              // 设置状态为 1（Returned）
    return;
}
```

### 5.7 安全守卫

```
bb24: {  // State 2: Panicked
    assert(const false, "`async fn` resumed after panicking");
}

bb25: {  // State 1: Returned
    assert(const false, "`async fn` resumed after completion");
}
```

### 5.8 恢复执行（从挂起状态回来）

```
bb22: {  // State 3: 从 add_one.await 恢复
    goto -> bb4;    // 跳回去重新 poll add_one
}

bb23: {  // State 4: 从 double.await 恢复
    goto -> bb13;   // 跳回去重新 poll double
}
```

---

## 6. 还原：编译器到底生成了什么

综合 HIR 和 MIR 的信息，我们可以还原出编译器为 `compute` 生成的等价结构：

```rust
// ═══════════════════════════════════════════════════════════════
// 编译器为 async fn compute(x: u32) -> u32 生成的（伪代码还原）
// ═══════════════════════════════════════════════════════════════

enum ComputeStateMachine {
    Unresumed  { x: u32 },                                    // State 0
    Returned,                                                  // State 1
    Panicked,                                                  // State 2
    Suspend0   { __awaitee: AddOneFuture },                    // State 3: 在 add_one.await
    Suspend1   { __awaitee: DoubleFuture },                    // State 4: 在 double.await
}

impl Future for ComputeStateMachine {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        match self.discriminant {
            0 => {  // Unresumed: 首次执行
                let x = self.x;
                let add_one_future = add_one(x).into_future();
                *self = Suspend0 { __awaitee: add_one_future };
                // fall through to poll it
                match self.__awaitee.poll(cx) {
                    Pending  => { self.discriminant = 3; return Pending; }
                    Ready(a) => {
                        drop(self.__awaitee);
                        let double_future = double(a).into_future();
                        *self = Suspend1 { __awaitee: double_future };
                        // fall through to poll it
                        match self.__awaitee.poll(cx) {
                            Pending  => { self.discriminant = 4; return Pending; }
                            Ready(v) => { self.discriminant = 1; return Ready(v); }
                        }
                    }
                }
            }
            3 => {  // Suspend0: 从 add_one.await 恢复
                match self.__awaitee.poll(cx) {
                    Pending  => return Pending,
                    Ready(a) => { /* 同上：创建 double future，继续 */ }
                }
            }
            4 => {  // Suspend1: 从 double.await 恢复
                match self.__awaitee.poll(cx) {
                    Pending  => return Pending,
                    Ready(v) => { self.discriminant = 1; return Ready(v); }
                }
            }
            1 => panic!("`async fn` resumed after completion"),
            2 => panic!("`async fn` resumed after panicking"),
        }
    }
}
```

### 规律总结

| async fn | await 数量 | 状态数 | 公式 |
|----------|-----------|--------|------|
| `add_one` | 0 | 3 | 0 + 3（Unresumed + Returned + Panicked） |
| `compute` | 2 | 5 | 2 + 3 |
| `pipeline` | 3 | 6 | 3 + 3 |
| `nested` | 2 | 5 | 2 + 3 |

**公式：状态数 = await 数量 + 3**（固定的 Unresumed、Returned、Panicked）

---

## 7. 运行输出与内存布局

```
compute(5)  = 12
pipeline(3) = pipeline(3) = 9
nested(5)   = 38

--- Future sizes (state machine enum size) ---
add_one(0)  = 8 bytes
double(0)   = 8 bytes
compute(0)  = 16 bytes
pipeline(0) = 20 bytes
nested(0)   = 28 bytes
```

### 内存布局分析

```
add_one(x) → 8 bytes
┌──────────────────────────┐
│ discriminant: u32  (tag) │  4 bytes
│ x: u32                   │  4 bytes
└──────────────────────────┘
（没有 await，最大 variant 只有参数 x）

compute(x) → 16 bytes
┌──────────────────────────────────────┐
│ discriminant: u32                    │  4 bytes
│ max variant payload:                 │
│   Suspend0: { add_one_future: 8B }  │  8 bytes
│   Suspend1: { double_future: 8B }   │
│ padding                              │  4 bytes
└──────────────────────────────────────┘
（最大 variant 包含一个内部 Future = 8B）

pipeline(x) → 20 bytes
┌──────────────────────────────────────┐
│ discriminant + x + 最大 variant      │
│ 3 个 await，需要保存 step1/step2     │
│ 跨 await 的局部变量 + 内部 Future    │
└──────────────────────────────────────┘

nested(x) → 28 bytes
┌──────────────────────────────────────┐
│ discriminant + 最大 variant          │
│ 内部包含 compute 的 Future (16B)     │
│ + 跨 await 的 a: u32                │
└──────────────────────────────────────┘
（嵌套 async 的大小 = 外层开销 + 内层 Future 大小）
```

**关键洞察**：
- Future 的大小 = `discriminant` + 最大 variant 的大小
- 嵌套 async 会递归包含内层 Future，所以 `nested` (28B) > `compute` (16B)
- 跨 await 的局部变量越多，Future 越大

---

## 8. 总结

### async fn 的完整旅程

```
┌─────────────────────────────────────────────────────────┐
│  你写的代码                                               │
│                                                         │
│  async fn compute(x: u32) -> u32 {                      │
│      let a = add_one(x).await;                          │
│      double(a).await                                    │
│  }                                                      │
└────────────────────┬────────────────────────────────────┘
                     │
    ┌────────────────┼────────────────┐
    ▼                ▼                ▼
 Expanded          HIR              MIR
 (不变)         (.await 脱糖)     (状态机)
                     │                │
                     ▼                ▼
              loop {           enum {
                poll()           Unresumed { x },
                Ready→break      Returned,
                Pending→yield    Panicked,
              }                  Suspend0 { add_one_fut },
                                 Suspend1 { double_fut },
                               }
```

### 核心要点

| 你写的 | 编译器做的 |
|--------|-----------|
| `async fn compute(x)` | 创建一个 enum 实例，存入参数 `x`，不执行任何代码 |
| `.await` | `loop { match poll() { Ready → break, Pending → yield } }` |
| 跨 await 的局部变量 `a` | 保存在 enum variant 的字段中 |
| 多个 `.await` | 每个 await 对应一个 Suspend variant |
| 嵌套 `compute(x).await` | 外层 enum 的 variant 中包含内层 enum |

### 零成本抽象的证据

- **无堆分配**：状态机大小编译期确定，全部在栈上
- **无虚函数调用**：`poll()` 是单态化的，编译器内联优化
- **无额外开销**：你手写状态机也不会比编译器生成的更小

### 如何自己验证

```bash
cd future_explore
make run          # 运行，看输出
make all-ir       # 生成所有编译阶段
make peek-future  # 从 expanded 中提取 Future 相关代码
make peek-states  # 从 MIR 中提取状态机转换

# 重点看这三个文件：
cat build/future_explore.expanded.rs   # async fn 宏展开
cat build/future_explore.hir           # .await 脱糖为 loop+poll
cat build/future_explore.mir           # 真正的状态机
```

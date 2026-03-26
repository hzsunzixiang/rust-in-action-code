# async_basic.rs 深度解析

> 对应源码: `src/async_basic.rs`
> 运行: `cargo run --bin async_basic`

---

## 一、本质是什么？协程 / 线程 / 异步IO？

### 直接回答：Rust async/await = **无栈协程 (Stackless Coroutine) + 异步IO**

| 维度 | C++ | Go | Erlang | Rust |
|------|-----|----|--------|------|
| **并发模型** | OS 线程 / C++20 协程 | goroutine (有栈协程) | 轻量级进程 (Actor) | 无栈协程 (Future) |
| **调度器** | OS 内核调度 | Go runtime (GMP 调度器) | BEAM VM 调度器 | tokio runtime (用户态调度) |
| **栈** | 每线程 ~8MB 栈 | 初始 2KB，可动态增长到 1GB | 每进程 ~2KB 堆栈 | **无栈**，状态编译为状态机 |
| **切换开销** | ~1-10μs (内核态切换) | ~100-200ns (用户态) | ~0.1-1μs (VM 调度) | **~10-50ns** (纯用户态，无系统调用) |
| **可创建数量** | ~千级 (受内存限制) | ~十万到百万级 | ~百万级 | **~百万级** |
| **IO 模型** | 阻塞IO / epoll 手动管理 | netpoller (自动异步化) | 内置异步IO | 基于 epoll/kqueue 的异步IO |
| **挂起方式** | 手动 / co_await | **隐式**（runtime 自动） | 隐式（VM 自动） | **显式**（必须写 .await） |
| **函数染色** | 有 (co_await) | **无** ✅ | 无 | 有 (async/await) |

### 深入理解

**Rust 的 `async fn` 本质上是一个状态机生成器。** 编译器会把你写的 async 函数转换成一个实现了 `Future` trait 的状态机结构体：

```rust
// 你写的代码：
async fn fetch_data(id: u32) -> String {
    println!("Starting...");
    sleep(Duration::from_millis(100)).await;  // ← 挂起点
    println!("Done!");
    format!("Data {}", id)
}

// 编译器大致生成的（伪代码）：
enum FetchDataFuture {
    State0 { id: u32 },                          // 初始状态
    State1 { id: u32, sleep_future: SleepFuture }, // 等待 sleep 完成
    Done,
}

impl Future for FetchDataFuture {
    type Output = String;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<String> {
        match self.state {
            State0 => {
                println!("Starting...");
                let sleep_fut = sleep(Duration::from_millis(100));
                self.state = State1 { sleep_future: sleep_fut };
                // 尝试 poll sleep_future...
            }
            State1 => {
                match self.sleep_future.poll(cx) {
                    Poll::Pending => Poll::Pending,  // 还没好，让出控制权
                    Poll::Ready(()) => {
                        println!("Done!");
                        Poll::Ready(format!("Data {}", self.id))
                    }
                }
            }
        }
    }
}
```

**关键区别：**

- **C++ 线程**：每个线程有独立的调用栈，切换时需要保存/恢复整个栈帧，经过内核
- **Go goroutine**：有栈协程，初始 2KB 栈可动态增长，Go runtime 在用户态调度，挂起/恢复通过保存/恢复栈指针实现
- **Erlang 进程**：BEAM VM 管理的轻量级进程，有自己的小堆栈，VM 负责调度
- **Rust Future**：**没有栈！** 所有需要跨 `.await` 保存的变量直接编译进结构体字段，切换就是改个枚举值

### 为什么要强调"无栈"？—— 无栈 vs 有栈协程深度解析

"栈"指的是**调用栈 (call stack)**，它决定了协程挂起时如何保存执行状态。这是理解 Rust async 设计哲学的关键。

#### 核心区别：挂起时"状态存在哪里"

```
┌─────────────────────────────────────────────────────────────────┐
│                    有栈协程 (Stackful)                           │
│                                                                 │
│  每个协程有自己独立的调用栈（一块连续内存）                         │
│                                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                      │
│  │ 协程A的栈 │  │ 协程B的栈 │  │ 协程C的栈 │                      │
│  │ ┌──────┐ │  │ ┌──────┐ │  │ ┌──────┐ │                      │
│  │ │frame3│ │  │ │frame2│ │  │ │frame1│ │                      │
│  │ │frame2│ │  │ │frame1│ │  │ │      │ │                      │
│  │ │frame1│ │  │ │      │ │  │ │      │ │                      │
│  │ └──────┘ │  │ └──────┘ │  │ └──────┘ │                      │
│  └──────────┘  └──────────┘  └──────────┘                      │
│   预分配 2KB+    预分配 2KB+    预分配 2KB+                       │
│                                                                 │
│  挂起 = 保存栈指针(SP)和程序计数器(PC)                             │
│  恢复 = 恢复 SP 和 PC，继续在自己的栈上执行                        │
│                                                                 │
│  代表：Go goroutine, Erlang进程, Lua协程, Java虚拟线程            │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    无栈协程 (Stackless)                          │
│                                                                 │
│  没有独立的栈！编译器把协程变成一个状态机结构体                      │
│                                                                 │
│  enum FetchDataFuture {          // 编译器生成                   │
│      State0 { id: u32 },        // .await 之前的状态             │
│      State1 { id: u32,          // .await 之后的状态             │
│               sleep: SleepFut },                                │
│      Done,                                                      │
│  }                                                              │
│  // 大小 = 所有状态中最大的那个变体                                │
│  // 通常只有几十~几百字节                                         │
│                                                                 │
│  挂起 = 从 poll() 返回 Pending（普通函数返回）                     │
│  恢复 = 再次调用 poll()（普通函数调用）                            │
│                                                                 │
│  代表：Rust async/await, C++20 co_await, JS async/await         │
│        Python asyncio, Kotlin suspend                           │
└─────────────────────────────────────────────────────────────────┘
```

#### 编译器如何精确分析需要保存的变量

```rust
async fn fetch_data(id: u32) -> String {
    let prefix = format!("Task-{}", id);   // 局部变量 1
    sleep(Duration::from_millis(100)).await; // ← 挂起点 1
    let result = do_work(&prefix).await;    // ← 挂起点 2
    format!("{}: {}", prefix, result)        // prefix 跨越了两个 await
}
```

```
编译器分析过程：

  ┌───────────┬──────────────┬──────────────────────────┐
  │ 变量      │ 跨越 await？ │ 需要保存到状态机？        │
  ├───────────┼──────────────┼──────────────────────────┤
  │ id        │ 否（await前用完）│ 否 ✗                 │
  │ prefix    │ 是（两个await后还用）│ 是 ✓            │
  │ result    │ 否（第二个await后立即用）│ 否 ✗         │
  │ sleep_fut │ 是（第一个await期间）│ 是 ✓             │
  │ work_fut  │ 是（第二个await期间）│ 是 ✓             │
  └───────────┴──────────────┴──────────────────────────┘

  生成的状态机（伪代码）：
  enum FetchDataFuture {
      State0 { id: u32 },                                    // 8 bytes
      State1 { prefix: String, sleep_fut: SleepFuture },     // ~40 bytes
      State2 { prefix: String, work_fut: WorkFuture },       // ~48 bytes
      Done,
  }
  // 总大小 = max(State0, State1, State2) ≈ 48 bytes
  // 而不是预分配一个 2KB~8MB 的栈！
```

#### 有栈 vs 无栈：嵌套调用时的区别

假设有嵌套调用链 `a() → b() → c()`：

```
有栈协程（Go 的做法）：
  ┌─────────────┐ ← 栈顶 (SP)
  │  c() 的栈帧  │  局部变量、返回地址
  ├─────────────┤
  │  b() 的栈帧  │  局部变量、返回地址
  ├─────────────┤
  │  a() 的栈帧  │  局部变量、返回地址
  ├─────────────┤
  │  (空闲空间)  │  ← 预分配但未使用，浪费！
  └─────────────┘ ← 栈底

  挂起时：保存 SP 指针，切换到调度器的栈
  恢复时：恢复 SP 指针，从 c() 中间继续执行
  优点：可以在 c() 内部任意位置挂起，调用者完全不知道
  缺点：栈内存必须预先分配，即使只用了 16 字节也要预留几 KB

无栈协程（Rust 的做法）：
  编译器生成嵌套的状态机结构体：
  AFuture { state, BFuture { state, CFuture { state, SleepFuture } } }
  // 整个结构体大小 = 编译期确定，精确到字节
  // sizeof(AFuture) ≈ sizeof(BFuture) + 几字节 ≈ 可能就 64 字节

  挂起时：poll() 函数正常 return Pending，回到调用者（调度器）
  恢复时：调度器再次调用 poll()，状态机根据当前 state 跳到正确位置
```

#### 关键差异对比表

| 维度 | 有栈 (Go/Erlang) | 无栈 (Rust/C++20) |
|------|-------------------|-------------------|
| **内存开销** | 每协程预分配 2KB+ | **仅存状态机字段**，几十~几百字节 |
| **百万协程内存** | Go: 2GB+, Erlang: ~2GB | Rust: **几十~几百MB** |
| **挂起机制** | 保存/恢复栈指针 (类似 setjmp/longjmp) | 普通函数返回 `Poll::Pending` |
| **恢复机制** | 跳转到保存的栈位置 | 普通函数调用 `poll()` |
| **挂起位置** | **任意位置**，包括深层嵌套调用 | **只能在 `.await` 点**挂起 |
| **栈溢出风险** | 有（栈空间有限，需要栈增长机制） | **无**（用的是调用者的栈） |
| **编译器参与** | 少（运行时负责） | **重度参与**（编译器生成状态机） |
| **切换开销** | ~100-200ns（栈切换） | **~10-50ns**（纯函数调用） |
| **缓存友好** | 差（栈分散在内存各处） | **好**（结构体可紧凑排列） |
| **可调试性** | 好（有完整调用栈） | 差（调用栈是"扁平"的） |

#### 为什么这个区别如此重要？

**1. 内存效率差 40~100 倍**

```
百万并发连接的内存开销：

Go goroutine:   1,000,000 × 2KB  = 2 GB    （还可能增长到更多）
Rust Future:    1,000,000 × 48B  = 48 MB   （精确，不会增长）
                                    ↑
                              差了 ~40 倍！
```

**2. 切换开销**

```
有栈协程切换（Go）：
  1. 保存当前 goroutine 的寄存器（SP, PC, 通用寄存器）
  2. 将栈指针切换到目标 goroutine 的栈
  3. 恢复目标 goroutine 的寄存器
  4. 跳转到目标 goroutine 的 PC
  → 需要汇编指令，~100-200ns

无栈协程切换（Rust）：
  1. poll() 函数返回 Poll::Pending  ← 就是一个普通的 return！
  2. 调度器调用下一个 Future 的 poll() ← 就是一个普通的函数调用！
  → 纯粹的函数调用/返回，~10-50ns
  → 编译器甚至可以内联优化，开销趋近于零
```

**3. 编译器优化空间**

```rust
// 因为状态机就是一个普通的 enum + match，编译器可以：
// 1. 内联：把小的 Future 直接内联到调用者
// 2. 消除状态机：如果 Future 只有一个 await，可能直接优化掉枚举
// 3. 精确大小：sizeof 在编译期确定，可以栈上分配
// 4. 死代码消除：不需要的状态分支可以被优化掉
// 有栈协程做不到这些，因为栈是运行时的东西，编译器看不到
```

#### 无栈的代价

| 代价 | 原因 | 表现 |
|------|------|------|
| **必须写 `.await`** | 编译器需要知道哪里是挂起点 | 有栈协程可以在任意位置透明挂起 |
| **函数染色** | async fn 返回 Future，不是直接返回值 | sync 函数不能直接调用 async 函数 |
| **Pin 的复杂性** | 状态机可能包含自引用（跨 await 的引用） | 需要 `Pin<&mut Self>` 保证地址不变 |
| **递归 async 困难** | 递归导致状态机类型无限嵌套 | 需要 `Box::pin()` 打断类型递归 |
| **调试困难** | 没有真正的调用栈 | backtrace 看不到嵌套关系 |

对比 Go 的有栈协程——**挂起对调用者完全透明**：

```go
// Go: 可以在任意深度挂起，调用者完全无感
func a() { b() }           // 不需要 await！
func b() { c() }           // 不需要 await！
func c() { time.Sleep(1) } // 自动挂起，调用者不知道
// 代价：每个 goroutine 至少 2KB 栈，且需要栈增长机制
```

```rust
// Rust: 必须在每一层显式 .await
async fn a() { b().await; }  // 必须写 .await
async fn b() { c().await; }  // 必须写 .await
async fn c() { sleep(Duration::from_secs(1)).await; }
// 好处：零开销，编译器精确知道哪里会挂起
```

#### 一句话总结

> **"无栈"的本质**：编译器在编译期把"函数执行到一半的状态"从一块预分配的栈内存（运行时、粗粒度、浪费）变成了一个精确的结构体（编译期、字节级、零浪费）。这就是 Rust "零成本抽象"哲学的极致体现。
>
> **类比**：有栈协程像是给每个工人分配一个固定大小的储物柜（不管放多少东西都占那么大）；无栈协程像是给每个工人一个**量身定制的工具袋**（装多少东西就多大，一个口袋都不浪费）。编译器就是那个"裁缝"。

---

## 二、逐段代码解析

### 2.1 `async fn` + `.await` 基础

```rust
async fn fetch_data(id: u32) -> String {
    println!("  [Task {}] Starting fetch...", id);
    sleep(Duration::from_millis(100 * id as u64)).await;
    println!("  [Task {}] Fetch complete!", id);
    format!("Data from task {}", id)
}
```

#### C++ 对比

```cpp
// C++ 传统线程方式
std::string fetch_data(int id) {
    std::cout << "Starting fetch " << id << std::endl;
    std::this_thread::sleep_for(std::chrono::milliseconds(100 * id));  // 阻塞整个OS线程！
    std::cout << "Fetch complete " << id << std::endl;
    return "Data from task " + std::to_string(id);
}

// C++20 协程方式（语法复杂得多）
task<std::string> fetch_data(int id) {
    std::cout << "Starting fetch " << id << std::endl;
    co_await async_sleep(100ms * id);  // 类似 Rust 的 .await
    co_return "Data from task " + std::to_string(id);
}
```

#### Go 对比

```go
// Go: goroutine + time.Sleep
// 看起来像同步代码，但 Go runtime 自动处理调度
func fetchData(id int) string {
    fmt.Printf("Starting fetch %d\n", id)
    time.Sleep(time.Duration(100*id) * time.Millisecond)  // 挂起 goroutine，不阻塞 OS 线程
    fmt.Printf("Fetch complete %d\n", id)
    return fmt.Sprintf("Data from task %d", id)
}
// 注意：不需要 async/await！Go 的 runtime 自动在 time.Sleep 时切换 goroutine
// 这就是有栈协程的优势：挂起对调用者完全透明
```

#### Erlang 对比

```erlang
%% Erlang: 每个"任务"就是一个轻量级进程
fetch_data(Id) ->
    io:format("Starting fetch ~p~n", [Id]),
    timer:sleep(100 * Id),  %% 只阻塞当前 Erlang 进程，不阻塞 OS 线程
    io:format("Fetch complete ~p~n", [Id]),
    lists:flatten(io_lib:format("Data from task ~p", [Id])).
```

#### 核心差异

| | C++ `sleep_for` | Go `time.Sleep` | Erlang `timer:sleep` | Rust `sleep().await` |
|---|---|---|---|---|
| 阻塞什么？ | **OS 线程** | goroutine（不阻塞 OS 线程） | Erlang 进程 | **什么都不阻塞** |
| 线程状态 | 线程挂起，不能做别的 | Go runtime 切换到其他 goroutine | BEAM 调度器切换到其他进程 | tokio 调度器切换到其他 Future |
| 底层机制 | 系统调用 nanosleep | Go netpoller + runtime 调度 | BEAM VM 定时器 | epoll/kqueue 定时器 + 状态机切换 |
| 需要特殊语法？ | 否 | **否** ✅ | 否 | **是**（必须写 .await） |

---

### 2.2 顺序 vs 并发执行（600ms vs 300ms）

```rust
// 顺序执行：一个接一个
async fn sequential_demo() {
    let a = fetch_data(1).await;   // 等 100ms
    let b = fetch_data(2).await;   // 再等 200ms
    let c = fetch_data(3).await;   // 再等 300ms
    // 总计: 100 + 200 + 300 = 600ms
}

// 并发执行：同时跑
async fn concurrent_demo() {
    let (a, b, c) = tokio::join!(
        fetch_data(1),   // 100ms ─┐
        fetch_data(2),   // 200ms ─┤ 同时进行
        fetch_data(3),   // 300ms ─┘
    );
    // 总计: max(100, 200, 300) = 300ms
}
```

#### 时间线图解

```
顺序执行 (600ms):
  Task1: |████|
  Task2:       |████████|
  Task3:                  |████████████|
  Time:  0   100   200   300   400   500   600ms

并发执行 (300ms):
  Task1: |████|
  Task2: |████████|
  Task3: |████████████|
  Time:  0   100   200   300ms
```

#### C++ 对比

```cpp
// C++ 要实现并发，必须用线程或 std::async
auto f1 = std::async(std::launch::async, fetch_data, 1);  // 启动 OS 线程
auto f2 = std::async(std::launch::async, fetch_data, 2);  // 又一个 OS 线程
auto f3 = std::async(std::launch::async, fetch_data, 3);  // 又一个 OS 线程
auto a = f1.get();  // 等待
auto b = f2.get();
auto c = f3.get();
// 问题：3 个 OS 线程，每个 8MB 栈 = 24MB 内存开销
```

#### Go 对比

```go
// Go: 用 goroutine + channel 或 WaitGroup 实现并发
func concurrentDemo() {
    type result struct {
        id   int
        data string
    }
    ch := make(chan result, 3)

    for i := 1; i <= 3; i++ {
        go func(id int) {
            ch <- result{id, fetchData(id)}  // goroutine 自动并发
        }(i)
    }

    // 收集结果
    for i := 0; i < 3; i++ {
        r := <-ch
        fmt.Printf("Got: %s\n", r.data)
    }
    // 总计 ≈ 300ms（并发执行）
}
// Go 的优势：语法简洁，不需要 async/await
// Go 的劣势：channel 的类型不如 Rust join! 的元组精确
//           返回值顺序不确定，需要自己排序
```

#### Erlang 对比

```erlang
%% Erlang: 天然并发，spawn 3 个进程
concurrent_demo() ->
    Self = self(),
    spawn(fun() -> Self ! {1, fetch_data(1)} end),
    spawn(fun() -> Self ! {2, fetch_data(2)} end),
    spawn(fun() -> Self ! {3, fetch_data(3)} end),
    %% 收集结果
    A = receive {1, V1} -> V1 end,
    B = receive {2, V2} -> V2 end,
    C = receive {3, V3} -> V3 end,
    {A, B, C}.
%% Erlang 进程很轻量（~2KB），但没有 join! 这种优雅的语法
```

#### 四种语言并发方式对比

| | C++ | Go | Erlang | Rust |
|---|---|---|---|---|
| 并发原语 | `std::async` | `go func()` | `spawn(fun)` | `tokio::join!` |
| 创建开销 | 重（OS 线程） | 轻（2KB goroutine） | 轻（~2KB 进程） | **最轻**（无栈状态机） |
| 收集结果 | `future.get()` | `<-channel` | `receive` | **元组解构** ✅ |
| 类型安全 | 弱 | 弱（interface{}） | 无 | **强**（编译期） |
| 结果顺序 | 确定 | 不确定 | 可控 | **确定** ✅ |

#### Rust `tokio::join!` 的优势

- **零额外线程**：3 个 Future 在同一个线程上交替执行
- **零额外内存**：没有栈分配，状态机就是几个枚举 + 局部变量
- **编译期类型安全**：返回值类型是 `(String, String, String)`，编译器保证
- **结果顺序确定**：不像 Go channel 需要自己排序

---

### 2.3 `tokio::spawn` — 独立任务

```rust
let handle1 = tokio::spawn(async {
    sleep(Duration::from_millis(200)).await;
    42
});
let result1 = handle1.await.unwrap();
```

#### 四种语言的"启动并发任务"对比

```
┌─────────────────────────────────────────────────────────────────┐
│                    启动一个并发任务                                │
├──────────┬──────────────────────────────────────────────────────┤
│ C++      │ std::thread t([]{ return 42; });                    │
│          │ t.join();                                            │
│          │ // 问题：OS线程，重量级，无法直接获取返回值             │
├──────────┼──────────────────────────────────────────────────────┤
│ Go       │ ch := make(chan int, 1)                              │
│          │ go func() { ch <- 42 }()                             │
│          │ val := <-ch                                          │
│          │ // 轻量级，但需要 channel 传递返回值                   │
├──────────┼──────────────────────────────────────────────────────┤
│ Erlang   │ Pid = spawn(fun() -> 42 end),                       │
│          │ %% 问题：无法直接获取返回值，需要消息传递               │
├──────────┼──────────────────────────────────────────────────────┤
│ Rust     │ let h = tokio::spawn(async { 42 });                 │
│          │ let val = h.await.unwrap();  // 直接拿到返回值 ✅     │
│          │ // 轻量级任务，可以有返回值，类型安全                   │
└──────────┴──────────────────────────────────────────────────────┘
```

> **Go vs Rust spawn 的核心区别**：Go 的 `go func()` 启动的 goroutine 没有返回值，
> 必须通过 channel 传递结果；Rust 的 `tokio::spawn` 返回 `JoinHandle<T>`，
> 直接 `.await` 就能拿到类型安全的返回值。

#### `spawn` vs 直接 `.await` vs `join!`

| 方式 | 调度 | 适用场景 |
|------|------|---------|
| `fetch_data(1).await` | 在当前任务内执行，顺序 | 需要结果才能继续时 |
| `tokio::join!(a, b, c)` | 在当前任务内并发轮询 | 几个已知的并发操作 |
| `tokio::spawn(async {...})` | 提交给调度器，可能在**不同线程**执行 | 独立的后台任务、不确定数量的并发 |

**关键区别**：`join!` 不会创建新任务，只是在当前任务内交替 poll 多个 Future；`spawn` 会创建一个真正的独立任务，可以被调度到其他线程。

---

### 2.4 `tokio::select!` — 竞速，取最快

```rust
let winner = tokio::select! {
    val = slow_operation() => { format!("Slow won: {}", val) }
    val = fast_operation() => { format!("Fast won: {}", val) }
};
// fast_operation 先完成 → slow_operation 被自动取消（drop）
```

#### 这在 C++、Go 和 Erlang 中怎么做？

```cpp
// C++: 没有原生支持！需要手动实现
// 方案1: 用 std::future + 轮询（丑陋）
// 方案2: 用 boost::asio 的 awaitable_operators（复杂）
// 方案3: 自己写一个 select... 祝你好运

// 而且 C++ 无法自动取消"输掉"的操作
// 你需要手动设置 cancellation_token，手动检查...
```

```go
// Go: 用 select + channel 实现（Go 的 select 是语言内置的！）
func selectDemo() string {
    ch1 := make(chan string, 1)
    ch2 := make(chan string, 1)

    go func() {
        time.Sleep(500 * time.Millisecond)
        ch1 <- "slow result"
    }()
    go func() {
        time.Sleep(100 * time.Millisecond)
        ch2 <- "fast result"
    }()

    // Go 的 select 语法和 Rust 的 tokio::select! 很像
    select {
    case val := <-ch1:
        return "Slow won: " + val
    case val := <-ch2:
        return "Fast won: " + val
    }
    // 问题：输掉的 goroutine 还在跑！不会自动取消
    // 需要用 context.WithCancel 手动取消
}
```

```erlang
%% Erlang: 用 receive + after 可以部分实现
%% 但没有"自动取消输家"的机制
receive
    {fast, Val} -> {fast_won, Val};
    {slow, Val} -> {slow_won, Val}
after 1000 ->
    timeout
end.
%% 问题：慢的进程还在跑，不会自动停止
%% 需要手动 exit(Pid, kill)
```

#### Go select vs Rust select! 的关键区别

| | Go `select` | Rust `tokio::select!` |
|---|---|---|
| 作用对象 | channel 操作 | 任意 Future |
| 自动取消输家？ | **否** ❌（goroutine 泄漏！） | **是** ✅（Future 被 drop） |
| 取消机制 | 需要 `context.WithCancel` 手动传播 | 所有权系统自动处理 |
| 资源泄漏风险 | 高（忘记取消 = goroutine 泄漏） | **零**（编译器保证） |

#### Rust `select!` 的杀手特性：**自动取消**

```
select! 执行流程：

  slow_operation ──────────────────────→ (500ms)
  fast_operation ──────→ 完成! (100ms)
                        ↓
                  fast 赢了！
                  slow 的 Future 被 drop
                  → 自动释放所有资源
                  → 不需要手动取消
                  → 不会泄漏
```

这是 Rust 所有权系统的威力：Future 被 drop 时，其持有的所有资源（文件句柄、网络连接、内存）都会自动释放。C++ 和 Erlang 都做不到这种"零成本自动取消"。

---

### 2.5 async block — 匿名 Future

```rust
let future = async {
    sleep(Duration::from_millis(50)).await;
    "result from async block"
};
// future 此时还没执行！
let result = future.await;  // 现在才执行
```

#### 对比

| | C++ | Go | Erlang | Rust |
|---|---|---|---|---|
| 类比 | lambda `[](){ ... }` | 闭包 `func() { ... }` | `fun() -> ... end` | `async { ... }` |
| 惰性？ | lambda 是惰性的 ✅ | 闭包是惰性的 ✅ | fun 是惰性的 ✅ | async block 是惰性的 ✅ |
| 区别 | lambda 调用时**同步阻塞** | 闭包调用时**同步阻塞**（除非 `go` 启动） | fun 调用时**同步阻塞** | `.await` 时**异步非阻塞** |

> **Go 没有 async block 的等价物**：Go 的闭包 `func() { ... }` 本身是同步的，
> 要异步执行必须用 `go func() { ... }()` 启动一个 goroutine。
> 而 Rust 的 `async { ... }` 创建的是一个 Future 对象，可以存储、传递、组合，
> 直到你决定 `.await` 它时才执行——这种「惰性组合」能力是 Go 不具备的。

**Rust async block 的独特之处**：它捕获的变量遵循所有权规则。如果你 `move` 了一个变量进 async block，编译器保证外面不能再用它——这在 C++ lambda 中是程序员的"君子协定"，在 Rust 中是**编译器强制**的。

---

## 三、用到的 Rust / Tokio API 一览

| API | 类型 | 作用 | 来源 |
|-----|------|------|------|
| `async fn` | 语言关键字 | 声明异步函数，返回 `impl Future<Output=T>` | Rust 语言 |
| `.await` | 语言关键字 | 挂起当前任务，等待 Future 完成 | Rust 语言 |
| `async { ... }` | 语言关键字 | 创建匿名 Future（async block） | Rust 语言 |
| `#[tokio::main]` | 属性宏 | 将 `async fn main()` 包装为同步入口，启动 tokio runtime | tokio |
| `tokio::time::sleep()` | 异步函数 | 异步等待指定时间（不阻塞线程） | tokio::time |
| `tokio::time::Duration` | 结构体 | 时间段表示（实际是 `std::time::Duration` 的 re-export） | tokio::time |
| `tokio::join!()` | 宏 | 并发执行多个 Future，等待**全部**完成 | tokio |
| `tokio::select!()` | 宏 | 并发执行多个 Future，等待**第一个**完成，取消其余 | tokio |
| `tokio::spawn()` | 函数 | 将 Future 作为独立任务提交给调度器 | tokio::task |
| `std::time::Instant` | 结构体 | 高精度计时器，用于测量耗时 | std |

---

## 四、友好总结：一句话理解每个概念

| 概念 | 一句话 | 生活类比 |
|------|--------|---------|
| `async fn` | "我是一个**菜谱**，不是一道菜" | 写下菜谱 ≠ 做菜 |
| `.await` | "**开始做这道菜**，做好了叫我" | 把菜放进烤箱，去做别的事 |
| `join!` | "三道菜**同时做**，全好了再上桌" | 同时烤面包、煮汤、切沙拉 |
| `select!` | "两道菜**赛跑**，谁先好吃谁" | 外卖和自己做饭，谁先好吃谁 |
| `spawn` | "**雇个帮厨**去做这道菜" | 你继续做你的，帮厨独立工作 |
| `async block` | "**临时写个菜谱**，还没开始做" | 便签纸上写个快手菜步骤 |

### 最终对比：四种语言的哲学

```
C++:    "给你所有的绳子，你自己决定怎么用（或者上吊）"
        → 线程 + 手动同步 + 未定义行为是你的问题

Go:     "简单就是力量，goroutine 就是答案"
        → 有栈协程 + channel，写起来最简单
        → 代价：GC 停顿、goroutine 泄漏、缺乏泛型表达力
        → 哲学："少即是多"，用约定代替编译器检查

Erlang: "一切都是进程，一切都是消息"
        → 简单优雅，但没有类型安全，运行时才发现错误

Rust:   "编译器是你最严格的老师，但毕业后你永远不会犯错"
        → async/await + 所有权 = 编译期保证无数据竞争
        → 零成本抽象：async 编译后和手写状态机一样快
```

### Go vs Rust 异步模型深度对比

这是最常被拿来比较的两个，因为它们都是现代系统语言，都追求高并发：

```
┌─────────────────────────────────────────────────────────────────┐
│                  Go goroutine (有栈协程)                         │
│                                                                 │
│  ✅ 优势：                                                      │
│    • 写起来最简单——不需要 async/await，普通函数就行              │
│    • 没有「函数染色」问题——sync 和 async 代码长一样              │
│    • 调试友好——goroutine 有完整调用栈                            │
│    • 学习曲线平缓——go func() 就完事了                            │
│                                                                 │
│  ❌ 劣势：                                                      │
│    • 每个 goroutine 至少 2KB 栈（百万级 = 2GB+ 内存）            │
│    • goroutine 泄漏是常见 bug（忘记取消 = 永远在跑）             │
│    • GC 停顿（虽然很短，但对延迟敏感场景有影响）                 │
│    • 没有编译期数据竞争检查（靠 race detector 运行时检测）       │
│    • channel 类型安全弱（常用 interface{} 传递）                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                  Rust async/await (无栈协程)                     │
│                                                                 │
│  ✅ 优势：                                                      │
│    • 内存开销最小——无栈状态机，通常几十~几百字节                 │
│    • 零成本抽象——编译后和手写状态机一样快                        │
│    • 编译期保证无数据竞争（Send/Sync trait）                     │
│    • 自动取消——Future drop 时自动释放所有资源                    │
│    • 无 GC——确定性析构，延迟可预测                               │
│                                                                 │
│  ❌ 劣势：                                                      │
│    • 函数染色——async 传染性，sync/async 边界需要处理             │
│    • 学习曲线陡峭——Pin, lifetime, Send bound...                  │
│    • 调试困难——调用栈是扁平的状态机                              │
│    • 生态分裂——tokio vs async-std vs smol                        │
│    • 递归 async 需要 Box::pin()                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### 同一个任务，Go vs Rust 代码对比

```go
// Go: 启动 3 个并发任务，收集结果
func main() {
    ch := make(chan string, 3)
    for i := 1; i <= 3; i++ {
        go func(id int) {
            time.Sleep(time.Duration(100*id) * time.Millisecond)
            ch <- fmt.Sprintf("Data %d", id)
        }(i)
    }
    for i := 0; i < 3; i++ {
        fmt.Println(<-ch)  // 顺序不确定
    }
}
// 6 行核心代码，简洁直观
// 但：结果顺序不确定，类型靠 channel 约束
```

```rust
// Rust: 同样的任务
#[tokio::main]
async fn main() {
    let (a, b, c) = tokio::join!(
        async { sleep(Duration::from_millis(100)).await; "Data 1" },
        async { sleep(Duration::from_millis(200)).await; "Data 2" },
        async { sleep(Duration::from_millis(300)).await; "Data 3" },
    );
    println!("{}, {}, {}", a, b, c);  // 顺序确定！
}
// 同样简洁，但：结果顺序确定，类型编译期检查
// 零额外内存分配，零 goroutine 泄漏风险
```

#### 什么时候选 Go，什么时候选 Rust？

| 场景 | 推荐 | 原因 |
|------|------|------|
| Web API / 微服务 | Go ✅ | 开发速度快，goroutine 够用 |
| 高并发网络代理 | Rust ✅ | 内存精确控制，无 GC 停顿 |
| 快速原型 | Go ✅ | 语法简单，编译快 |
| 嵌入式 / IoT | Rust ✅ | 无 GC，无运行时，可裸机运行 |
| 百万级长连接 | Rust ✅ | 每连接几十字节 vs Go 的 2KB+ |
| 团队大、水平参差 | Go ✅ | 语言简单，不容易写出"天书" |
| 延迟敏感（交易系统） | Rust ✅ | 确定性延迟，无 GC 停顿 |
| CLI 工具 | 都行 | Go 编译快，Rust 二进制更小 |

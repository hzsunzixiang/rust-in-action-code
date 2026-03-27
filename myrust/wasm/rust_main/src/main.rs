// =============================================================================
// Rust WebAssembly Demo (with main function)
//
// Demonstrates:
//   1. Basic arithmetic (add, multiply)
//   2. Recursive algorithm (fibonacci)
//   3. Array operations (sort, sum)
//   4. String processing (reverse)
//   5. Prime number computation
//
// Compile target: wasm32-wasip1 (WASI — WebAssembly System Interface)
// Runtime: wasmtime (or any WASI-compatible runtime)
//
// Unlike wasm-bindgen (lib crate), this is a standalone binary with main().
// It uses println! which maps to WASI's fd_write — no browser needed.
// =============================================================================

// ── 1. Basic arithmetic ──

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

// ── 2. Fibonacci (iterative, to match C++ version's behavior) ──

fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let (mut a, mut b): (u64, u64) = (0, 1);
            for _ in 2..=n {
                let tmp = a + b;
                a = b;
                b = tmp;
            }
            b
        }
    }
}

// ── 3. Array operations ──

fn array_sum(arr: &[i64]) -> i64 {
    arr.iter().sum()
}

fn sort_numbers(numbers: &mut [f64]) {
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}

// ── 4. String processing ──

fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

// ── 5. Prime numbers ──

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

fn count_primes(max_n: u64) -> usize {
    (2..=max_n).filter(|&x| is_prime(x)).count()
}

fn primes_up_to(max_n: u64) -> Vec<u64> {
    (2..=max_n).filter(|&x| is_prime(x)).collect()
}

// ── Main entry point ──

fn main() {
    println!("=== Rust WASM (WASI) with main() ===");
    println!("Compiled to wasm32-wasip1, running via wasmtime");
    println!();

    // 1. Basic arithmetic
    println!("── 1. Basic Arithmetic ──");
    println!("  add(3, 4)          = {}", add(3, 4));
    println!("  add(-10, 25)       = {}", add(-10, 25));
    println!("  multiply(3.14, 2)  = {:.4}", multiply(3.14, 2.0));
    println!("  multiply(0.1, 0.2) = {:.4}", multiply(0.1, 0.2));
    println!();

    // 2. Fibonacci
    println!("── 2. Fibonacci Sequence ──");
    for n in [0, 1, 5, 10, 20, 30] {
        println!("  fibonacci({:>2}) = {}", n, fibonacci(n));
    }
    println!();

    // 3. Array operations
    println!("── 3. Array Operations ──");
    let arr: Vec<i64> = vec![10, 20, 30, 40, 50];
    println!("  array       = {:?}", arr);
    println!("  array_sum   = {}", array_sum(&arr));

    let mut floats = vec![3.14, 1.41, 2.72, 0.58, 1.73];
    println!("  unsorted    = {:?}", floats);
    sort_numbers(&mut floats);
    println!("  sorted      = {:?}", floats);
    println!();

    // 4. String processing
    println!("── 4. String Processing ──");
    let test_strings = ["hello", "WebAssembly", "Rust WASI", "racecar"];
    for s in test_strings {
        println!("  reverse(\"{}\") = \"{}\"", s, reverse_string(s));
    }
    println!();

    // 5. Prime numbers
    println!("── 5. Prime Numbers ──");
    let test_primes = [2, 7, 13, 97, 100, 1];
    for &n in &test_primes {
        println!("  is_prime({:>3}) = {}", n, is_prime(n));
    }
    println!("  count_primes(100)  = {}", count_primes(100));
    println!("  count_primes(1000) = {}", count_primes(1000));

    let small_primes = primes_up_to(50);
    println!("  primes up to 50    = {:?}", small_primes);
    println!();

    // 6. Size introspection (Rust-specific: show type sizes)
    println!("── 6. Type Sizes (Rust introspection) ──");
    println!("  size_of::<i32>()   = {} bytes", std::mem::size_of::<i32>());
    println!("  size_of::<i64>()   = {} bytes", std::mem::size_of::<i64>());
    println!("  size_of::<f64>()   = {} bytes", std::mem::size_of::<f64>());
    println!("  size_of::<bool>()  = {} bytes", std::mem::size_of::<bool>());
    println!("  size_of::<&str>()  = {} bytes (fat pointer: ptr + len)", std::mem::size_of::<&str>());
    println!("  size_of::<String>()= {} bytes (ptr + len + cap)", std::mem::size_of::<String>());
    println!("  size_of::<Vec<i64>>() = {} bytes", std::mem::size_of::<Vec<i64>>());
    println!();

    println!("=== Done! 🎉 ===");
}

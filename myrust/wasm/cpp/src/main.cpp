// =============================================================================
// C++ WebAssembly Demo
//
// Demonstrates:
//   1. Exporting C++ functions to JavaScript via EMSCRIPTEN_KEEPALIVE
//   2. Arithmetic operations (add, multiply)
//   3. Recursive algorithm (fibonacci)
//   4. Memory sharing between C++ and JS (array sum)
//   5. String processing (reverse string in-place)
//   6. Struct/class usage compiled to WASM
//
// Compile with: emcc (Emscripten)
// =============================================================================

#include <emscripten.h>
#include <cstdio>
#include <cstring>
#include <cstdlib>
#include <cmath>

// All exported functions must be extern "C" to avoid C++ name mangling
extern "C" {

// --- Basic arithmetic ---

EMSCRIPTEN_KEEPALIVE
int add(int a, int b) {
    return a + b;
}

EMSCRIPTEN_KEEPALIVE
double multiply(double a, double b) {
    return a * b;
}

// --- Recursive fibonacci ---

EMSCRIPTEN_KEEPALIVE
int fibonacci(int n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// --- Array operations (demonstrates shared memory) ---

EMSCRIPTEN_KEEPALIVE
int* create_array(int size) {
    return (int*)malloc(size * sizeof(int));
}

EMSCRIPTEN_KEEPALIVE
void free_array(int* ptr) {
    free(ptr);
}

EMSCRIPTEN_KEEPALIVE
long long array_sum(int* arr, int len) {
    long long sum = 0;
    for (int i = 0; i < len; i++) {
        sum += arr[i];
    }
    return sum;
}

// --- String processing (in-place reverse) ---

EMSCRIPTEN_KEEPALIVE
char* reverse_string(char* str) {
    int len = strlen(str);
    for (int i = 0; i < len / 2; i++) {
        char tmp = str[i];
        str[i] = str[len - 1 - i];
        str[len - 1 - i] = tmp;
    }
    return str;
}

// --- Math: check if prime ---

EMSCRIPTEN_KEEPALIVE
int is_prime(int n) {
    if (n < 2) return 0;
    if (n < 4) return 1;
    if (n % 2 == 0 || n % 3 == 0) return 0;
    for (int i = 5; i * i <= n; i += 6) {
        if (n % i == 0 || n % (i + 2) == 0) return 0;
    }
    return 1;
}

// --- Performance benchmark: count primes up to N ---

EMSCRIPTEN_KEEPALIVE
int count_primes(int max_n) {
    int count = 0;
    for (int i = 2; i <= max_n; i++) {
        if (is_prime(i)) count++;
    }
    return count;
}

} // extern "C"

// --- main: runs when WASM module is loaded (optional) ---

int main() {
    printf("[C++ WASM] Module loaded successfully!\n");
    printf("[C++ WASM] add(3, 4) = %d\n", add(3, 4));
    printf("[C++ WASM] fibonacci(10) = %d\n", fibonacci(10));
    printf("[C++ WASM] is_prime(97) = %d\n", is_prime(97));
    printf("[C++ WASM] count_primes(100) = %d\n", count_primes(100));
    return 0;
}

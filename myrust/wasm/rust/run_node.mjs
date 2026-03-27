// run_node.mjs - Run Rust WASM directly in Node.js (no browser needed)
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

// Load the WASM module and JS glue code
const __dirname = dirname(fileURLToPath(import.meta.url));
const wasmPath = join(__dirname, 'pkg', 'rust_wasm_demo_bg.wasm');
const wasmBytes = readFileSync(wasmPath);

// Import the glue code (initSync allows synchronous initialization without fetch)
import { initSync, greet, fibonacci, sort_numbers } from './pkg/rust_wasm_demo.js';

// Initialize WASM synchronously with the raw bytes
initSync({ module: new WebAssembly.Module(wasmBytes) });

// =============================================================
// Now call Rust functions directly from Node.js!
// =============================================================

console.log('=== Rust WASM running in Node.js ===\n');

// Example 1: greet
const greeting = greet('Node.js');
console.log(`greet: ${greeting}`);
console.log();

// Example 2: fibonacci
for (const n of [0, 1, 5, 10, 20, 50]) {
    const result = fibonacci(n);
    console.log(`fibonacci(${n}) = ${result}`);
}
console.log();

// Example 3: sort_numbers
const input = '42, 7, 13, 99, 1, 56, 23';
const sorted = sort_numbers(input);
console.log(`sort_numbers("${input}")`);
console.log(`  => ${sorted}`);
console.log();

console.log('=== Done! ===');

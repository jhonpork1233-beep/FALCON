// Rust Benchmark Suite - compile with: rustc -O benchmarks.rs -o bench_rust
use std::time::Instant;

fn fib(n: i64) -> i64 {
    if n <= 1 { return n; }
    fib(n - 1) + fib(n - 2)
}

fn sum_loop(n: i64) -> i64 {
    let mut sum: i64 = 0;
    for i in 0..n {
        sum += i;
    }
    sum
}

fn nested_loops(n: i64) -> i64 {
    let mut sum: i64 = 0;
    for _ in 0..n {
        for _ in 0..n {
            sum += 1;
        }
    }
    sum
}

fn main() {
    println!("=== Rust Benchmark Suite ===");
    
    // 1. Fibonacci
    let start = Instant::now();
    let fib_result = fib(35);
    let duration = start.elapsed();
    println!("Fib(35): {} ({:.2} ms)", fib_result, duration.as_secs_f64() * 1000.0);
    
    // 2. Sum loop
    let start = Instant::now();
    let sum_result = sum_loop(10000000);
    let duration = start.elapsed();
    println!("Sum 10M: {} ({:.2} ms)", sum_result, duration.as_secs_f64() * 1000.0);
    
    // 3. Nested loops
    let start = Instant::now();
    let nested_result = nested_loops(1000);
    let duration = start.elapsed();
    println!("Nested 1Kx1K: {} ({:.2} ms)", nested_result, duration.as_secs_f64() * 1000.0);
    
    println!("=== Done ===");
}

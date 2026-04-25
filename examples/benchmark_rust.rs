// Rust benchmark - sum 1 to 10 million
use std::time::Instant;

fn main() {
    let start = Instant::now();
    
    let mut sum: i64 = 0;
    let mut i: i64 = 1;
    while i <= 10000000 {
        sum = sum + i;
        i = i + 1;
    }
    
    let elapsed = start.elapsed();
    
    println!("Sum: {}", sum);
    println!("Time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
}

// Rust benchmark - sum 1 to 10 million
fn main() {
    let mut sum: i64 = 0;
    let mut i: i64 = 1;
    while i <= 10000000 {
        sum = sum + i;
        i = i + 1;
    }
    println!("Sum: {}", sum);
}

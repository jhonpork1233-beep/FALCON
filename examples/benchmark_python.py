# benchmark_python.py — Python Benchmark: Sum 1 to 10,000,000
# Same algorithm as benchmark.c for fair comparison

import time

start = time.perf_counter()

total = 0
i = 1
while i <= 10_000_000:
    total += i
    i += 1

elapsed = time.perf_counter() - start
print(f"Python done: {elapsed:.3f}s")

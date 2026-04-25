# Python Benchmark Suite
import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def sum_loop(n):
    total = 0
    for i in range(n):
        total += i
    return total

def nested_loops(n):
    total = 0
    for i in range(n):
        for j in range(n):
            total += 1
    return total

print("=== Python Benchmark Suite ===")

# 1. Fibonacci
start = time.perf_counter()
fib_result = fib(35)
end = time.perf_counter()
print(f"Fib(35): {fib_result} ({(end - start) * 1000:.2f} ms)")

# 2. Sum loop
start = time.perf_counter()
sum_result = sum_loop(10000000)
end = time.perf_counter()
print(f"Sum 10M: {sum_result} ({(end - start) * 1000:.2f} ms)")

# 3. Nested loops
start = time.perf_counter()
nested_result = nested_loops(1000)
end = time.perf_counter()
print(f"Nested 1Kx1K: {nested_result} ({(end - start) * 1000:.2f} ms)")

print("=== Done ===")

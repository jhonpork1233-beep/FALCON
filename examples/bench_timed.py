# Python benchmark with internal timer - sum to 10 million
import time

start = time.perf_counter()

total = 0
for i in range(10000000):
    total = total + i

end = time.perf_counter()

print(f"Sum: {total}")
print(f"Time: {(end - start) * 1000:.2f} ms")

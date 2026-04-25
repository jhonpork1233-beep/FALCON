# Python benchmark - sum 1 to 10 million
import time

start = time.time()

total = 0
i = 1
while i <= 10000000:
    total = total + i
    i = i + 1

end = time.time()

print(f"Sum: {total}")
print(f"Time: {end - start:.3f} seconds")

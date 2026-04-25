// Same benchmark in C
#include <stdio.h>
#include <time.h>

int main() {
    clock_t start = clock();
    
    long long sum = 0;
    for (long long i = 1; i <= 10000000; i++) {
        sum += i;
    }
    
    clock_t end = clock();
    double time_ms = (double)(end - start) / CLOCKS_PER_SEC * 1000.0;
    
    printf("Sum: %lld\n", sum);
    printf("Time: %.2f ms\n", time_ms);
    return 0;
}

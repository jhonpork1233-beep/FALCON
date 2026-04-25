// C benchmark - sum 1 to 10 million
#include <stdio.h>
#include <time.h>
#include <stdint.h>

int main() {
    clock_t start = clock();
    
    int64_t sum = 0;
    int64_t i = 1;
    while (i <= 10000000) {
        sum = sum + i;
        i = i + 1;
    }
    
    clock_t end = clock();
    double time_ms = ((double)(end - start)) / CLOCKS_PER_SEC * 1000;
    
    printf("Sum: %lld\n", sum);
    printf("Time: %.2f ms\n", time_ms);
    return 0;
}

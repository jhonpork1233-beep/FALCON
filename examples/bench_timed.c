// C benchmark with internal timer - sum to 10 million
#include <stdio.h>
#include <stdint.h>
#include <time.h>

int main() {
    int64_t sum = 0;
    int64_t i;
    
    clock_t start = clock();
    
    for (i = 0; i < 10000000; i++) {
        sum = sum + i;
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC * 1000;
    
    printf("Sum: %lld\n", sum);
    printf("Time: %.2f ms\n", time_spent);
    return 0;
}

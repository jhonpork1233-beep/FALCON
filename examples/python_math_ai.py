#!/usr/bin/env python3
"""
Python Math + AI Benchmark
Same workload as falcon_math_ai.fc for fair comparison
"""

import subprocess
import time

def main():
    print("╔═══════════════════════════════════════════════╗")
    print("║  🐍 PYTHON Math + AI Benchmark               ║")
    print("╚═══════════════════════════════════════════════╝")
    print()
    
    total_start = time.perf_counter()
    
    # === MATH SECTION ===
    print("📊 Running math computation (Sum 1 to 10M)...")
    math_start = time.perf_counter()
    
    # Sum 1 to 10 million - same as Falcon
    total = 0
    for i in range(1, 10000001):
        total += i
    
    math_time = time.perf_counter() - math_start
    print(f"  ✓ Sum: {total}")
    print(f"\n⏱️  Math time: {math_time:.3f}s")
    
    # === AI SECTION ===
    print("\n🤖 Sending results to LLM...")
    ai_start = time.perf_counter()
    
    result = subprocess.run(
        ["ollama", "run", "llama3.1:8b", 
         "I calculated the sum of 1 to 10000000 and got 50000005000000. Is this correct? Answer in one sentence."],
        capture_output=True, text=True
    )
    
    ai_time = time.perf_counter() - ai_start
    print(f"[Ollama] Response:\n{result.stdout.strip()}")
    print(f"\n⏱️  AI time: {ai_time:.3f}s")
    
    total_time = time.perf_counter() - total_start
    
    print("\n" + "="*50)
    print(f"📈 PYTHON TOTAL: {total_time:.3f}s")
    print(f"   - Math: {math_time:.3f}s ({math_time/total_time*100:.1f}%)")
    print(f"   - AI:   {ai_time:.3f}s ({ai_time/total_time*100:.1f}%)")

if __name__ == "__main__":
    main()

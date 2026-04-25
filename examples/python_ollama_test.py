#!/usr/bin/env python3
"""
Python Ollama Chatbot - For benchmarking against Falcon
Same functionality as adwaith_ai.fc
"""

import subprocess
import time

def ollama_chat(model: str, personality: str, user_input: str) -> str:
    """Call Ollama and return response"""
    full_prompt = f"{personality} User: {user_input}"
    
    cmd = ["ollama", "run", model, full_prompt]
    
    start = time.perf_counter()
    result = subprocess.run(cmd, capture_output=True, text=True)
    elapsed = time.perf_counter() - start
    
    return result.stdout, elapsed

def main():
    print("╔═══════════════════════════════════════════════╗")
    print("║   🐍 PYTHON AI - Ollama Benchmark Test 🐍     ║")
    print("║   Powered by LLaMA 3.1:8b (subprocess)        ║")
    print("╚═══════════════════════════════════════════════╝")
    print()
    
    model = "llama3.1:8b"
    personality = "You are a helpful AI assistant. Answer the users question directly and concisely. If they ask for code, write the code. If they ask a question, answer it. Do not introduce yourself unless asked. Be helpful and practical."
    
    # Test prompt
    test_prompt = "write a simple C hello world program"
    
    print(f"Test prompt: {test_prompt}")
    print()
    
    # Measure total time including subprocess overhead
    total_start = time.perf_counter()
    response, llm_time = ollama_chat(model, personality, test_prompt)
    total_time = time.perf_counter() - total_start
    
    print("🤖 Response:")
    print(response)
    print()
    print(f"⏱️  LLM response time: {llm_time:.3f}s")
    print(f"⏱️  Total time: {total_time:.3f}s")

if __name__ == "__main__":
    main()

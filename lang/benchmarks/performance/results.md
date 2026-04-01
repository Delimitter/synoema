# Synoema Performance Benchmarks

## Synoema JIT (Cranelift) vs CPython 3.12

| Benchmark | Python | Synoema JIT | Speedup |
|-----------|--------|-----------|---------|
| fib(30) | 277ms | 47ms | **5.9x** |
| gcd (100K iter) | 143ms | 83ms | **1.7x** |
| collatz (10K) | 505ms | 90ms | **5.6x** |
| **Average** | | | **4.4x** |

Includes JIT compilation time. Pure execution is even faster.

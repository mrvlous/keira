<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `fuzz_abi.elf` System Call Fuzz Tester

Automated Ring 3 fuzzing and chaos test suite powered by the Syzkaller-Lite mutation engine, verifying kernel robustness across 10,000+ mutated vectors and stress conditions.

---

## 1. Invocation

```bash
run /system/bin/fuzz_abi.elf
```

---

## 2. Test Phases

1. **Phase 1: Randomized Syscall Boundary Injections**: Executes 10,000 mutated parameter vectors (illegal pointers, unmapped kernel ranges, non-existent syscall vectors).
2. **Phase 2: Descriptor Table Exhaustion**: Saturates file descriptor capacities to capacity limits and validates leak-free descriptor reclamation.
3. **Phase 3: Memory Boundary & Churn**: Tests extreme heap allocations, boundary overflows, and demand paging limits.
4. **Phase 4: Rapid Process Churn**: Rapidly spawns and reaps concurrent tasks to verify task table clean-up and process isolation.
5. **Phase 5: High-Frequency Signal Storm**: Delivers bursts of asynchronous signals to verify frame trap integrity and intact `sigreturn` unwinding.

---

## 3. Stability Criteria

The kernel must safely return standard POSIX error codes (`EFAULT`, `EBADF`, `EINVAL`, `ENOMEM`, `EPERM`) and remain rock solid with zero kernel panics or page table corruptions.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `fuzz_abi.elf` System Call Fuzz Tester

Fuzzing utility designed to test kernel robustness against malformed pointers, invalid buffers, and extreme arguments.

---

## Fuzzing Strategies

* **Pointer Fuzzing**: Passes `NULL`, unmapped kernel memory (`0xC0000000+`), out-of-bounds addresses, and cross-boundary pointers.
* **Buffer Length Fuzzing**: Negative lengths, `SIZE_MAX`, integer overflow combinations.
* **File Descriptor Fuzzing**: Negative FDs, `INT_MAX`, closed FDs, out-of-range indices.
* **Expected Outcome**: The kernel must return appropriate error codes (`EFAULT`, `EBADF`, `EINVAL`) and never trigger a kernel panic or leak privileged memory.

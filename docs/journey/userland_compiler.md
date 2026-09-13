<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Development Journey: Native C Compiler & Ring 3 Userland

This document chronicles the creation of the native C compiler (`kcc`), standard C library (SDK), and standalone ELF toolchain inside Keira Kernel.

---

## Self-Hosting Toolchain Vision

```mermaid
graph LR
    Dev["Developer on Keira Terminal"] --> Edit["Edit Source: edit /users/admin/app.c"]
    Edit --> Compile["Compile: kcc -o /apps/bin/app.elf /users/admin/app.c"]
    Compile --> Run["Execute: run /apps/bin/app.elf"]
    Run --> Output["Ring 3 Userland Process Running on Keira Kernel"]
```

---

## Key Engineering Milestones

* **Native C Compiler (`kcc`)**: Built a complete recursive-descent C compiler capable of parsing C syntax, performing type checking, and emitting native ELF executables.
* **C Standard Library (SDK)**: Implemented standard libc primitives (`stdio`, `stdlib`, `string`, `ctype`, `math`, `time`, `signal`) backed by native kernel system calls.
* **System V ABI & Auxiliary Vector (`auxv[]`)**: Standard initial user stack framing with `argc`, `argv[]`, `envp[]`, and initialization metadata (`AT_PAGESZ`, `AT_ENTRY`, `AT_RANDOM` canary entropy).
* **Copy-on-Write (COW) Memory Sharing**: Zero-copy physical frame sharing across child address spaces during `sys_fork()` with on-demand `#PF` resolution.
* **POSIX Signals & Interactive TTY Line Discipline**: Asynchronous signal trampolines (`sigaction`, `sigreturn`) coupled with live keyboard interrupt dispatching (`Ctrl+C` -> `SIGINT`, `Ctrl+Z` -> `SIGSTOP`).
* **High-Resolution Clock Subsystem**: High-precision `clock_gettime` and `nanosleep` implementation with sub-millisecond accuracy.
* **Freestanding C Runtime Startup (`crt0`) & Ring 3 ABI Harness**: Standardized application entry point unpacking user stack arguments to `main(argc, argv)` across `x86_64` and `i686`, validated by a dedicated Ring 3 ABI security and fault-injection verification harness (`test_abi.elf`).
* **Buffered Standard I/O & Dual-Tier Memory Allocator**: Implemented 1024-byte block/line/unbuffered stream I/O (`FILE*`, `_IOFBF`, `_IOLBF`, `_IONBF`) and a hybrid heap (`sbrk` with boundary-tag coalescing) and mmap tier allocator for zero-fragmentation userland execution.
* **Environment Variable Management (`environ`)**: Integrated stack-passed `envp` initialization into `crt0` startup, backed by heap-allocated dynamic environment modification (`getenv`, `setenv`, `unsetenv`, `putenv`).
* **Stream Formatting & Token Scanning (`fprintf`, `sscanf`)**: Advanced formatted stream output and string parsing engine supporting format specifications (`%d`, `%i`, `%u`, `%x`, `%s`, `%c`).
* **Ring 3 Multiprocess Orchestration**: Preemptive process cloning via `fork()`, exit code propagation to `TaskState::Zombie`, and blocking parent reaping with POSIX `waitpid()` and exit status decoding (`WIFEXITED`, `WEXITSTATUS`).
* **Anonymous Inter-Process Pipe Streaming**: Bidirectional and unidirectional FIFO memory-backed pipes (`pipe()`, `read()`, `write()`) connecting parent and child userland processes.
* **Process Memory Mutation Isolation**: Dedicated physical frame duplication and deep page-table cloning during `fork()` guaranteeing strict task isolation and preventing memory corruption on child reap.
* **Cross-Architecture Kernel Boundary Guard**: Strict user pointer sanitization across canonical user space ranges (`[0x10000, 0x0000_7FFF_FFFF_FFFF]` on `x86_64`, `[0x10000, 0xBFFF_FFFF]` on `i686`) rejecting high-half kernel pointers with `-EFAULT`.
* **Native In-Kernel Self-Hosting Compilation Loop**: Seamless compilation of C source files into freestanding ELF binaries using native `kcc.elf` and immediate execution under the Keira shell.
* **Zero-Dependency Self-Contained Ecosystem**: Complete ability to develop, compile, test, and run native applications directly on bare metal without host dependencies.

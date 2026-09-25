<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `test_abi.elf` Kernel ABI Test Suite

Comprehensive Ring 3 test suite verifying that kernel system calls behave strictly according to POSIX specifications and that CPU hardware traps are securely contained.

---

## 1. Invocation

```bash
run /system/bin/test_abi.elf
```

---

## 2. Test Verification Modules

1. **Process Lifecycle & Credentials**: Validates `fork`, `execve`, `waitpid`, `getpid`, credential isolation (`UID`/`GID`), and PID 0 orphan reparenting.
2. **Memory Boundary & VMM**: Validates `sbrk` heap expansion, dual-tier allocation (`mmap`/`munmap`), Copy-on-Write (COW) page isolation, and file-backed persistence.
3. **Virtual Filesystem & Descriptors**: Validates `open`, `read`, `write`, `close`, `lseek`, buffered stream I/O, `/system/dev/tty`, file locking, and `dup`/`dup2` descriptor targeting.
4. **IPC & Network Engines**: Validates anonymous inter-process pipes, BSD stream sockets, and asynchronous event notifications.
5. **Hardware Fault Containment**: Ensures `#UD` (Illegal Instruction) converts to `SIGILL`, `#DE` (Divide by Zero) converts to `SIGFPE`, and `#PF` (Page Fault) converts to `SIGSEGV` without kernel panic.
6. **Stress & Boundary Injections**: Validates cross-architecture high-pointer rejections (`EFAULT`), stack canary protections (`__stack_chk_guard`), and out-of-bounds syscall handling.

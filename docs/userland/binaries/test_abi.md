<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `test_abi.elf` Kernel ABI Test Suite

Comprehensive test suite verifying that kernel system calls behave according to POSIX specifications.

---

## Test Categories

1. **Process Management**: `fork`, `execve`, `waitpid`, `getpid`.
2. **File System Operations**: `open`, `read`, `write`, `close`, `lseek`, `unlink`, `stat`.
3. **Memory Management**: `mmap`, `munmap`, `brk` boundary expansion.
4. **Signal Delivery**: Signal handlers, masking, signal delivery under interruption.
5. **IPC Mechanisms**: Anonymous pipes, shared memory regions, futex locking.

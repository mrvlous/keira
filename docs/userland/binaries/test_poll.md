<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `test_poll.elf` Freestanding Core Utility

The `test_poll.elf` executable (`/bin/test_poll.elf` and `/bin/test_poll`) is a freestanding Ring 3 test utility validating the POSIX `poll` synchronous I/O multiplexing subsystem and Century-Plus Syscall 106 (`SYS_POLL`).

---

## 1. Architectural Purpose

In accordance with UNIX purity and the Keira v0.8.0 milestone roadmap, synchronous multiplexing verification is implemented as a freestanding unprivileged Ring 3 binary:

1. **Ring 3 Privilege Isolation**: Executes entirely in unprivileged Ring 3, invoking Syscall 106 (`SYS_POLL`) through the libc wrapper `poll()`.
2. **Readiness State Machine Verification**: Tests edge-triggered and level-triggered transitions across `POLLIN`, `POLLOUT`, `POLLHUP` and `POLLNVAL`.
3. **Multi-Descriptor Scalability**: Validates simultaneous polling across multiple pipe endpoints and character device descriptors.
4. **Non-Blocking & Timeout Semantics**: Validates zero-timeout non-blocking poll cycles and immediate readiness reporting.

---

## 2. Test Battery Matrix

| Test Suite | Target Subsystem | Assertions |
| :--- | :--- | :--- |
| **Test 1** | Pipe Readiness Transitions | Verifies `POLLOUT` on write end and lack of `POLLIN` on empty read end, followed by active `POLLIN` upon transmission |
| **Test 2** | Multi-Descriptor Multiplexing | Tests concurrent polling across multiple distinct pipe channels simultaneously |
| **Test 3** | Zero-Timeout Non-Blocking | Asserts immediate return without sleeping when timeout is set to 0 milliseconds |
| **Test 4** | Closed Pipe Handoff (`POLLHUP`) | Verifies that closing a pipe write end correctly triggers `POLLHUP` and `POLLIN` on reader |
| **Test 5** | Invalid Descriptor Safety (`POLLNVAL`) | Asserts that out-of-bounds or closed descriptors correctly report `POLLNVAL` without kernel faults |

---

## 3. Usage Examples

### Executing Full Poll Test Battery

```bash
keira:/# test_poll.elf
[TEST 1] Poll readiness transitions on anonymous pipe...
  [PASS] Empty pipe correctly reported POLLOUT and not POLLIN
  [PASS] Pipe reader correctly reported POLLIN after write
  [PASS] Drain read succeeded
[TEST 2] Multi-descriptor multiplexing across pipe endpoints...
  [PASS] Multiplexed poll correctly detected only pipe B ready
[TEST 3] Zero and immediate timeout non-blocking polling...
  [PASS] Zero-timeout poll on empty pipe returned 0 immediately
[TEST 4] POLLHUP detection on closed pipe writer...
  [PASS] Closed writer pipe correctly reported POLLHUP and POLLIN
[TEST 5] Invalid file descriptor handling (POLLNVAL)...
  [PASS] Invalid descriptor correctly reported POLLNVAL
[SUMMARY] All 5 poll() tests PASSED!
```

### Executing Via Canonical Symlink

```bash
keira:/# test_poll
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/test_poll/main.c`
- **Binary Locations**: `/bin/test_poll.elf`, `/bin/test_poll`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<poll.h>`, `<unistd.h>`, `<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<errno.h>`)
- **System Calls**: `sys_poll` (Vector 106), `sys_pipe`, `sys_read`, `sys_write`, `sys_close` and `sys_exit`

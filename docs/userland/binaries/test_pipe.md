<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `test_pipe.elf` Anonymous Pipe Subsystem Test Suite

The `test_pipe.elf` executable (`/bin/test_pipe.elf` and `/bin/test_pipe`) is a freestanding Ring 3 test suite validating in-kernel anonymous pipes, multi-channel concurrency and POSIX stream semantics.

---

## 1. Architectural Purpose

Pipe verification exercises the dynamic IPC subsystem (`crates/ipc`) from userland across five phases:

1. **Basic Pipe Streaming**: Creates an anonymous pipe pair via `pipe()`, streams data bytes and confirms payload integrity.
2. **Multi-Channel Isolation**: Allocates multiple concurrent pipe channels simultaneously and verifies that buffers do not experience crosstalk or corruption.
3. **POSIX EOF Semantics**: Closes the write endpoint and verifies that subsequent reads on an empty buffer return `0` (EOF).
4. **Non-Blocking Mode (`pipe2`)**: Allocates a pipe with `O_NONBLOCK` and confirms that reading an empty active channel returns `-1` with `errno == EAGAIN`.
5. **Broken Pipe Detection**: Closes the read endpoint, attempts writing to the write endpoint and verifies `-1` return with `errno == EPIPE` and `SIGPIPE` delivery.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Execute pipe verification suite:
keira:/# /bin/test_pipe.elf
========================================
Keira Kernel Pipe Subsystem Test Suite
========================================

[TEST 1] Basic pipe creation and byte streaming...
  [OK] Pipe allocated with descriptors (rd=3, wr=4)
  [OK] Single pipe read and write verified
[TEST 2] Multi-channel concurrent isolation...
  [OK] Independent multi-channel isolation verified
[TEST 3] POSIX EOF condition on write endpoint closure...
  [OK] POSIX EOF condition validated
[TEST 4] pipe2 with O_NONBLOCK flag...
  [OK] O_NONBLOCK behavior validated
[TEST 5] Broken pipe detection and EPIPE...
  [OK] EPIPE error and broken pipe handling validated

----------------------------------------
Test Suite Result: [PASSED] (5/5 tests successful)
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/test_pipe/main.c`
- **Binary Locations**: `/bin/test_pipe.elf`, `/bin/test_pipe`
- **Privilege Level**: Ring 3 unprivileged
- **System Calls Invoked**: `SYS_PIPE` (23), `SYS_PIPE2` (96), `SYS_READ` (7), `SYS_WRITE` (8), `SYS_CLOSE` (9)

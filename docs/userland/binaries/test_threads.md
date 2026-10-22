<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `test_threads.elf` Freestanding Multithreading Test Suite

The `test_threads.elf` executable (`/bin/test_threads.elf` and `/bin/test_threads`) is a freestanding Ring 3 test suite validating multithreading and thread synchronization primitives.

---

## 1. Architectural Purpose

Thread verification is exercised in Ring 3 against the work-stealing APIC kernel scheduler:

1. **Thread Spawning**: Tests `clone` syscall with `CLONE_VM` and `CLONE_THREAD` flags.
2. **Mutex Synchronization**: Verifies userland mutex locking, atomic fast paths and futex sleep contention.
3. **Thread Join & Exit**: Ensures thread stacks and descriptors are reclaimed cleanly upon termination.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Execute multithreading verification suite:
keira:/# /bin/test_threads.elf
[TEST] Initializing multithreading verification...
[TEST] Spawning concurrent worker threads...
[PASS] Mutex contention and counter synchronization passed
[PASS] Thread joining and stack reclamation verified
[DONE] All multithreading test suites passed successfully!
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/test_threads/main.c`
- **Binary Locations**: `/bin/test_threads.elf`, `/bin/test_threads`
- **Privilege Level**: Ring 3 unprivileged

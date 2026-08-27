<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Freestanding C SDK Header & Runtime Library (`libc`)

This submodule specifies the freestanding C standard library headers and runtime functions provided in Keira Kernel under `user/include/` and `user/lib/`.

---

## SDK Header Catalog

| Header Module | Document | Description |
| :--- | :--- | :--- |
| **Standard I/O** | [`stdio.md`](stdio.md) | Formatted console output, file stream operations (`fopen`, `fread`, `fwrite`) |
| **Standard Utility** | [`stdlib.md`](stdlib.md) | Dynamic heap memory (`malloc`/`free`), string conversions, process exit |
| **String Manipulation** | [`string.md`](string.md) | Memory block copying, string search, comparison, and length routines |
| **Mathematical Functions** | [`math.md`](math.md) | Integer square root, exponentiation, min/max, and boundary clamping |
| **Character Classification** | [`ctype.md`](ctype.md) | ASCII digit/letter validation and uppercase/lowercase mapping |
| **System Call Wrappers** | [`syscalls.md`](syscalls.md) | Low-level Ring 3 inline assembly system call dispatchers (`syscall0`..`syscall6`) |

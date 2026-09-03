<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ring 3 Userland Process Memory Map

This document specifies virtual address space segmentation, stack allocation, user heap boundaries, and isolation barriers in Keira Kernel.

---

## Virtual Address Space Layout

```mermaid
graph TD
    subgraph Userland Virtual Memory (0x00000000 - 0x7FFFFFFFFFFF)
        Text["0x0040_0000: Executable Text (.text) [R-X]"]
        Data["0x0060_0000: Initialized Data (.data / .rodata) [RW-]"]
        BSS["0x0080_0000: Uninitialized Data (.bss) [RW-]"]
        Heap["0x0100_0000 - 0x1000_0000: User Dynamic Heap (brk / mmap) [RW-]"]
        Stack["0x7FFF_FFFF_0000: User Thread Stack (Growth Downwards) [RW-]"]
    end
    subgraph Kernel Space Barrier (0xFFFF800000000000 - 0xFFFFFFFFFFFFFFFF)
        KernelHigh["Kernel Code, Direct Physical Map & Stacks [Supervisor]"]
    end
```

---

## Technical Specifications

| Segment | Virtual Start Range | Permissions | Purpose |
| :--- | :--- | :--- | :--- |
| **`.text`** | `0x0040_0000` | Read / Execute (`R-X`) | ELF instructions |
| **`.data` / `.bss`** | `0x0060_0000` | Read / Write (`RW-`) | Global variables |
| **User Heap** | `0x0100_0000` | Read / Write (`RW-`) | `malloc()` dynamic heap allocations |
| **User Stack** | `0x7FFFFFD8_0000` – `0x7FFFFFE0_0000` | Read / Write (`RW-`) | Function call frames, local variables, and System V CLI arguments |

---

## System V Initial Stack Framing & CLI Arguments

At process startup, the kernel formats the top of the user stack with the command-line argument vector before transferring execution privilege to Ring 3 via `jump_to_user`:

```
+-------------------------------------------------------------+ High Address
| ASCII Strings (argv[0], ..., envp[0], ..., AT_RANDOM 16B)   |
+-------------------------------------------------------------+
| Auxiliary Vector (Elf64_auxv_t / Elf32_auxv_t)             |
|   - AT_NULL (0, 0)                                          |
|   - AT_RANDOM (25, pointer to 16-byte random canary seed)   |
|   - AT_CLKTCK (17, 100)                                     |
|   - AT_EUID (12, 0)                                         |
|   - AT_UID (11, 0)                                          |
|   - AT_FLAGS (8, 0)                                         |
|   - AT_BASE (7, 0)                                          |
|   - AT_ENTRY (9, entry_point)                               |
|   - AT_PAGESZ (6, 4096)                                     |
+-------------------------------------------------------------+
| NULL Pointer (envp terminator)                              |
| NULL Pointer (argv terminator)                              |
| char *argv[argc-1]                                          |
| ...                                                         |
| char *argv[0]                                               |
| uint64_t argc                                               | <-- Initial %rsp (16-byte aligned)
+-------------------------------------------------------------+ Low Address
```

- **x86_64 Calling Convention**: `_start(int argc, char **argv)` receives `RDI = argc` and `RSI = argv`.
- **i686 Calling Convention**: `_start(int argc, char **argv)` receives arguments on stack with `[ESP+0] = ret_dummy`, `[ESP+4] = argc`, `[ESP+8] = argv`, `[ESP+12] = envp`.

---

## Auxiliary Vector (`Elf64_auxv_t` / `Elf32_auxv_t`) Specification

The Auxiliary Vector conveys kernel execution parameters directly to the dynamic linker or C runtime before `main()`:

| Tag | Value | Description |
| :--- | :--- | :--- |
| `AT_PAGESZ` | `6` | System page size in bytes (`4096`) |
| `AT_ENTRY` | `9` | Entry point virtual address of the program executable |
| `AT_BASE` | `7` | Base address of the ELF interpreter (`0` for static binaries) |
| `AT_FLAGS` | `8` | Processor execution flags (`0`) |
| `AT_UID` | `11` | Real user identifier of the calling process (`0` for admin) |
| `AT_EUID` | `12` | Effective user identifier of the calling process (`0`) |
| `AT_CLKTCK` | `17` | Frequency of system timer clock ticks (`100` Hz) |
| `AT_RANDOM` | `25` | Pointer to 16 bytes of random entropy for `-fstack-protector` canaries |
| `AT_NULL` | `0` | End-of-vector sentinel marker |

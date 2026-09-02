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
| ASCII Strings (argv[0], argv[1], ..., envp[0], ...)         |
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

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel System Call Infrastructure

The `syscall` subsystem implements the boundary between unprivileged Userland (Ring 3) and privileged Kernel (Ring 0).

---

## Architecture & Transition Boundary

```mermaid
graph LR
    User["Ring 3 Userland<br/>(kcc.elf, shell)"] -->|"syscall (64-bit) / int 0x80 (32-bit)"| Entry["Entry Trampoline<br/>(MSR_LSTAR / IDT 0x80)"]
    Entry --> TSS["TSS Stack Switch<br/>(Load Ring 0 RSP0)"]
    TSS --> Dispatch["dispatcher.md<br/>Syscall Routing Engine"]
    Dispatch --> Valid["user_copy.md<br/>Pointer Bounds Validation"]
    Valid --> Table["table.md<br/>62 Kernel Handlers"]
```

---

## Syscall Module Index

| Document | Component | Description |
| :--- | :--- | :--- |
| [`table.md`](table.md) | System Call Vector Table | Numerical vectors, argument types, and return values for system calls |
| [`dispatcher.md`](dispatcher.md) | Dispatcher & ABI | Syscall routing, register argument unpacking, and POSIX errno mapping |
| [`user_copy.md`](user_copy.md) | Validated Pointer Copying | Hardened memory transfer between Ring 0 and Ring 3 with boundary checks |

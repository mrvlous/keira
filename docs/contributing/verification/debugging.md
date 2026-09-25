<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Debugging & Diagnostics Guide

This document details techniques for debugging Keira Kernel using GDB, QEMU serial traces, stack unwinding, and hardware breakpoint inspection.

---

## 1. Remote GDB Debugging via QEMU

QEMU includes a built-in GDB stub allowing full remote kernel debugging:

```bash
# 1. Launch kernel in QEMU with GDB stub enabled (-s) and CPU halted at start (-S)
qemu-system-x86_64 -s -S -cdrom build/x86/x86_64/iso/keira-x86_64.iso -serial stdio

# 2. In a separate terminal, launch GDB
gdb build/x86/x86_64/bin/keira.bin
```

Inside GDB:
```gdb
# Connect to QEMU GDB server on localhost:1234
(gdb) target remote :1234

# Set breakpoint at kernel main entry
(gdb) break kernel_main
(gdb) continue

# Inspect register values
(gdb) info registers

# Print backtrace
(gdb) backtrace

# Step instruction by instruction
(gdb) stepi
(gdb) nexti
```

---

## 2. Serial Console Logging

Keira configures UART 16550 COM1 (`0x3F8`) as early diagnostic output:
* Launch with `make run` or pass `-serial stdio` to view all kernel printk logs directly in host terminal.
* High-volume tracing: Pass `-serial file:serial.log` to capture boot logs to disk.

---

## 3. Kernel Panic Analysis & Registers

When a fatal condition occurs, `keira_kernel::runtime::panic` captures CPU state:
```text
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
               KERNEL PANIC
  Reason: Page fault at unmapped address 0xDEADBEEF
!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
CPU: 0  |  CR2: 0xDEADBEEF  |  CR3: 0x00201000
RAX: 0x00000000  RBX: 0x00100000  RCX: 0x00000001
RIP: 0x0010542A  RSP: 0xFFFF80000001FE90
```
Use `addr2line` on host to locate the exact source file and line:
```bash
addr2line -e build/x86/x86_64/bin/keira.bin 0x0010542A
```

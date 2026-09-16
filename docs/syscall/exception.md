<!-- SPDX-License-Identifier: GPL-2.0-only -->

# CPU Exception Dispatcher & Userland Fault Containment

This document specifies the fault containment architecture, hardware exception to POSIX signal translation matrix, and persistent core dump generation mechanism in Keira Kernel.

---

## Fault Containment Invariant

No userland (Ring 3) program fault—including invalid opcodes, integer division by zero, null pointer dereferences, unmapped virtual memory accesses, or general protection violations—shall ever panic the kernel or halt the machine.

When a CPU exception fires while executing in user mode (`(cs & 3) == 3`), the kernel's centralized exception dispatcher (`crates/syscall/src/exception/mod.rs`) catches the trap and takes one of three containment paths:

1. **Demand Paging Resolution**: For unmapped memory faults (`#PF`, Vector 14), if the faulting virtual address (CR2) resides within a registered Virtual Memory Area (VMA) or the task's dynamic heap (`program_break_start..program_break`), the kernel transparently allocates a physical frame, maps the page with user-accessible permissions, invalidates the TLB (`invlpg`), and resumes user execution without raising an exception.
2. **Userland Signal Dispatch**: If the process registered a custom signal handler via `sys_sigaction` (vector 64) for the corresponding POSIX signal, the kernel preserves the interrupted register context in `task.saved_sigcontext`, configures the user stack frame, and redirects execution to the user signal handler. Upon completion, the handler executes `sys_sigreturn` (vector 65) to restore the saved execution context.
3. **Controlled Termination & Core Dump**: If the signal has no custom handler (default action), the kernel logs crash telemetry to the serial console and VGA display, formats an architectural core dump containing registers and stack frame details, writes the diagnostic artifact to `/data/log/core_<pid>.dmp` via the Virtual Filesystem (VFS), and terminates the process with `exit_current(-(sig as i32))`.

---

## Hardware Exception to POSIX Signal Matrix

| Vector | Exception Name | Mnemonic | POSIX Signal | Signal Name | Core Dump Generated |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `0` | Divide-by-Zero Error | `#DE` | `8` | `SIGFPE` | Yes |
| `4` | Overflow Trap | `#OF` | `8` | `SIGFPE` | Yes |
| `5` | Bound Range Exceeded | `#BR` | `11` | `SIGSEGV` | Yes |
| `6` | Invalid Opcode | `#UD` | `4` | `SIGILL` | Yes |
| `7` | Device Not Available | `#NM` | `8` | `SIGFPE` | Yes |
| `11` | Segment Not Present | `#NP` | `7` | `SIGBUS` | Yes |
| `12` | Stack-Segment Fault | `#SS` | `7` | `SIGBUS` | Yes |
| `13` | General Protection Fault | `#GP` | `11` | `SIGSEGV` | Yes |
| `14` | Unresolved Page Fault | `#PF` | `11` | `SIGSEGV` | Yes |
| `16` | x87 Floating-Point Exception | `#MF` | `8` | `SIGFPE` | Yes |
| `17` | Alignment Check | `#AC` | `7` | `SIGBUS` | Yes |
| `19` | SIMD Floating-Point Exception | `#XM` | `8` | `SIGFPE` | Yes |

---

## Core Dump Format (`/data/log/core_<pid>.dmp`)

When an unhandled fatal exception terminates a userland task, a structured ASCII diagnostic dump is persisted to `/data/log/core_<pid>.dmp` on the primary FAT16 storage partition:

```text
=== KEIRA CORE DUMP ===
PID: 4
Name: test_abi
Signal: 11 (SIGSEGV)
Vector: 14 (Page Fault (#PF))
Error Code: 0x6
RIP: 0x4012A4
RSP: 0x7FFFFFE01FC0
RBP: 0x7FFFFFE01FE0
RFLAGS: 0x202
CR2: 0x1234
Status: TERMINATED BY SIGNAL
```

---

## Parent Notification & Status Decoding

When a parent process reaps a terminated child via `waitpid(pid, &status, options)`:
- `WIFSIGNALED(status)` evaluates to true (`((status) & 0x7f) > 0`).
- `WTERMSIG(status)` evaluates to the exact POSIX signal number (`((status) & 0x7f)`).

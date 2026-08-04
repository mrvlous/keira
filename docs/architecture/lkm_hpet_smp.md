# Loadable Kernel Modules (LKM), HPET Timer & SMP Subsystems

This document details the Loadable Kernel Module (LKM) dynamic loader architecture, High-Precision Event Timer (HPET) hardware interface, Kernel Callstack Unwinder, and Symmetric Multiprocessing (SMP) Inter-Processor Interrupt (IPI) engine in Keira Kernel.

---

## 1. Loadable Kernel Module (LKM) Subsystem

Keira Kernel provides dynamic loadable module support for kernel-space drivers and subsystem extensions:

*   **Dynamic Symbol Table (`kallsyms`)**: The kernel maintains an in-memory symbol table ([module.rs](../../kernel/src/entry/module.rs)) mapping kernel function symbols (`vga_print_str`, `heap_alloc`, `scheduler_yield`) to physical/virtual memory entry addresses.
*   **Module Relocation & Binding**: Dynamically loaded ELF relocatable kernel modules (`.ko`) resolve external symbol references against `kallsyms` during module initialization.

### System Call Interface

```c
// Syscall 34: Initialize and load dynamic kernel module
long sys_init_module(const void *img_ptr, unsigned long len);

// Syscall 35: Unload and release dynamic kernel module resources
long sys_delete_module(const char *name_ptr);
```

---

## 2. High-Precision Event Timer (HPET) Subsystem

Implemented in [hpet.rs](../../kernel/src/arch/hpet.rs):

*   **Nanosecond Counter Resolution**: Hardware register space at `0xFED00000` provides high-precision timing sources calibrated for microsecond and nanosecond measurement.
*   **System Call Interface**: `sys_clock_gettime` (Syscall 36) returns high-resolution nanosecond hardware timestamps for real-time process benchmarking and precision sleeping.

---

## 3. Kernel Callstack Unwinder & Debugging (`sys_ptrace`)

Implemented in [unwind.rs](../../kernel/src/arch/unwind.rs):

*   **Stack Frame Pointer Walking**: In the event of a Kernel Panic or Page Fault exception, `unwind_stack()` parses chained `RBP`/`RSP` stack frames to output the precise function callstack sequence (`kernel_main` -> `isr_handler` -> `fault_handler`).
*   **Process Tracing (`sys_ptrace`)**: Syscall 37 provides process tracing and register inspection primitives for userland debuggers and diagnostic tools.

---

## 4. SMP Inter-Processor Interrupts (IPI) & TLB Shootdown

Implemented in [smp.rs](../../kernel/src/arch/smp.rs):

*   **Local APIC IPI Messaging**: Multi-core CPU cores coordinate execution using Inter-Processor Interrupts (IPI) dispatched via the Local APIC Interrupt Command Register (ICR).
*   **Cross-Core TLB Shootdown**: Invalidates stale virtual memory page translation entries (`invlpg`) across all active CPU cores during memory remap operations to enforce cache coherency.

<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Architecture & Core Kernel Subsystems

Welcome to the Core Architecture documentation section for Keira Kernel.

## Documents

* [Bootstrapping & Trampolining](bootstrapping.md): Multi-stage boot sequence from GRUB to Rust 64-bit Long Mode.
* [Memory Management](memory.md): Physical Memory Manager (PMM), Virtual Memory Manager (VMM), `sys_mmap`/`sys_munmap`, and early C heap.
* [Task Scheduler](scheduler.md): Preemptive priority multitasking model, scheduler queue, and task states.
* [System Calls & Interrupts](syscalls.md): System call dispatcher, Local APIC controller, dynamic TSS RSP0 stack switching.
* [Symmetric Multiprocessing (SMP)](smp.md): Multi-core CPU initialization and LAPIC IPI shootdown.
* [Loadable Kernel Modules (LKM)](lkm.md): Dynamic module loading and kallsyms symbol resolution (`sys_init_module`).
* [High Precision Event Timer (HPET)](hpet.md): Nanosecond timer resolution and ACPI HPET mapping (`sys_clock_gettime`).
* [High-Resolution POSIX Interval Timers](timer.md): POSIX nanosecond interval timers (`sys_timer_create`/`sys_timer_settime`).
* [PCIe ECAM & MSI/MSI-X Interrupts](pcie.md): PCIe configuration space and Message Signaled Interrupts.
* [DMA Scatter-Gather Allocator](dma.md): Contiguous physical DMA buffer allocation and Scatter-Gather list mapping.
* [ACPI Power Management & NMI Watchdog](power.md): ACPI power state transitions (S0/S3/S5) and hardware NMI watchdog.
* [Hardware Performance PMU Counters](perf.md): CPU hardware event monitoring unit counters (`sys_perf_event_open`).
* [Kernel Event Logging & Syslog](klog.md): Circular `dmesg` kernel log ring buffer and diagnostic system call (`sys_syslog`).
* [Kernel Callstack Unwinder Engine](unwind.md): RBP/RSP pointer frame walking for kernel panic debugging backtraces.
* [Resource Control Groups (cgroups)](cgroups.md): Process memory accounting & PID namespace isolation.

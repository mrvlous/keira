<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Journey Archive: Milestones 1 to 15 (v0.1.0 to v0.6.0)

## Overview

This document archives the foundational engineering milestones of the Keira Kernel, spanning versions `v0.1.0` through `v0.6.0`. It chronicles how the kernel was constructed strictly from first principles as a **pure freestanding kernel from scratch** in `#![no_std]` Rust, without relying on host operating systems, foreign runtimes, or distribution packages.

---

## Table of Archived Milestones

| Milestone | Target Version | Focus Architecture |
| :--- | :--- | :--- |
| [Milestone 1: Bare-Metal Multiboot2 Handshake & Core CPU Descriptors](#milestone-1-bare-metal-multiboot2-handshake--core-cpu-descriptors) | `v0.1.0` | Multiboot2, GDT, TSS, IDT, PIC/APIC and SMP AP Trampoline |
| [Milestone 2: Physical Frame Allocation, Paging & Heap Memory](#milestone-2-physical-frame-allocation-paging--heap-memory) | `v0.1.0` | Bitmap Allocator, 4-Level Paging, Segregated Free-List Heap and Swap |
| [Milestone 3: Preemptive Scheduler & Process Management](#milestone-3-preemptive-scheduler--process-management) | `v0.2.0` | APIC Timer Scheduling, PCB Context Switching, Cgroups and POSIX Signals |
| [Milestone 4: Storage Subsystem, FAT16/32 & Ext4 Extents](#milestone-4-storage-subsystem-fat1632--ext4-extents) | `v0.2.0` | AHCI SATA, NVMe, Sector Cache, FAT16/32 and Ext4 Extents |
| [Milestone 5: Native Networking Engine & Socket Abstractions](#milestone-5-native-networking-engine--socket-abstractions) | `v0.3.0` | Intel e1000 DMA Rings, ARP, IPv4, TCP State Machine and TLS 1.3 Engine |
| [Milestone 6: Ring 3 Privilege Separation & Native C Compiler (KCC)](#milestone-6-ring-3-privilege-separation--native-c-compiler-kcc) | `v0.3.0` | Userland Isolation, Syscall Gateway, Freestanding libc.a and KCC Compiler |
| [Milestone 7: TPM 2.0 Enclave, eBPF Engine & Asynchronous io_uring](#milestone-7-tpm-20-enclave-ebpf-engine--asynchronous-io_uring) | `v0.4.0` | TPM 2.0 PCR Integrity, In-Kernel eBPF Virtual Machine and io_uring Rings |
| [Milestone 8: Modular Network Fetch Engine & Netfilter Firewall](#milestone-8-modular-network-fetch-engine--netfilter-firewall) | `v0.4.0` | Chunked HTTP Streaming, Live Progress Bars and Netfilter Packet Filtering |
| [Milestone 9: Symmetrical Multi-Architecture Parity & Syzkaller Fuzzing](#milestone-9-symmetrical-multi-architecture-parity--syzkaller-fuzzing) | `v0.5.0` | Dual-Architecture x86_64/i686 Parity, 81 Syscall Vectors and Fuzz Defense |
| [Milestone 10: Epoch-Based Reclamation (EBR) & Copy-on-Write (COW)](#milestone-10-epoch-based-reclamation-ebr--copy-on-write-cow) | `v0.6.0` | Lock-Free Memory Reclamation, Frame Refcounting and COW Page Fault Engine |
| [Milestone 11: Per-CPU Lock-Free Work-Stealing Scheduler](#milestone-11-per-cpu-lock-free-work-stealing-scheduler) | `v0.6.0` | Chase-Lev Deques, Decentralized Load Balancing and EBR CPU Pinning |
| [Milestone 12: Hierarchical Per-CPU Magazine Slab Allocator](#milestone-12-hierarchical-per-cpu-magazine-slab-allocator) | `v0.6.0` | Jeff Bonwick Magazine Caching, Depot Spilling and Zero-Lock Fast Paths |
| [Milestone 13: Local APIC IPI Framework & SMP TLB Shootdown](#milestone-13-local-apic-ipi-framework--smp-tlb-shootdown) | `v0.6.0` | Inter-Processor Interrupts (Vector 0xFD), Rendezvous Barriers and Cross-Core Shootdown |
| [Milestone 14: SMP Reschedule IPI & Vector 0xFE Dispatch](#milestone-14-smp-reschedule-ipi--vector-0xfe-dispatch) | `v0.6.0` | Cross-Core Reschedule IPI, Preemption Stubs and Latency Reduction |
| [Milestone 15: Userland Multi-Threading & POSIX Pthreads Engine](#milestone-15-userland-multi-threading--posix-pthreads-engine) | `v0.6.0` | Clone Syscall (CLONE_VM, CLONE_THREAD), Thread Group Sharing and POSIX Pthreads |

---

## Milestone 1: Bare-Metal Multiboot2 Handshake & Core CPU Descriptors

- **Handshake Protocol**: Implemented 32-bit Multiboot2 bootstrap assembly routines parsing memory tags, framebuffer descriptors and initrd modules passed by GRUB.
- **CPU Descriptor Management**: Configured Global Descriptor Table (GDT), Task State Segment (TSS) with dedicated double-fault interrupt stacks (IST), and Interrupt Descriptor Table (IDT) covering all 32 x86 CPU exception vectors.
- **Interrupt Routing**: Implemented dual-mode PIC 8259 remapping and Local APIC / IO-APIC routing with spurious vector protection.
- **Multi-Core Trampoline**: Designed a 16-bit real-mode Application Processor (AP) assembly trampoline executed via APIC INIT-SIPI-SIPI sequences to transition secondary cores into 64-bit Long Mode.

---

## Milestone 2: Physical Frame Allocation, Paging & Heap Memory

- **Physical Frame Allocator**: Constructed a bitmap physical memory manager tracking 4 KiB frames across all usable E820 physical RAM regions.
- **4-Level Paging Engine**: Implemented PML4, PDPT, PD and PT identity-mapped virtual memory management with NX bit protection, write-protect flags and user/supervisor attribute enforcement.
- **Dynamic Kernel Heap**: Built a segregated free-list heap allocator supporting power-of-two size classes, coalescing adjacent free blocks and zeroing uninitialized memory.
- **Auxiliary Swap**: Introduced an in-kernel swap storage subsystem allowing anonymous dirty pages to be backed by designated disk blocks.

---

## Milestone 3: Preemptive Scheduler & Process Management

- **APIC Timer Preemption**: Programmed the Local APIC timer for periodic scheduling ticks at 100 Hz.
- **Process Control Blocks (PCB)**: Created task descriptors encapsulating saved general-purpose registers, CR3 page table roots, execution states (`Ready`, `Running`, `Blocked`, `Zombie`), virtual address maps and file descriptor tables.
- **Control Groups (cgroups)**: Implemented hierarchical resource limitation slices tracking CPU shares, maximum tasks and memory usage quotas.
- **POSIX Signal Delivery**: Built asynchronous signal dispatching (`SIGINT`, `SIGKILL`, `SIGSEGV`, `SIGALRM`, `SIGCHLD`) with userland signal stack frames and sigreturn handling.

---

## Milestone 4: Storage Subsystem, FAT16/32 & Ext4 Extents

- **Storage Controllers**: Implemented native PCI AHCI SATA and NVMe block device drivers using Direct Memory Access (DMA) physical command lists.
- **Virtual File System (VFS)**: Defined uniform file, directory and device node operations abstracting underlying physical drivers.
- **FAT16/FAT32 Driver**: Built complete File Allocation Table parsing including Long File Name (LFN) checksum synthesis, cluster traversal and root directory lookup.
- **Ext4 Extents Read Driver**: Added read-only support for Linux Ext4 filesystems through extent tree traversal and inode metadata extraction.
- **Sector LRU Cache**: Implemented a write-back sector cache utilizing Least Recently Used (LRU) eviction to reduce raw disk I/O operations.

---

## Milestone 5: Native Networking Engine & Socket Abstractions

- **NIC Hardware Driver**: Implemented an Intel e1000 Gigabit PCI network driver managing physical circular DMA transmit and receive descriptor rings.
- **Ethernet & ARP**: Handled frame encapsulation, MAC address filtering, ARP request generation and dynamic neighbor cache table maintenance.
- **IPv4 & ICMP**: Built an Internet Protocol version 4 packet processing pipeline with checksum validation and ICMP Echo Request / Echo Reply handlers.
- **TCP State Machine**: Implemented a Transmission Control Protocol engine supporting 3-way handshakes (`SYN`, `SYN-ACK`, `ACK`), sliding window flow control, packet retransmission and connection teardown (`FIN-ACK`).
- **TLS 1.3 Engine**: Constructed an in-kernel cryptographic handshake layer executing X25519 Elliptic Curve Diffie-Hellman and AES-GCM cipher suites for secure HTTPS traffic.

---

## Milestone 6: Ring 3 Privilege Separation & Native C Compiler (KCC)

- **Ring 3 Privilege Separation**: Established user mode transition routines loading userland code/data segment selectors (`0x23`, `0x1B`) and dropping execution from Ring 0 to Ring 3 via `iretq` / `sysexit`.
- **System Call Gateway**: Built Fast System Call dispatching using x86-64 `syscall` / `sysret` MSRs (`MSR_STAR`, `MSR_LSTAR`, `MSR_FMASK`).
- **Freestanding C Runtime (`libc.a`)**: Built a complete autonomous standard C library with `stdio`, `string`, `stdlib`, `unistd`, `sys/stat` and memory functions.
- **In-Kernel Native C Compiler (KCC)**: Integrated a self-hosting subset C compiler directly into the environment capable of compiling C source files into static ELF binaries.

---

## Milestone 7: TPM 2.0 Enclave, eBPF Engine & Asynchronous io_uring

- **TPM 2.0 Enclave**: Programmed Memory-Mapped I/O (MMIO) interfaces for Trusted Platform Module (TPM) chips to execute PCR measurements and cryptographic hash verification.
- **In-Kernel eBPF VM**: Implemented a 64-bit virtual machine interpreter executing sandboxed extended Berkeley Packet Filter (eBPF) bytecode with safety verifiers and map abstractions.
- **Asynchronous io_uring**: Designed lock-free ring buffers (Submission Queue `SQ` and Completion Queue `CQ`) mapped directly into user space for zero-syscall asynchronous block and network I/O.

---

## Milestone 8: Modular Network Fetch Engine & Netfilter Firewall

- **Modular Network Fetch Engine**: Implemented chunked HTTP/HTTPS streaming directly into file nodes with terminal transfer progress bars.
- **Netfilter Firewall**: Built an in-kernel stateful packet filter evaluating IPv4 and TCP/UDP header criteria against configurable rule chains (`INPUT`, `OUTPUT`, `FORWARD`).

---

## Milestone 9: Symmetrical Multi-Architecture Parity & Syzkaller Fuzzing

- **Dual-Architecture Parity**: Expanded all core kernel abstractions to support 32-bit Protected Mode (`i686`) symmetrically alongside 64-bit Long Mode (`x86_64`), including two-tier paging and `int 0x80` syscall dispatching.
- **81 POSIX Syscalls**: Reached 81 distinct standardized POSIX system call vectors with boundary checks and errno conversions.
- **Syzkaller-Lite Defense**: Hardened userland pointer access through validation stubs and kernel exception trap containment to prevent privilege escalation or panic triggers.

---

## Milestone 10: Epoch-Based Reclamation (EBR) & Copy-on-Write (COW)

- **Epoch-Based Reclamation (EBR)**: Introduced lock-free memory reclamation utilizing a global epoch counter and per-core thread registration to safely defer deallocations without stop-the-world synchronization.
- **Physical Frame Reference Counting**: Maintained atomic reference counters for all mapped physical memory frames.
- **Copy-on-Write (COW)**: Optimized process cloning by sharing physical memory pages marked read-only, trapping write faults in Page Fault Vector 14, and duplicating pages lazily upon write access.

---

## Milestone 11: Per-CPU Lock-Free Work-Stealing Scheduler

- **Per-CPU Chase-Lev Deques**: Implemented lock-free work-stealing circular buffers per CPU core supporting fast LIFO local push/pop and atomic FIFO remote stealing.
- **Decentralized Load Balancer**: Removed central scheduler runqueue locks by allowing idle cores to dynamically steal tasks from overloaded neighbors.
- **EBR CPU Pinning**: Integrated epoch protection into scheduler context switches to prevent reclamation of active task structures.

---

## Milestone 12: Hierarchical Per-CPU Magazine Slab Allocator

- **Magazine Layer**: Implemented Jeff Bonwick's magazine caching architecture with thread-local object arrays (`M` rounds) providing zero-contention allocation fast paths.
- **Depot Layer**: Synchronized local magazine exchanges through a shared central depot mutex only when magazines are completely full or empty.
- **Slab Fallback**: Backed object allocations by contiguous physical page slabs with freelist indexing.

---

## Milestone 13: Local APIC IPI Framework & SMP TLB Shootdown

- **Inter-Processor Interrupts (IPI)**: Engineered low-level hardware IPI delivery via the Local APIC Interrupt Command Register (ICR) targeting specific APIC IDs or broadcasts.
- **SMP TLB Shootdown Vector (0xFD)**: Implemented an asynchronous multi-core rendezvous barrier invalidating translation lookaside buffers (`invlpg`) across remote cores whenever virtual page mappings change.

---

## Milestone 14: SMP Reschedule IPI & Vector 0xFE Dispatch

- **Reschedule Vector (0xFE)**: Created a dedicated low-latency hardware interrupt vector for waking idle cores or notifying overloaded cores to yield execution.
- **Cross-Core Task Preemption**: Eliminated scheduling latency when enqueuing urgent threads onto remote CPUs by triggering immediate context switches.

---

## Milestone 15: Userland Multi-Threading & POSIX Pthreads Engine

- **Clone Syscall**: Added `SYS_CLONE` supporting `CLONE_VM` (shared virtual address space), `CLONE_FS`, `CLONE_FILES`, `CLONE_SIGHAND` and `CLONE_THREAD` (shared thread group ID).
- **Freestanding POSIX Pthreads**: Implemented `pthread_create`, `pthread_join`, `pthread_mutex_init`, `pthread_mutex_lock`, `pthread_mutex_unlock` and `pthread_exit` in userland `libc.a`.
- **Thread Local Storage (TLS)**: Configured FS segment base registers (`MSR_FS_BASE` / `arch_prctl`) for per-thread execution context.

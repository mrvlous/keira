<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Documentation Map

Welcome to the technical documentation for Keira Kernel. This modular documentation system is designed to provide developers and contributors with a clear, in-depth understanding of the kernel's design, architecture, and code layout.

## Documentation Directory Structure

The documentation is organized into 4 primary categories:

### 1. Hyper-Modular Kernel Crates (`docs/crates/`)
Explore the dedicated technical guides for each of the 12 workspace member crates:
*   [keira-core](crates/core/README.md): Foundational collections (`RingBuffer`, `LruCache`), `SpinLock`, `SpinMutex`, `klog` logging, and error taxonomy.
*   [keira-arch](crates/arch/README.md): x86_64 CPU instructions, port I/O, APIC/IDT interrupts, timers, PMU counters, ACPI power, callstack unwinding, and KVM virtualization.
*   [keira-crypto](crates/crypto/README.md): Bare-metal SHA-256, HMAC, AES-128, AES-128-GCM, Curve25519 (X25519), and TPM 2.0 security enclave.
*   [keira-mem](crates/mem/README.md): Physical memory manager (PMM), virtual 4-level paging (VMM), kernel heap, DMA buffers, and swap pager.
*   [keira-io](crates/io/README.md): VGA text console, VBE linear framebuffer (LFB), 16550 UART serial, PCI/PCIe bus, storage (IDE, AHCI, NVMe, RAM disk), USB, sound (HDA, speaker), and multi-virtual terminals (`tty1`–`tty4`).
*   [keira-fs](crates/fs/README.md): VFS traits, FAT12/16/32, EXT4/EXT2, USTAR initrd reader, 64-bit ELF loader, `/dev` device nodes, file locking, and LVM/RAID.
*   [keira-net](crates/net/README.md): Intel e1000, Realtek RTL8139, VirtIO-Net, Ethernet, ARP, IPv4, ICMP Ping, UDP, DHCP client, DNS resolver, TCP 3-way handshake, BSD sockets, native TLS 1.3, Netfilter firewall, and eBPF engine.
*   [keira-task](crates/task/README.md): Preemptive multitasking scheduler, cgroups resource controls, PID namespaces, MAC security, Seccomp BPF filter, and POSIX signal delivery.
*   [keira-ipc](crates/ipc/README.md): Anonymous pipes, zero-copy `splice`, POSIX shared memory, `io_uring`, `eventfd`/`signalfd`, `epoll`, `mqueue`, and Futex wait queues.
*   [keira-syscall](crates/syscall/README.md): Complete 62-vector system call table, dispatcher, Ring 3 TSS IST configuration, and CPU exception routing.
*   [keira-shell](crates/shell/README.md): 74 native shell commands, fullscreen `kvi` text editor, tab auto-completion, history buffer, and command executor.
*   [keira-kernel](crates/kernel/README.md): Multiboot2 entry trampoline (`kernel_main`), early hardware init orchestration, and Blue Screen of Death panic handler.

### 2. Contributor Guidelines (`docs/contributing/`)
*   [Contributor Overview](contributing/README.md): Guidelines and roadmap for contributing.
*   [Workspace Setup](contributing/setup.md): Toolchain requirements, Rust nightly, and emulator targets.
*   [Building and Running](contributing/build.md): Makefile targets for compilation, ISO creation, disk image partitioning, and QEMU configuration.
*   [Coding Style Standards](contributing/style.md): Style rules, comment standards for C/Assembly/Rust, copyright attribution, and linter configurations.
*   [Contribution & Patch Workflow](contributing/workflow.md): Developer guide for branching, local testing, formatting, and submitting pull requests.
*   [Automated Testing Suite](contributing/testing.md): Unit tests, headless QEMU smoke tests, QMP automated tests, and 20-cycle stress tests.
*   [Debugging Techniques](contributing/debugging.md): Remote GDB debugging (`:1234`), COM1 serial logs, and QEMU monitor commands.
*   [Unsafe Rust Safety Invariants](contributing/unsafe_guidelines.md): `# Safety` contracts, pointer validation, and MMIO safety rules.
*   [Architecture Review Rubric](contributing/architecture_review.md): Subsystem isolation rules, dependency layering, and zero-bloat policy.
*   [Tutorial: Adding System Calls](contributing/adding_syscalls.md): Step-by-step tutorial for implementing new system calls.
*   [Tutorial: Adding Shell Commands](contributing/adding_commands.md): Step-by-step tutorial for creating native shell commands.
*   [Tutorial: Adding Hardware Drivers](contributing/adding_drivers.md): Step-by-step tutorial for developing hardware and block drivers.

### 3. System Architecture & Blueprints (`docs/architecture/`)
*   [Memory Model & Virtual Paging](architecture/memory_model.md): 4-level paging, address space layout, identity mappings, and KASLR.
*   [Privilege Rings & Hardware Isolation](architecture/privilege_rings.md): Ring 0 vs Ring 3 privilege transitions, GDT, TSS, and syscall MSRs.
*   [Scheduling Model](architecture/scheduling_model.md): Preemptive multitasking, context switches, timer ticks, and task states.
*   [Boot Pipeline](architecture/boot_pipeline.md): Multiboot2, 32->64 Bit assembly trampoline, C hardware init, and Rust `kernel_main`.
*   [Security Architecture](architecture/security_model.md): MAC path policies, Seccomp BPF filters, NX bit, and TPM 2.0 enclave.

### 4. Userland Subsystems & C SDK (`docs/userland/`)
*   [C SDK Header Catalog](userland/c_sdk.md): Freestanding C library (`stdio.h`, `stdlib.h`, `string.h`, `syscall.h`, `socket.h`, `fcntl.h`, `time.h`).
*   [In-Kernel GCC Compiler Toolchain](userland/gcc_compiler.md): In-kernel freestanding GCC compiler binary (`/apps/bin/gcc.elf`).
*   [Ring 3 ELF Execution & Memory Isolation](userland/elf_execution.md): ELF loading, isolated PML4 tables, and process lifecycles.
*   [User Runtime Library](userland/runtime.md): Dynamic memory allocation, environment variables, and POSIX file I/O.
*   [Multi-User Accounts & System Hostname](userland/users.md): User authentication, privilege separation, and persistent hostname configuration.
*   [POSIX File Security & Permissions](userland/permissions.md): Permission bits and attribute protection flags.

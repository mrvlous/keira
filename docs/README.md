<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Keira Kernel Documentation Map

Welcome to the technical documentation for Keira Kernel. This modular documentation system is designed to provide developers and contributors with a clear, in-depth understanding of the kernel's design, architecture, and code layout.

## Documentation Directory Structure

To help you navigate the codebase, the documentation is divided into 9 modular categories:

### 1. Architecture & Core Kernel (`docs/architecture/`)
*   [Bootstrapping & Trampolining](architecture/bootstrapping.md): Multi-stage boot sequence from GRUB to Rust 64-bit Long Mode.
*   [Memory Management](architecture/memory.md): Physical Memory Manager (PMM), Virtual Memory Manager (VMM), `sys_mmap`/`sys_munmap`, and early C heap.
*   [Task Scheduler](architecture/scheduler.md): Preemptive priority multitasking model, scheduler queue, and task states.
*   [System Calls & Interrupts](architecture/syscalls.md): System call dispatcher, Local APIC controller, dynamic TSS RSP0 stack switching.
*   [Symmetric Multiprocessing (SMP)](architecture/smp.md): Multi-core CPU initialization and LAPIC IPI shootdown.
*   [Loadable Kernel Modules (LKM)](architecture/lkm.md): Dynamic module loading and kallsyms symbol resolution (`sys_init_module`).
*   [High Precision Event Timer (HPET)](architecture/hpet.md): Nanosecond timer resolution and ACPI HPET mapping (`sys_clock_gettime`).
*   [High-Resolution POSIX Interval Timers](architecture/timer.md): POSIX nanosecond interval timers (`sys_timer_create`/`sys_timer_settime`).
*   [PCIe ECAM & MSI/MSI-X Interrupts](architecture/pcie.md): PCIe configuration space and Message Signaled Interrupts.
*   [DMA Scatter-Gather Allocator](architecture/dma.md): Contiguous physical DMA buffer allocation and Scatter-Gather list mapping.
*   [ACPI Power Management & NMI Watchdog](architecture/power.md): ACPI power state transitions (S0/S3/S5) and hardware NMI watchdog.
*   [Hardware Performance PMU Counters](architecture/perf.md): CPU hardware event monitoring unit counters (`sys_perf_event_open`).
*   [Kernel Event Logging & Syslog](architecture/klog.md): Circular `dmesg` kernel log ring buffer and diagnostic system call (`sys_syslog`).
*   [Kernel Callstack Unwinder Engine](architecture/unwind.md): RBP/RSP pointer frame walking for kernel panic debugging backtraces.
*   [Resource Control Groups (cgroups)](architecture/cgroups.md): Process memory accounting & PID namespace isolation.

### 2. Hardware & Software Security (`docs/security/`)
*   [Cryptographic Subsystem](security/crypto.md): Bare-metal Rust `no_std` implementations of SHA-256/HMAC, AES-128-GCM AEAD, and Curve25519 X25519 ECDH key exchange.
*   [Hardware Security TPM 2.0 Enclave](security/tpm.md): Trusted Platform Module MMIO interface, PCR measurement banks, and hardware key storage.
*   [NX Bit & KASLR Hardware Security](security/nx.md): Hardware No-Execute (NX) page protection and KASLR randomization.
*   [Mandatory Access Control (MAC)](security/mac.md): Path-based security rule evaluation and process sandboxing policies.
*   [Seccomp BPF Syscall Filtering Sandbox](security/seccomp.md): In-kernel BPF system call sandbox filtering (`sys_seccomp`).

### 3. Inter-Process Communication & Asynchronous I/O (`docs/ipc/`)
*   [Asynchronous Kernel I/O Engine (io_uring)](ipc/iouring.md): Zero-copy ring buffer I/O (`sys_io_uring_setup`).
*   [Fast Userspace Mutex (Futex)](ipc/futex.md): Atomic userspace locking and kernel wait queue synchronization (`sys_futex`).
*   [Epoll Scalable I/O Event Engine](ipc/epoll.md): Scalable O(1) event multiplexing descriptors (`sys_epoll_create`/`sys_epoll_ctl`).
*   [EventFD & SignalFD Subsystem](ipc/eventfd.md): Counter notification descriptors (`sys_eventfd`) and POSIX signal routing (`sys_signalfd`).
*   [POSIX Message Queue IPC Subsystem](ipc/mqueue.md): In-kernel priority message queues (`sys_mq_open`).
*   [Zero-Copy Kernel Pipe Splice](ipc/splice.md): In-kernel page swapping between file descriptors (`sys_splice`/`sys_vmsplice`).
*   [POSIX Shared Memory IPC & Semaphore Subsystem](ipc/shm_sem.md): Shared physical memory pages (`shmget`/`shmat`) and counting semaphores (`sys_shm_sem`).
*   [POSIX Real-Time Signal Engine & Process Job Control](ipc/signal.md): POSIX signals (`SIGKILL`, `SIGTERM`, `SIGINT`) and terminal job control (`sys_kill`).

### 4. Virtualization & Hypervisor (`docs/virtualization/`)
*   [Hardware Virtualization Hypervisor (KVM)](virtualization/kvm.md): Intel VMX / AMD SVM guest VM execution context (`sys_kvm_create_vm`/`sys_kvm_run_vcpu`).

### 5. Networking & Packet Filtering (`docs/networking/`)
*   [Intel e1000 Network Driver & Socket API](networking/network.md): PCI enumeration, MAC address parsing, TCP state engine, DHCP client, Dynamic ARP cache, POSIX Sockets, and Native TLS 1.3 Engine (`https`).
*   [In-Kernel DNS Resolver & Cache Table](networking/dns_resolver.md): 16-slot dynamic LRU DNS cache table and UDP 53 RFC 1035 packet resolution.
*   [In-Kernel Stateful NAT Firewall Engine](networking/netfilter.md): Stateful IPv4 packet filtering, IPTables rules, and 1:N NAT masquerading (`sys_netfilter`).
*   [Zero-Copy BPF Packet Filter Engine](networking/bpf.md): In-kernel BPF bytecode interpreter for raw socket packet filtering.

### 6. Device Drivers (`docs/drivers/`)
*   [NVMe PCIe Controller Driver](drivers/nvme.md): High-speed NVMe 1.4 PCIe SSD storage driver with Admin Queues, Doorbell registers, and Namespace mapping.
*   [VGA Text Console, Code Editor & VBE Framebuffer](drivers/vga.md): Display buffer manipulation, cursor positioning, PS/2 input, interactive 128-line code editor (`edit`), and VBE Auto-Adaptive 32-bpp Linear Framebuffer Graphics (`framebuffer`).
*   [Serial UART COM1](drivers/serial.md): Low-level 16550A serial communication driver for boot debugging logs.
*   [Sound Programming](drivers/sound.md): Programming PIT Channel 2 for PC Speaker sound generation and Intel High Definition Audio (HDA) DMA controller initialization.
*   [USB Mass Storage & USB HID Device Subsystem](drivers/usb_storage.md): USB Bulk-Only Transport (BOT) framing, SCSI commands, FAT16 flash drive mounting, and USB HID parsing (`sys_usb_device`).
*   [PS/2 Mouse Driver](drivers/mouse.md): PS/2 mouse packet decoding, resolution setup, and coordinate tracking.
*   [CMOS Real-Time Clock Driver](drivers/rtc.md): CMOS Real-Time Clock register queries and UTC timestamp parsing.
*   [USB Host Controller Driver](drivers/usb.md): PCI enumeration for xHCI/EHCI/UHCI USB controllers, descriptor decoding, and bus status querying (`usb`).

### 7. Filesystems & Storage (`docs/filesystems/`)
*   [Virtual Filesystem (VFS)](filesystems/vfs.md): Core VFS traits, Keira native directory structure (`/system/dev/`), POSIX `/dev/` path aliasing, file descriptors, and abstraction layers.
*   [Hardware RAID & Logical Volume Manager (LVM)](filesystems/lvm_raid.md): Storage pooling over SATA/NVMe, Volume Groups, and RAID 0/1 arrays (`sys_raid_lvm`).
*   [Native EXT4 Filesystem Driver](filesystems/ext4.md): Native Linux EXT4 superblock parsing, inode table reading, and extent tree block mapping.
*   [FAT Filesystem](filesystems/fat.md): FAT12/16/32 directory walking, cluster allocation tables, long file name (LFN) entries, cluster read/write/append operations, sector block cache `sync`, and native file protection (`protect`, `fileinfo`).
*   [Swap Space & Virtual Memory Pager](filesystems/swap.md): Anonymous physical memory page swapping (`sys_swapon`/`sys_swapoff`).
*   [TAR Archive Reader](filesystems/tar.md): Read-only parsing of the USTAR archive format loaded as the boot initrd.

### 8. Userland Subsystems & Toolchain (`docs/userland/`)
*   [User Runtime Library (libc & Extensions)](userland/runtime.md): Dynamic memory allocation (malloc), POSIX stdio file I/O, environment variables, socket programming (`socket.h`), C Math (`math.h`) & Time (`time.h`), and system call wrappers.
*   [ELF64 Binary Loader & Userland Execution](userland/elf_loader.md): Ring 3 user mode execution, PML4 address space isolation, segment mapping, and stack frame initialization.
*   [POSIX File Descriptors & Stream I/O](userland/posix_io.md): Standard POSIX file descriptor table (0..1024), access mode flags (`fcntl.h`), and stream system call vectors.
*   [Multi-User Account Management](userland/users.md): Persistent user management (`user`), password storage (`/system/etc/passwd`), dynamic prompt, 3-attempt retry fallback, and UNIX privilege separation.
*   [System Hostname Configuration](userland/hostname.md): System hostname configuration (`hostname`) persisted to `/system/etc/hostname`.
*   [POSIX File Security & Attributes](userland/permissions.md): POSIX file security & protection flags (`protect`, `fileinfo`).
*   [Multi-Virtual Terminal Subsystem](userland/tty.md): Virtual terminal switching (`tty1`..`tty4`) and console screen buffers.
*   [System Initialization & Userland Bootstrap](userland/init.md): Kernel userland bootstrapping and interactive shell session initialization.
*   [Self-Hosting C Compiler](userland/gcc.md): Parser, lexer, AST builder, and ELF64 code emission inside the built-in C compiler (`gcc.c`).

### 9. Contribution Guidelines (`docs/contributing/`)
*   [Workspace Setup](contributing/setup.md): Installing the toolchain, cross-compiler packages, Rust nightly, and emulator targets.
*   [Building and Running](contributing/build.md): Makefile targets for compilation, ISO creation, disk image partitioning, and QEMU configuration.
*   [Coding Style Standards](contributing/style.md): Style rules, comment standards for C/Assembly/Rust, copyright attribution, and linter configurations.
*   [Contribution & Patch Workflow](contributing/workflow.md): Step-by-step developer guide for branching, local testing, formatting, and submitting pull requests.

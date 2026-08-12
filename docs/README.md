<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Keira Kernel Documentation Map

Welcome to the technical documentation for Keira Kernel. This modular documentation system is designed to provide developers and contributors with a clear, in-depth understanding of the operating system's design, architecture, and code layout.

## Documentation Directory Structure

To help you navigate the codebase, the documentation is divided into the following categories:

### 1. Architecture and Core Kernel
*   [Bootstrapping and Trampolining](architecture/bootstrapping.md): The multi-stage boot sequence from GRUB to Rust 64-bit Long Mode.
*   [Memory Management](architecture/memory.md): Design of Physical Memory Manager (PMM), Virtual Memory Manager (VMM), `sys_mmap`/`sys_munmap` page allocation, and early C bump heap allocator.
*   [Task Scheduler](architecture/scheduler.md): Preemptive priority multitasking model, scheduler queue, SMP multi-core execution (`smp_init`), and Unix signals (`sys_kill`).
*   [System Calls and Interrupts](architecture/syscalls.md): Exception handling, 76 system call vectors, Local APIC controller, dynamic TSS RSP0 stack switching, process cloning (`sys_fork`), virtual memory protection (`sys_mprotect`/`sys_madvise`), loadable kernel modules (`sys_init_module`), high-precision HPET timer (`sys_clock_gettime`), kernel unwinder (`sys_ptrace`), async I/O (`sys_io_uring_setup`), futex threading (`sys_futex`), hypervisor (`sys_kvm_create_vm`), syslog (`sys_syslog`), interval timers (`sys_timer_create`), pipe splice (`sys_splice`), PMU counters (`sys_perf_event_open`), eBPF JIT (`sys_bpf_jit`), Virtio (`sys_virtio`), SEV/TDX (`sys_sev`), io_worker (`sys_io_uring_register`), KFENCE (`sys_kfence`), and Sched_Deadline (`sys_sched_setattr`).
*   [Cryptographic Subsystem](architecture/crypto.md): Bare-metal Rust `no_std` implementations of SHA-256/HMAC, AES-128-GCM AEAD, and Curve25519 X25519 ECDH key exchange.
*   [Hardware Virtualization Hypervisor (KVM)](architecture/kvm.md): Intel VMX / AMD SVM guest VM execution context (`sys_kvm_create_vm`/`sys_kvm_run_vcpu`).
*   [Hardware Security TPM 2.0 Enclave](architecture/tpm.md): Trusted Platform Module MMIO interface, PCR measurement banks, and hardware key storage.
*   [Zero-Copy BPF Packet Filter Engine](architecture/bpf.md): In-kernel BPF bytecode interpreter for raw socket packet filtering.
*   [DMA Scatter-Gather Allocator](architecture/dma.md): Contiguous physical DMA buffer allocation and Scatter-Gather list mapping.
*   [Kernel Event Logging & Syslog](architecture/klog.md): Circular `dmesg` kernel log ring buffer and diagnostic system call (`sys_syslog`).
*   [Mandatory Access Control (MAC)](architecture/mac.md): Path-based security rule evaluation and process sandboxing policies.
*   [High-Resolution POSIX Interval Timers](architecture/timer.md): POSIX nanosecond interval timers (`sys_timer_create`/`sys_timer_settime`).
*   [Zero-Copy Kernel Pipe Splice](architecture/splice.md): In-kernel page swapping between file descriptors (`sys_splice`/`sys_vmsplice`).
*   [ACPI Power Management & NMI Watchdog](architecture/power.md): ACPI power state transitions (S0/S3/S5) and hardware NMI watchdog.
*   [Hardware Performance PMU Counters](architecture/perf.md): CPU hardware event monitoring unit counters (`sys_perf_event_open`).
*   [EventFD & SignalFD Subsystem](architecture/eventfd.md): Counter notification descriptors (`sys_eventfd`) and POSIX signal routing (`sys_signalfd`).
*   [Seccomp BPF Syscall Filtering Sandbox](architecture/seccomp.md): In-kernel BPF system call sandbox filtering (`sys_seccomp`).
*   [Swap Space & Virtual Memory Pager](architecture/swap.md): Anonymous physical memory page swapping (`sys_swapon`/`sys_swapoff`).
*   [Epoll Scalable I/O Event Engine](architecture/epoll.md): Scalable O(1) event multiplexing descriptors (`sys_epoll_create`/`sys_epoll_ctl`).
*   [KASAN Shadow Memory Diagnostic Engine](architecture/kasan.md): Shadow memory validation for heap access safety (`sys_kasan`).
*   [POSIX Message Queue IPC Subsystem](architecture/mqueue.md): In-kernel priority message queues (`sys_mq_open`).
*   [Resource Control Groups (cgroups)](architecture/cgroups.md): Process memory accounting & PID namespace isolation.
*   [Fast Userspace Mutex (Futex)](architecture/futex.md): Atomic userspace locking and kernel wait queue synchronization (`sys_futex`).
*   [Loadable Kernel Modules (LKM)](architecture/lkm.md): Dynamic module loading and kallsyms symbol resolution (`sys_init_module`).
*   [High Precision Event Timer (HPET)](architecture/hpet.md): Nanosecond timer resolution and ACPI HPET mapping (`sys_clock_gettime`).
*   [Symmetric Multiprocessing (SMP)](architecture/smp.md): Multi-core CPU initialization and LAPIC IPI shootdown.
*   [PCIe ECAM & MSI/MSI-X Interrupts](architecture/pcie.md): PCIe configuration space and Message Signaled Interrupts.
*   [Asynchronous Kernel I/O Engine (io_uring)](architecture/iouring.md): Zero-copy ring buffer I/O (`sys_io_uring_setup`).
*   [NX Bit & KASLR Hardware Security](architecture/nx.md): Hardware No-Execute (NX) page protection and KASLR randomization.
*   [eBPF JIT Compiler Engine](architecture/bpf_jit.md): Native x86_64 JIT bytecode translation (`sys_bpf_jit`).
*   [Virtio 1.0 Paravirtualized PCI Driver](architecture/virtio.md): Split/Packed Virtqueues (`sys_virtio`).
*   [AMD SEV & Intel TDX Subsystem](architecture/sev.md): Confidential computing enclaves (`sys_sev`).
*   [io_uring Worker Thread Pool Engine](architecture/io_worker.md): Async kernel polling worker threads (`sys_io_uring_register`).
*   [KFENCE Memory Guard Engine](architecture/kfence.md): Sampling heap memory guard (`sys_kfence`).
*   [POSIX Sched_Deadline EDF Scheduler](architecture/deadline.md): Hard real-time Earliest Deadline First scheduler (`sys_sched_setattr`).
*   [Hyper-V Hypercall & SynIC Engine](architecture/hyperv.md): Microsoft Hyper-V / Azure hypercalls and SynIC synthetic interrupts (`sys_hyperv`).
*   [io_uring Async Network Socket Polling](architecture/io_uring_net.md): Zero-copy async network socket polling (`sys_io_uring_net`).
*   [POSIX PTP Hardware Clock Subsystem](architecture/ptp.md): IEEE 1588 nanosecond-precision hardware clock (`sys_ptp_clock`).
*   [Kernel Page Table Isolation (KPTI / KASI)](architecture/kpti.md): Ring 0 / Ring 3 page table isolation (`sys_kpti`).
*   [POSIX Sched_Autogroup Task Isolation](architecture/autogroup.md): Per-TTY terminal session autogrouping (`sys_sched_autogroup`).
*   [Kernel Callstack Unwinder Engine](architecture/unwind.md): RBP/RSP pointer frame walking for kernel panic debugging backtraces.
*   [POSIX Real-Time Signal Engine & Process Job Control](architecture/signal.md): POSIX signals (`SIGKILL`, `SIGTERM`, `SIGINT`) and terminal job control (`sys_kill`).
*   [POSIX Shared Memory IPC & Semaphore Subsystem](architecture/shm_sem.md): Shared physical memory pages (`shmget`/`shmat`) and counting semaphores (`sys_shm_sem`).

### 2. Device Drivers
*   [NVMe PCIe Controller Driver](drivers/nvme.md): High-speed NVMe 1.4 PCIe SSD storage driver with Admin Queues, Doorbell registers, and Namespace mapping.
*   [VGA Text Console, Code Editor & VBE Framebuffer](drivers/vga.md): Display buffer manipulation, cursor positioning, PS/2 input, interactive 128-line code editor (`edit`), and VBE Auto-Adaptive 32-bpp Linear Framebuffer Graphics (`framebuffer`).
*   [Serial UART COM1](drivers/serial.md): Low-level 16550A serial communication driver for boot debugging logs.
*   [Sound Programming](drivers/sound.md): Programming PIT Channel 2 for PC Speaker sound generation and Intel High Definition Audio (HDA) DMA controller initialization.
*   [Intel High Definition Audio (HDA) DSP & WAV Streaming Engine](drivers/audio_dsp.md): Audio DMA stream ring buffers, RIFF WAV header parsing, and master volume control (`sys_audio_dsp`).
*   [USB Mass Storage & USB HID Device Subsystem](drivers/usb_storage.md): USB Bulk-Only Transport (BOT) framing, SCSI commands, FAT16 flash drive mounting, and USB HID parsing (`sys_usb_device`).
*   [PS/2 Mouse Driver](drivers/mouse.md): PS/2 mouse packet decoding, resolution setup, and coordinate tracking.
*   [CMOS Real-Time Clock Driver](drivers/rtc.md): CMOS Real-Time Clock register queries and UTC timestamp parsing.
*   [Intel e1000 Network Driver & Socket API](drivers/network.md): PCI enumeration, MAC address parsing, TCP state engine, DHCP client, UDP 53 DNS Resolver with 16-slot cache table, Dynamic ARP cache, POSIX Sockets, and Native TLS 1.3 Engine (`https`).
*   [In-Kernel Stateful NAT Firewall Engine](drivers/netfilter.md): Stateful IPv4 packet filtering, IPTables rules, and 1:N NAT masquerading (`sys_netfilter`).
*   [USB Host Controller Driver](drivers/usb.md): PCI enumeration for xHCI/EHCI/UHCI USB controllers, descriptor decoding, and bus status querying (`usb`).
*   [USB 3.0 xHCI Isochronous Driver](drivers/xhci.md): High-speed USB 3.0 xHCI isochronous transfer ring buffers (`sys_xhci_iso`).

### 3. Filesystems & Storage
*   [Virtual Filesystem (VFS)](filesystems/vfs.md): Core VFS traits, Keira native directory structure (`/system/dev/`), POSIX `/dev/` path aliasing, file descriptors, and abstraction layers.
*   [Hardware RAID & Logical Volume Manager (LVM)](filesystems/lvm_raid.md): Storage pooling over SATA/NVMe, Volume Groups, and RAID 0/1 arrays (`sys_raid_lvm`).
*   [Native EXT4 Filesystem Driver](filesystems/ext4.md): Native Linux EXT4 superblock parsing, inode table reading, and extent tree block mapping.
*   [FAT Filesystem](filesystems/fat.md): FAT12/16/32 directory walking, cluster allocation tables, long file name (LFN) entries, cluster read/write/append operations, sector block cache `sync`, and native file protection (`protect`, `fileinfo`).
*   [TAR Archive Reader](filesystems/tar.md): Read-only parsing of the USTAR archive format loaded as the boot initrd.

### 4. Userland Subsystems & IPC
*   [User Runtime Library (libc & Extensions)](userland/runtime.md): Dynamic memory allocation (malloc), POSIX stdio file I/O, environment variables, socket programming (`socket.h`), C Math (`math.h`) & Time (`time.h`), and system call wrappers.
*   [Multi-User Account Management](userland/users.md): Persistent user management (`user`), password storage (`/system/etc/passwd`), dynamic prompt, 3-attempt retry fallback, and UNIX privilege separation.
*   [System Hostname Configuration](userland/hostname.md): System hostname configuration (`hostname`) persisted to `/system/etc/hostname`.
*   [POSIX File Security & Attributes](userland/permissions.md): POSIX file security & protection flags (`protect`, `fileinfo`).
*   [Multi-Virtual Terminal Subsystem](userland/tty.md): Virtual terminal switching (`tty1`..`tty4`) and console screen buffers.
*   [The Init Process](userland/init.md): User-space initialization sequence (`bin/init`) spawning system processes.
*   [Self-Hosting C Compiler](userland/gcc.md): Parser, lexer, AST builder, and helper structures inside the built-in C compiler (`bin/gcc`).

### 5. Contribution Guidelines
*   [Workspace Setup](contributing/setup.md): Installing the toolchain, cross-compiler packages, Rust nightly, and emulator targets.
*   [Building and Running](contributing/build.md): Makefile targets for compilation, ISO creation, disk image partitioning, and QEMU configuration.
*   [Coding Style Standards](contributing/style.md): Style rules, comment standards for C/Assembly/Rust, and linter configurations.

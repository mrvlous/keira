# Keira Kernel Documentation Map

Welcome to the technical documentation for Keira Kernel. This modular documentation system is designed to provide developers and contributors with a clear, in-depth understanding of the operating system's design, architecture, and code layout.

## Documentation Directory Structure

To help you navigate the codebase, the documentation is divided into the following categories:

### 1. Architecture and Core Kernel
*   [Bootstrapping and Trampolining](architecture/bootstrapping.md): The multi-stage boot sequence from GRUB to Rust 64-bit Long Mode.
*   [Memory Management](architecture/memory.md): Design of the Physical Memory Manager (PMM), Virtual Memory Manager (VMM), `sys_mmap`/`sys_munmap` page allocation, and early C bump heap allocator.
*   [Task Scheduler](architecture/scheduler.md): Preemptive priority multitasking model, scheduler queue, SMP multi-core execution (`smp_init`), and Unix signals (`sys_kill`).
*   [System Calls and Interrupts](architecture/syscalls.md): Exception handling, 39 system call vectors, Local APIC controller, dynamic TSS RSP0 stack switching, process cloning (`sys_fork`), virtual memory protection (`sys_mprotect`/`sys_madvise`), loadable kernel modules (`sys_init_module`), high-precision HPET timer (`sys_clock_gettime`), kernel unwinder (`sys_ptrace`), async I/O (`sys_io_uring_setup`/`sys_io_uring_enter`), and TLS 1.3 encrypted connections (`sys_tls_connect`).
*   [Cryptographic Subsystem](architecture/crypto.md): Bare-metal Rust `no_std` implementations of SHA-256/HMAC, AES-128-GCM AEAD, and Curve25519 X25519 ECDH key exchange.
*   [LKM, HPET & SMP Subsystems](architecture/lkm_hpet_smp.md): Dynamically Loadable Kernel Modules (`sys_init_module`), HPET nanosecond timer (`sys_clock_gettime`), kernel unwinder (`sys_ptrace`), and SMP IPI TLB shootdown.
*   [PCIe, io_uring & NX/KASLR Subsystems](architecture/pcie_iouring_nx.md): PCIe ECAM & MSI/MSI-X interrupts, asynchronous kernel I/O (`io_uring`), hardware NX bit enforcement, and KASLR randomization.

### 2. Device Drivers
*   [VGA Text Console, Code Editor & VBE Framebuffer](drivers/vga.md): Display buffer manipulation, cursor positioning, PS/2 input, interactive 128-line code editor (`edit`), and VBE Auto-Adaptive 32-bpp Linear Framebuffer Graphics (`framebuffer`).
*   [Serial UART COM1](drivers/serial.md): Low-level 16550A serial communication driver for boot debugging logs.
*   [Sound Programming](drivers/sound.md): Programming PIT Channel 2 for PC Speaker sound generation and Intel High Definition Audio (HDA) DMA controller initialization.
*   [Mouse and RTC Drivers](drivers/mouse_rtc.md): PS/2 mouse packet decoding, resolution setup, and CMOS Real-Time Clock register queries.
*   [Intel e1000 Network Driver & Socket API](drivers/network.md): PCI enumeration, MAC address parsing, TCP state engine, DHCP client, UDP 53 DNS Resolver with 16-slot cache table, Dynamic ARP cache, POSIX Sockets, and Native TLS 1.3 Engine (`https`).
*   [USB Host Controller Driver](drivers/usb.md): PCI enumeration for xHCI/EHCI/UHCI USB controllers, descriptor decoding, and bus status querying (`usb`).

### 3. Filesystems & Storage
*   [Virtual Filesystem (VFS)](filesystems/vfs.md): Core VFS traits, Keira native directory structure (`/system/dev/`), POSIX `/dev/` path aliasing, file descriptors, and abstraction layers.
*   [TAR Archive Reader](filesystems/tar.md): Read-only parsing of the USTAR archive format loaded as the boot initrd.
*   [FAT Filesystem](filesystems/fat.md): FAT12/16/32 directory walking, cluster allocation tables, long file name (LFN) entries, cluster read/write/append operations, sector block cache `sync`, and native file protection (`protect`, `fileinfo`).

### 4. Userland Subsystems & IPC
*   [User Runtime Library (libc & Extensions)](userland/runtime.md): Dynamic memory allocation (malloc), POSIX stdio file I/O, environment variables, socket programming (`socket.h`), C Math (`math.h`) & Time (`time.h`), and system call wrappers.
*   [Multi-User Accounts & System Hostname](userland/users_hostname.md): Persistent user management (`user`), password storage (`/system/etc/passwd`), system hostname configuration (`hostname`), dynamic prompt, 3-attempt retry fallback, and UNIX privilege separation.
*   [POSIX File Permissions, Redirection & Multi-TTY](userland/tty_permissions.md): POSIX file security & permissions (`chmod`/`protect`), file I/O redirection (`>`, `>>`, `<`), multi-stage pipe chains (`|`), and Multi-Virtual Terminal Subsystem (`tty`).
*   [The Init Process](userland/init.md): User-space initialization sequence (`bin/init`) spawning system processes.
*   [Self-Hosting C Compiler](userland/gcc.md): Parser, lexer, AST builder, and helper structures inside the built-in C compiler (`bin/gcc`).

### 5. Contribution Guidelines
*   [Workspace Setup](contributing/setup.md): Installing the toolchain, cross-compiler packages, Rust nightly, and emulator targets.
*   [Building and Running](contributing/build.md): Makefile targets for compilation, ISO creation, disk image partitioning, and QEMU configuration.
*   [Coding Style Standards](contributing/style.md): Style rules, comment standards for C/Assembly/Rust, and linter configurations.

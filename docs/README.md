# Keira Kernel Documentation Map

Welcome to the technical documentation for Keira Kernel. This modular documentation system is designed to provide developers and contributors with a clear, in-depth understanding of the operating system's design, architecture, and code layout.

## Documentation Directory Structure

To help you navigate the codebase, the documentation is divided into the following categories:

### 1. Architecture and Core Kernel
*   [Bootstrapping and Trampolining](architecture/bootstrapping.md): The multi-stage boot sequence from GRUB to Rust 64-bit Long Mode.
*   [Memory Management](architecture/memory.md): Design of the Physical Memory Manager (PMM), Virtual Memory Manager (VMM), and early C bump heap allocator.
*   [Task Scheduler](architecture/scheduler.md): Multitasking model, scheduler queue, context structures, and switching mechanics.
*   [System Calls and Interrupts](architecture/syscalls.md): Exception handling, system call vector dispatching, and user-to-kernel space privilege level transitions.

### 2. Device Drivers
*   [VGA Text Console](drivers/vga.md): Display buffer manipulation, cursor positioning, and text-mode mouse cursor rendering.
*   [Serial UART COM1](drivers/serial.md): Low-level 16550A serial communication driver for boot debugging logs.
*   [Sound Programming](drivers/sound.md): Programming PIT Channel 2 for PC Speaker sound generation and Intel High Definition Audio (HDA) DMA controller initialization.
*   [Mouse and RTC Drivers](drivers/mouse_rtc.md): PS/2 mouse packet decoding, resolution setup, and CMOS Real-Time Clock register queries.
*   [Intel e1000 Network Driver](drivers/network.md): PCI enumeration, MAC address parsing, TCP state engine, DHCP client, and UDP 53 DNS Resolver.

### 3. Filesystems
*   [Virtual Filesystem (VFS)](filesystems/vfs.md): Core VFS traits, mount points, file descriptors, and abstraction layers.
*   [TAR Archive Reader](filesystems/tar.md): Read-only parsing of the USTAR archive format loaded as the boot initrd.
*   [FAT Filesystem](filesystems/fat.md): FAT12/16/32 directory walking, cluster allocation tables, long file name (LFN) entries, and cluster read/write operations.

### 4. Userland Subsystems
*   [User Runtime Library (libc)](userland/runtime.md): Dynamic memory allocation (malloc), POSIX stdio file I/O, environment variables, string operations, and system call wrappers.
*   [The Init Process](userland/init.md): User-space initialization sequence (`bin/init`) spawning system processes.
*   [Self-Hosting C Compiler](userland/gcc.md): Parser, lexer, AST builder, and helper structures inside the built-in C compiler (`bin/gcc`).

### 5. Contribution Guidelines
*   [Workspace Setup](contributing/setup.md): Installing the toolchain, cross-compiler packages, Rust nightly, and emulator targets.
*   [Building and Running](contributing/build.md): Makefile targets for compilation, ISO creation, disk image partitioning, and QEMU configuration.
*   [Coding Style Standards](contributing/style.md): Style rules, comment standards for C/Assembly/Rust, and linter configurations.

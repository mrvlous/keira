<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel Documentation Map

Welcome to the technical documentation for Keira Kernel. Built as an open, educational systems programming learning journey, this documentation system is organized into **13 first-class hyper-modular domains** providing exhaustive, in-depth architectural and implementation details.

* **Official Website & Interactive Showcase**: [https://mrvlous.github.io/keira/](https://mrvlous.github.io/keira/)
* **Source Repository**: [https://github.com/mrvlous/keira](https://github.com/mrvlous/keira)

---

## The Learning Journey

* **[The Keira Learning Journey](journey/README.md)**: The engineering journal, design philosophy, and step-by-step milestones (from CPU bootstrap to bare-metal networking and userland C compilers).

---

## 13 First-Class Hyper-Modular Domains

| Domain Module | Path | Description |
| :--- | :--- | :--- |
| **Learning Journey** | [`journey/`](journey/README.md) | Engineering journal and educational milestones 1 through 6 |
| **Kernel Core** | [`kernel/`](kernel/README.md) | Multiboot2, entry trampolines, GDT, TSS, IDT, APIC timers, HAL, and panic |
| **Memory** | [`memory/`](memory/README.md) | Physical frame allocator (PMM), 4-level paging (VMM), heap, DMA, and swap |
| **Task & Scheduling** | [`task/`](task/README.md) | Preemptive scheduler, context switching, task descriptors, cgroups, and signals |
| **System Calls** | [`syscall/`](syscall/README.md) | System call vector table, dispatcher ABI, and validated user copying |
| **IPC** | [`ipc/`](ipc/README.md) | Anonymous pipes, zero-copy splice, shared memory, futex, eventfd, and mqueue |
| **Filesystems** | [`fs/`](fs/README.md) | Virtual Filesystem (VFS), FAT12/16/32, EXT4, USTAR initrd, and sector caching |
| **Hardware Drivers** | [`drivers/`](drivers/README.md) | Block storage, NICs, VGA/VBE, serial UART, sound, PCI/USB, and TTYs |
| **Networking Stack** | [`net/`](net/README.md) | Layered bare-metal TCP/IP stack, ARP, IPv4, UDP, TCP, TLS 1.3, and firewall |
| **Cryptography** | [`crypto/`](crypto/README.md) | SHA-256, AES-128-GCM, Curve25519, TPM 2.0 enclave, Seccomp BPF, and MAC |
| **Shell & Utilities** | [`shell/`](shell/README.md) | Command line interface, `kvi` editor, autocomplete, history, and 78 commands |
| **Userland & C SDK** | [`userland/`](userland/README.md) | C runtime headers, in-kernel KCC compiler, dynamic ELF loader, and multi-user |
| **Contributor Guide** | [`contributing/`](contributing/README.md) | Environment setup, build targets, coding style, testing, and debugging |

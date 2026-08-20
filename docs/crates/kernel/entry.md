<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel Entry Point (`kernel_main`)

Documentation for bootstrap flow in [`crates/kernel/src/entry.rs`](../../../crates/kernel/src/entry.rs).

## Bootstrapping Sequence
1. **Multiboot2 Tag Traversal**: Extracts memory map, initrd module location, and VBE framebuffer information.
2. **Memory Subsystem**: Calls `keira_mem::init()` to configure PMM frame allocator and map linear framebuffer pages.
3. **Task Scheduler**: Initializes preemptive round-robin scheduler tables.
4. **Hardware Drivers**: Initializes PCI bus, AHCI SATA, IDE, HDA sound, and Intel e1000 NIC.
5. **Filesystem Mount**: Mounts AHCI/IDE block storage as FAT16 root filesystem.
6. **Privilege Transition**: Programs GDT, TSS IST stack, and syscall MSRs (`EFER`, `STAR`, `LSTAR`, `FMASK`).
7. **Shell Spawn**: Clears screen, displays ASCII welcome banner, enables CPU interrupts (`sti`), and enters interactive shell event loop.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Pure Rust Kernel Boot Pipeline

Keira Kernel's boot sequence transitions directly from assembly trampolines to the pure Rust kernel:

```
[ BIOS / GRUB ]
      │ (32-bit Protected Mode, Multiboot2 Magic 0x36D76289)
      v
[ arch/x86/boot/entry32.asm ]  --> Verifies CPUID & Long Mode, sets up temporary page tables
      │
      v
[ arch/x86/boot/entry64.asm ]  --> Enables Long Mode (EFER.LME, CR0.PG), loads GDT64, sets RSP
      │
      v
[ crates/kernel/src/entry.rs ] --> Pure Rust kernel_main():
                                   - keira-arch (IDT, PIC, PIT 1000Hz)
                                   - keira-io (PS/2 Keyboard, Mouse, CMOS RTC, Serial, VGA)
                                   - keira-mem (PMM, VMM, Pure Rust Bump Heap)
                                   - keira-task (Scheduler)
                                   - keira-fs / keira-net (PCI, AHCI, e1000, FAT16)
                                   - keira-syscall (Syscall MSRs, Ring 3)
                                   - keira-shell (Interactive Terminal Shell)
```

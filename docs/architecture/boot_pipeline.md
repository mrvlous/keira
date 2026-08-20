<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tri-Language Boot Pipeline

Keira Kernel's boot sequence progresses through three language environments:

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
[ arch/x86/kernel/hw_init.c ]  --> C Drivers: Serial, VGA, IDT, PIC, PIT, PS/2 Keyboard/Mouse, C Heap
      │
      v
[ crates/kernel/src/entry.rs ] --> Rust kernel_main(): PMM, VMM, Scheduler, PCI, AHCI, e1000, FAT16, Shell
```

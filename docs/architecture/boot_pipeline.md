<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Pure Rust Kernel Boot Pipeline

Keira Kernel's boot sequence transitions from Multiboot2 assembly trampolines directly into the pure Rust kernel:

## Dual-Architecture Boot Sequences

### A. 64-Bit Architecture (`ARCH=x86_64`)
```
[ BIOS / GRUB ]
      │ (32-bit Protected Mode, Multiboot2 Magic 0x36D76289)
      v
[ arch/x86/boot/entry32.asm ]  --> Verifies Multiboot2, constructs 2 MiB identity pages, enables PAE
      │
      v
[ arch/x86/boot/entry64.asm ]  --> Enables Long Mode (EFER.LME, CR0.PG), loads GDT64, jumps to 64-bit code
      │
      v
[ crates/kernel/src/entry.rs ] --> Pure Rust kernel_main():
                                   - keira-arch (64-bit IDT, PIC, PIT 1000Hz)
                                   - keira-io (PS/2, CMOS RTC, Serial, VGA/Framebuffer)
                                   - keira-mem (PMM, 4-level VMM, Kernel Bump/Global Heap)
                                   - keira-task (Priority Scheduler)
                                   - keira-fs / keira-net (PCI, AHCI, NVMe, e1000, FAT16)
                                   - keira-syscall (Syscall MSRs, Ring 3 IST)
                                   - keira-shell (Interactive Terminal Shell)
```

### B. Pure 32-Bit Architecture (`ARCH=i686`)
```
[ BIOS / GRUB ]
      │ (32-bit Protected Mode, Multiboot2 Magic 0x36D76289)
      v
[ arch/x86/boot/entry32.asm ]  --> Verifies Multiboot2, loads 32-bit GDT, reloads segments
      │
      v
[ crates/kernel/src/entry.rs ] --> Pure Rust kernel_main():
                                   - keira-arch (32-bit IDT, PIC, PIT 1000Hz)
                                   - keira-io (PS/2, CMOS RTC, Serial, VGA/Framebuffer)
                                   - keira-mem (PMM, 32-bit VMM, Kernel Heap)
                                   - keira-task (Priority Scheduler)
                                   - keira-fs / keira-net (PCI, AHCI, e1000, FAT16)
                                   - keira-syscall (32-bit TSS, int 0x80 Syscalls, Ring 3)
                                   - keira-shell (Interactive Terminal Shell)
```

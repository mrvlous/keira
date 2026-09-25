<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Multiboot2 Specification & Early Handshake

Keira conforms to the Multiboot2 Specification, allowing standard bootloaders like GRUB to initialize the machine state, switch to 32-bit Protected Mode, and pass boot metadata structures to the kernel.

---

## 1. Multiboot2 Header Architecture

The Multiboot2 header is defined in `arch/x86/common/boot/multiboot2_header.asm`. It resides within the first 32 KiB of the kernel image aligned to an 8-byte boundary:

```nasm
section .multiboot_header
align 8
header_start:
    dd 0xE85250D6                ; Multiboot2 magic number
    dd 0                         ; Architecture 0 (i386 32-bit protected mode)
    dd header_end - header_start ; Header length
    dd -(0xE85250D6 + 0 + (header_end - header_start)) ; Checksum

    ; Tag: Request Framebuffer Info
    dw 5                         ; Type: Framebuffer
    dw 0                         ; Flags
    dd 20                        ; Size
    dd 1024                      ; Preferred Width
    dd 768                       ; Preferred Height
    dd 32                        ; Preferred Depth (bpp)

    ; Tag: End of tags
    dw 0
    dw 0
    dd 8
header_end:
```

---

## 2. Bootloader Handshake Registers

Upon transferring control to `_start`, the bootloader guarantees:
* **`EAX`**: Must contain the Multiboot2 bootloader magic: `0x36D76289`.
* **`EBX`**: Contains the 32-bit physical base address of the Multiboot2 Information Structure (MBI).
* **Interrupts**: Disabled (`IF = 0`).
* **A20 Gate**: Enabled.
* **Paging**: Disabled (Direct flat physical addressing).

---

## 3. Parsing Multiboot2 Tags

The kernel parses the MBI sequentially in `crates/kernel/src/boot/multiboot/`. Each tag begins with:
```rust
#[repr(C)]
pub struct MultibootTagHeader {
    pub tag_type: u32,
    pub size: u32,
}
```

### Critical Tag Types Handled:
| Tag ID | Name | Description |
| :--- | :--- | :--- |
| `0` | End Tag | Terminator indicating no further tags |
| `1` | Command Line | Kernel boot arguments passed from GRUB |
| `2` | Bootloader Name | String identifying bootloader (e.g. `GRUB 2.12`) |
| `3` | Modules | Information about loaded modules (e.g. `initrd.tar`) |
| `4` | Basic Memory Info | Lower and upper conventional memory limits |
| `6` | Memory Map | Comprehensive physical memory layout (Usable, Reserved, ACPI) |
| `8` | Framebuffer Info | VESA/EFI linear framebuffer address, resolution, pitch, and pixel format |
| `9` | ELF Symbols | Kernel symbol table for stack unwinding |
| `14` | ACPI Old RSDP | ACPI 1.0 Root System Description Pointer address |
| `15` | ACPI New RSDP | ACPI 2.0+ Extended System Description Pointer address |

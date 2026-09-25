<!-- SPDX-License-Identifier: GPL-2.0-only -->

# ELF Linker Scripts (`linker.ld`)

Linker scripts dictate the virtual memory placement of userland binaries, ensuring compatibility with the kernel's virtual memory manager and demand paging.

---

## i686 Virtual Memory Layout (`userland/arch/x86/i686/linker.ld`)

Userland binaries for 32-bit x86 are linked to load at virtual base `0x08048000`:

```ld
ENTRY(_start)

SECTIONS
{
    . = 0x08048000;

    .text : ALIGN(4K) {
        *(.text*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata*)
    }

    .data : ALIGN(4K) {
        *(.data*)
    }

    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss*)
    }

    /DISCARD/ : {
        *(.comment)
        *(.note*)
    }
}
```

---

## x86_64 Virtual Memory Layout (`userland/arch/x86/x86_64/linker.ld`)

Userland binaries for 64-bit x86_64 are linked to load at virtual base `0x00400000`:

```ld
ENTRY(_start)

SECTIONS
{
    . = 0x00400000;

    .text : ALIGN(4K) {
        *(.text*)
    }

    .rodata : ALIGN(4K) {
        *(.rodata*)
    }

    .data : ALIGN(4K) {
        *(.data*)
    }

    .bss : ALIGN(4K) {
        *(COMMON)
        *(.bss*)
    }

    /DISCARD/ : {
        *(.comment)
        *(.note*)
    }
}
```

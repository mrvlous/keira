; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Multiboot2 Specification Header
;
; This section must appear within the first 32,768 bytes of the kernel binary image.
; The linker script explicitly places the `.multiboot_header` section before all other
; sections so that Multiboot2-compliant bootloaders (such as GRUB2) can locate and
; validate it during boot.
;
; Reference: Multiboot2 Specification, Section 3.1

%include "constants.inc"

section .multiboot_header
align 8

header_start:
    dd MULTIBOOT2_MAGIC
    dd MULTIBOOT2_ARCH_I386
    dd header_end - header_start
    dd -(MULTIBOOT2_MAGIC + MULTIBOOT2_ARCH_I386 + (header_end - header_start))

    ; Framebuffer Request Tag (Tag Type 5)
    align 8
    dw 5
    dw 0
    dd 20
    dd 0
    dd 0
    dd 32

    ; End Tag (Tag Type 0)
    align 8
    dw 0
    dw 0
    dd 8
header_end:

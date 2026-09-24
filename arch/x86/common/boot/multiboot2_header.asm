; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Multiboot2 Specification Header Definition
;
; Architectural Role:
;   This assembly source file defines the canonical Multiboot2 binary header
;   conforming to the Free Software Foundation (FSF) Multiboot2 Specification.
;   Compliant bootloaders (such as GRUB2) search the first 32,768 bytes (32 KiB)
;   of the kernel ELF payload for this specific header structure.
;
; Placement and Invariants:
;   The header MUST be 64-bit (8-byte) aligned.
;   The linker script explicitly places the .multiboot_header section at
;   the very beginning of the loaded kernel binary (immediately after the
;   1 MiB physical load offset) to guarantee it resides well within the
;   initial 32 KiB window.
;   All tags within the Multiboot2 stream must be aligned on 8-byte boundaries.
;
; References:
;   Multiboot2 Specification, Version 2.0 (Section 3.1: "OS image format")
;   GNU GRUB Manual: Multiboot2 Header Architecture

%include "constants.inc"

section .multiboot_header
align 8

header_start:
    ; Mandatory Header Fields (16 bytes total)
    ; magic:
    ;   Identifies this image as compliant with Multiboot2 specification.
    ;   Defined as 0xE85250D6 (MULTIBOOT2_MAGIC).
    dd MULTIBOOT2_MAGIC

    ; architecture:
    ;   Specifies the CPU architecture expected at handoff:
    ;     0 = 32-bit (i386) protected mode (MULTIBOOT2_ARCH_I386).
    ;     4 = MIPS.
    ;   Even for x86_64 kernels, GRUB2 enters via 32-bit protected mode.
    dd MULTIBOOT2_ARCH_I386

    ; header_length:
    ;   Total size in bytes of the entire Multiboot2 header structure,
    ;   including all tags and the terminating end tag.
    dd header_end - header_start

    ; checksum:
    ;   32-bit unsigned addition with the above three fields must equal zero:
    ;     checksum = -(magic + architecture + header_length)
    ;   The bootloader validates this arithmetic identity before proceeding.
    dd -(MULTIBOOT2_MAGIC + MULTIBOOT2_ARCH_I386 + (header_end - header_start))

    ; Multiboot2 Tag: Framebuffer Request (Tag Type 5)
    ; Informs the bootloader that the kernel requests a linear VBE/GOP graphics
    ; framebuffer mode to be configured prior to kernel entry.
    ;
    ; Structure:
    ;   u16 type   = 5   (MULTIBOOT_HEADER_TAG_FRAMEBUFFER)
    ;   u16 flags  = 0   (0 = Optional requirement; non-fatal if unsupported)
    ;   u32 size   = 20  (Total byte size of this tag)
    ;   u32 width  = 0   (0 = No preference / fallback to bootloader default)
    ;   u32 height = 0   (0 = No preference / fallback to bootloader default)
    ;   u32 depth  = 32  (32-bit true color: 8-bit ARGB/RGBA per pixel)
    align 8
    dw 5                ; Tag Type: Framebuffer Request
    dw 0                ; Flags: Optional tag (bit 0 = 0)
    dd 20               ; Tag Size: 20 bytes
    dd 0                ; Requested Width: 0 (bootloader default / optimal)
    dd 0                ; Requested Height: 0 (bootloader default / optimal)
    dd 32               ; Requested Depth: 32 bits per pixel (TrueColor)

    ; Multiboot2 Tag: Termination / End Tag (Tag Type 0)
    ; Marks the end of the Multiboot2 header tag array.
    ;
    ; Structure:
    ;   u16 type  = 0   (MULTIBOOT_HEADER_TAG_END)
    ;   u16 flags = 0
    ;   u32 size  = 8   (Header size itself, no payload)
    align 8
    dw 0                ; Tag Type: End Tag
    dw 0                ; Flags: 0
    dd 8                ; Tag Size: 8 bytes

header_end:

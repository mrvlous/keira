; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Architecture Assembly Constants and Hardware Definitions
;
; Architectural Role:
;   This include file defines foundational constants, physical memory offsets,
;   CPU control register bitmasks, Model-Specific Register (MSR) addresses, and
;   segment selectors shared across all low-level x86/x86_64 bootstrap and
;   privilege transition assembly routines in Keira.
;
; Reference Specifications:
;   Intel 64 and IA-32 Architectures Software Developer Manual (Volume 3A/3B)
;   AMD64 Architecture Programmer Manual (Volume 2: System Programming)
;   Multiboot2 Specification, Version 2.0

; Multiboot2 Protocol Constants
; MULTIBOOT2_MAGIC:
;   Magic number placed in the Multiboot2 header to identify compliance.
MULTIBOOT2_MAGIC        equ 0xE85250D6

; MULTIBOOT2_ARCH_I386:
;   Architecture specifier denoting standard 32-bit x86 protected mode entry.
MULTIBOOT2_ARCH_I386    equ 0

; MULTIBOOT2_BOOTLOADER:
;   Magic value passed by compliant bootloader in the EAX register upon entry.
MULTIBOOT2_BOOTLOADER   equ 0x36D76289

; Physical Memory and Kernel Stack Topology
; KERNEL_PHYS_BASE:
;   Physical memory load base address (1 MiB boundary).
;   Avoids legacy IBM PC real-mode memory holes, BIOS data areas (BDA),
;   and video display memory buffers residing below 1 MiB.
KERNEL_PHYS_BASE        equ 0x00100000

; KERNEL_STACK_SIZE:
;   Size of the initial bootstrap execution stack in bytes (32 KiB).
;   Sufficient for early C/Rust runtime setup before dynamic thread stacks exist.
KERNEL_STACK_SIZE       equ 0x8000

; VGA Text Mode Hardware Layout (MMIO)
; VGA_BUFFER_ADDR:
;   Linear physical address of the standard 80x25 text-mode video buffer.
;   Each character cell consists of 2 bytes: [ASCII byte][Attribute byte].
VGA_BUFFER_ADDR         equ 0x000B8000

; VGA Grid Dimensions:
VGA_COLUMNS             equ 80
VGA_ROWS                equ 25

; VGA Color Attributes:
;   Bit 0-3: Foreground color (0x0F = White, 0x0C = Light Red).
;   Bit 4-6: Background color (0x00 = Black).
;   Bit 7:   Blink / Bright background enable.
VGA_ATTR_WHITE_ON_BLACK equ 0x0F
VGA_ATTR_RED_ON_BLACK   equ 0x0C

; UART Serial Communication Ports (I/O Port Addresses)
; COM1_PORT:
;   Base I/O port address for the primary 16550-compatible serial UART.
;   Used for early headless kernel diagnostic output and kernel panic logging.
COM1_PORT               equ 0x03F8

; CPU Control Registers (CR0, CR4) Bit Definitions
; CR0_PROTECTED_BIT:
;   Bit 0 (PE - Protection Enable): Enables 32-bit protected mode execution.
CR0_PROTECTED_BIT       equ (1 << 0)

; CR0_PAGING_BIT:
;   Bit 31 (PG - Paging): Enables hardware memory paging.
;   In combination with EFER.LME, transitions the processor into IA-32e Long Mode.
CR0_PAGING_BIT          equ (1 << 31)

; CR4_PAE_BIT:
;   Bit 5 (PAE - Physical Address Extension): Enables 36-bit/52-bit physical
;   addressing and 64-bit page table descriptors. Mandatory prerequisite for Long Mode.
CR4_PAE_BIT             equ (1 << 5)

; Model-Specific Registers (MSR) and Extended Feature Bits
; EFER_MSR:
;   Extended Feature Enable Register (IA32_EFER, address 0xC0000080).
;   Controls activation of 64-bit Long Mode, Syscall Extensions, and NX memory protection.
EFER_MSR                equ 0xC0000080

; EFER_LONG_MODE_BIT:
;   Bit 8 (LME - Long Mode Enable): Arms processor for Long Mode upon CR0.PG assertion.
EFER_LONG_MODE_BIT      equ (1 << 8)

; EFER_NXE_BIT:
;   Bit 11 (NXE - No-Execute Enable): Enables the execute-disable (XD/NX) bit 63
;   in page table entries to enforce hardware W^X memory security.
EFER_NXE_BIT            equ (1 << 11)

; Page Table Architecture Bitmasks (x86_64 4-Level Paging)
; PAGE_PRESENT:
;   Bit 0 (P): Page is present in physical memory.
PAGE_PRESENT            equ (1 << 0)

; PAGE_WRITABLE:
;   Bit 1 (R/W): Read/Write allowed (1 = writable, 0 = read-only).
PAGE_WRITABLE           equ (1 << 1)

; PAGE_HUGE:
;   Bit 7 (PS - Page Size): Designates a 2 MiB huge page when set in a Page Directory entry.
PAGE_HUGE               equ (1 << 7)

; PAGE_RW_PRESENT:
;   Convenience composite mask: Present and Writable.
PAGE_RW_PRESENT         equ (PAGE_PRESENT | PAGE_WRITABLE)

; Global Descriptor Table (GDT) Segment Selectors
; GDT_NULL_SEL:
;   Index 0 (Offset 0x00): Mandatory CPU null segment selector.
GDT_NULL_SEL            equ 0x00

; GDT_CODE64_SEL:
;   Index 1 (Offset 0x08): 64-bit Kernel Code Segment (RPL 0, DPL 0, Long Mode flag set).
GDT_CODE64_SEL          equ 0x08

; GDT_DATA64_SEL:
;   Index 2 (Offset 0x10): 64-bit Kernel Data Segment (RPL 0, DPL 0, Writable).
GDT_DATA64_SEL          equ 0x10

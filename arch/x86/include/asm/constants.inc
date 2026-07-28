; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Operating System Kernel
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Architecture Assembly Constants
;
; Shared constants, physical memory offsets, and CPU configuration bits
; used across low-level x86 bootstrap routines.

; Multiboot2 Specification Definitions
MULTIBOOT2_MAGIC        equ 0xE85250D6
MULTIBOOT2_ARCH_I386    equ 0
MULTIBOOT2_BOOTLOADER   equ 0x36D76289

; Memory Topology Configuration
KERNEL_PHYS_BASE        equ 0x00100000
KERNEL_STACK_SIZE       equ 0x4000

; VGA Text Mode Hardware Layout
VGA_BUFFER_ADDR         equ 0xB8000
VGA_COLUMNS             equ 80
VGA_ROWS                equ 25
VGA_ATTR_WHITE_ON_BLACK equ 0x0F
VGA_ATTR_RED_ON_BLACK   equ 0x0C

; UART Serial Communication Ports
COM1_PORT               equ 0x3F8

; CPU Control Registers and Model-Specific Registers (MSR)
CR0_PROTECTED_BIT       equ (1 << 0)
CR0_PAGING_BIT          equ (1 << 31)
CR4_PAE_BIT             equ (1 << 5)

EFER_MSR                equ 0xC0000080
EFER_LONG_MODE_BIT      equ (1 << 8)

; Page Table Architecture Flags
PAGE_PRESENT            equ (1 << 0)
PAGE_WRITABLE           equ (1 << 1)
PAGE_HUGE               equ (1 << 7)
PAGE_RW_PRESENT         equ (PAGE_PRESENT | PAGE_WRITABLE)

; Global Descriptor Table (GDT) Segment Selectors
GDT_NULL_SEL            equ 0x00
GDT_CODE64_SEL          equ 0x08
GDT_DATA64_SEL          equ 0x10

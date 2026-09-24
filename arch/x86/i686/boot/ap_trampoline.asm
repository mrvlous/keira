; SPDX-License-Identifier: GPL-2.0-only
;
; Keira Kernel - Freestanding Kernel from Scratch
; Copyright (C) 2026 Moh. Ananda Firmansyah Putra
;
; This program is free software; you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation; version 2 of the License.

; Application Processor (AP) 16-Bit Real-Mode Bootstrap Trampoline (i686)
;
; Architectural Role:
;   This assembly code serves as the SMP initialization trampoline for secondary
;   cores (Application Processors - APs) on 32-bit x86 systems.
;
; Multi-Core Bootstrap Sequence:
;   1. The Bootstrap Processor (BSP) discovers secondary cores via ACPI MADT.
;   2. The BSP copies this trampoline binary payload to physical page 0x8000
;      (page frame number 0x08).
;   3. The BSP populates the parameter block (target entry address, per-core
;      kernel stack, and unique AP core ID).
;   4. The BSP dispatches an Inter-Processor Interrupt (IPI) sequence:
;      - INIT IPI: Asserts hardware reset on the target AP core.
;      - 10 ms delay.
;      - Startup IPI (SIPI) with vector 0x08 (directing the AP to begin
;        execution at physical address 0x08 * 4096 = 0x8000).
;   5. The AP starts executing at ap_real16_entry in 16-bit real mode.
;   6. The AP enables Protected Mode (CR0.PE) and transfers control to the
;      assigned Rust kernel entry point (ap_main).

%include "constants.inc"

; Base physical relocation address where BSP copies this trampoline code
%define AP_TRAMPOLINE_BASE 0x8000

section .rodata
global ap_trampoline_start
global ap_trampoline_end
global ap_cr3_val
global ap_stack_val
global ap_entry_val
global ap_core_id
global ap_status_flag

ap_trampoline_start:
[bits 16]
ap_real16_entry:
    ; Step 1: Real-Mode Hardware Initialization
    ; Disable interrupts immediately to prevent unexpected BIOS/PIC interrupts.
    cli
    cld                     ; Clear direction flag (string ops increment)

    ; Zero-initialize segment registers for flat real-mode addressing (CS = 0x0800)
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00          ; Temporary real-mode scratch stack

    ; Step 2: Load Temporary 32-Bit GDT
    ; The GDT descriptor is addressed relative to the known AP_TRAMPOLINE_BASE.
    lgdt [AP_TRAMPOLINE_BASE + (ap_gdt_desc - ap_trampoline_start)]

    ; Step 3: Transition to 32-Bit Protected Mode
    ; Set Protection Enable bit (PE = 1) in Control Register 0 (CR0).
    mov eax, cr0
    or al, 1
    mov cr0, eax

    ; Far jump to 32-bit code segment (Selector 0x08) to serialize pipeline
    ; and reload CS register.
    jmp dword 0x08:(AP_TRAMPOLINE_BASE + (ap_pm32_entry - ap_trampoline_start))

[bits 32]
ap_pm32_entry:
    ; Step 4: Reload Protected Mode Data Segment Selectors
    ; Selector 0x10 points to 32-bit Data Segment (Base 0, Limit 4GB, DPL 0).
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax

    ; Step 5: Establish Per-Core Kernel Execution Stack
    ; Read pre-allocated kernel stack pointer assigned to this specific core.
    mov esp, [AP_TRAMPOLINE_BASE + (ap_stack_val - ap_trampoline_start)]

    ; Step 6: Invoke Rust Secondary Core Entrypoint (ap_main)
    ; Push core_id argument for standard cdecl calling convention:
    ;   extern "C" fn ap_main(core_id: usize) -> !;
    push dword [AP_TRAMPOLINE_BASE + (ap_core_id - ap_trampoline_start)]

    ; Call target Rust entrypoint address configured by BSP.
    mov eax, [AP_TRAMPOLINE_BASE + (ap_entry_val - ap_trampoline_start)]
    call eax

    ; Fallback Idle Loop
    ; In case ap_main ever returns, enter a permanent disabled halt state.
.ap_halt:
    cli
    hlt
    jmp .ap_halt

; Trampoline Parameter Block (Populated dynamically by BSP prior to SIPI)
align 4
ap_cr3_val:     dd 0        ; CR3 page directory base (if paging active)
ap_stack_val:   dd 0        ; Pointer to top of allocated per-core kernel stack
ap_entry_val:   dd 0        ; Function pointer to Rust ap_main
ap_core_id:     dd 0        ; Logical AP core index (0, 1, 2, ...)
ap_status_flag: dd 0        ; Synchronization barrier / handshake flag

; Temporary Trampoline Global Descriptor Table (GDT)
align 16
ap_gdt_start:
    ; Selector 0x00: Null Descriptor
    dq 0x0000000000000000

    ; Selector 0x08: 32-bit Kernel Code Segment (Base 0, Limit 4GB, DPL 0, Exec/Read)
    dq 0x00CF9A000000FFFF

    ; Selector 0x10: 32-bit Kernel Data Segment (Base 0, Limit 4GB, DPL 0, Read/Write)
    dq 0x00CF92000000FFFF
ap_gdt_end:

ap_gdt_desc:
    dw ap_gdt_end - ap_gdt_start - 1                        ; GDT Limit (size - 1)
    dd AP_TRAMPOLINE_BASE + (ap_gdt_start - ap_trampoline_start) ; GDT Linear Base

align 16
ap_trampoline_end:

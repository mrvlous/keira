; SPDX-License-Identifier: GPL-2.0-only

; Interrupt Descriptor Table (IDT) Register Loader (x86_64)
;
; Architectural Role:
;   This assembly routine invokes the privileged lidt instruction to load
;   the CPU Interrupt Descriptor Table Register (IDTR) on 64-bit x86_64 hardware.
;
; 64-Bit IDTR Descriptor Format (10 Bytes Total):
;   Bytes 0-1: IDT Limit (table length in bytes minus 1, e.g., 256 * 16 - 1 = 4095).
;   Bytes 2-9: 64-bit linear base address pointing to the IDT gate array.
;
; Calling Convention:
;   System V AMD64 ABI:
;     extern "C" fn idt_load(idt_ptr: *const IdtPointer);
;     Argument 1 (RDI): Linear memory address of the 10-byte IDTR structure.

section .text
bits 64

; idt_load: Load IDTR from passed pointer
global idt_load
idt_load:
    lidt [rdi]                  ; Load IDTR with table limit and 64-bit base address
    ret

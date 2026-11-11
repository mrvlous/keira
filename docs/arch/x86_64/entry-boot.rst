.. SPDX-License-Identifier: GPL-2.0-only

==============================================
Early Kernel Bringup & Initialization Sequence
==============================================

The transition from legacy 32-bit Protected Mode to 64-bit IA-32e Long Mode is implemented in ``arch/x86/x86_64/boot/entry32.asm`` and ``arch/x86/x86_64/boot/entry64.asm``.

1. Transition Pipeline
======================

1. **Stack Setup**: Allocates a temporary 32 KiB bootstrap stack in ``.bss`` aligned to 16 bytes.
2. **Multiboot2 Handshake**: Confirms that ``EAX == 0x36D76289`` and preserves the Multiboot2 info pointer in ``EBX``.
3. **Paging Construction**: Invokes ``setup_page_tables`` to identity-map the first gigabytes of physical memory.
4. **CR3 Assignment**: Loads the physical address of the Page Map Level 4 (PML4) table into ``CR3``.
5. **PAE Activation**: Sets ``CR4.PAE`` (bit 5) to enable 64-bit page table structures.
6. **EFER Configuration**: Writes to Model-Specific Register ``IA32_EFER`` (``0xC0000080``), asserting ``LME`` (bit 8) and ``NXE`` (bit 11).
7. **Paging Activation**: Asserts ``CR0.PG`` (bit 31), transitioning the CPU into IA-32e mode.
8. **64-bit Far Jump**: Executes ``jmp 0x08:_start64`` to flush the instruction pipeline and enter pure 64-bit submode.

2. 64-bit Entry Operations
==========================

Upon entering ``_start64``:

* Data segment registers (``DS``, ``ES``, ``FS``, ``GS``, ``SS``) are reloaded with ``0x10`` (kernel data descriptor).
* The 64-bit stack pointer ``RSP`` is reset to the top of the stack.
* The ``.bss`` uninitialized data section is zero-cleared.
* The Multiboot2 info pointer is forwarded as the first argument in ``RDI`` conforming to the System V AMD64 ABI.
* Control jumps to the Rust kernel entry point: ``kernel_main(multiboot_info_ptr: usize)``.

3. Subsystem Bringup Checklist
==============================

Inside ``crates/kernel/src/lib.rs``:

1. **Diagnostic Console**: UART 16550 serial init and VGA framebuffer text console.
2. **Memory Hierarchy**: PMM frame allocator and 4-level VMM paging setup.
3. **Privilege & Traps**: 64-bit GDT, TSS and 256-gate IDT table initialization.
4. **Hardware Timers**: LAPIC and HPET high-precision timer calibration.
5. **Preemptive Scheduler**: Idle task setup and scheduler start.

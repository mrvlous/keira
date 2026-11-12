.. SPDX-License-Identifier: GPL-2.0-only


Early Kernel Bringup & Initialization Sequence
==============================================

The early bringup sequence transitions the CPU from 32-bit Protected Mode assembly bootstrap to the high-level ``#![no_std]`` Rust kernel runtime.

1. 32-bit Protected Mode Assembly Bootstrap
===========================================

The 32-bit entry sequence is implemented in ``arch/x86/i686/boot/entry.asm``:

1. **Bootstrap Stack Setup**: Establishes a 32 KiB temporary stack in ``.bss`` aligned to 16 bytes for standard C/Rust ABI compliance.
2. **Bootloader Handshake**: Checks ``EAX`` for ``0x36D76289`` and preserves the Multiboot2 info pointer from ``EBX``.
3. **GDT Installation**: Loads the 32-bit Global Descriptor Table via ``lgdt``.
4. **Segment Reload**: Executes a far jump ``jmp 0x08:.flush`` to flush the prefetch queue and sets ``DS``, ``ES``, ``FS``, ``GS`` and ``SS`` to data selector ``0x10``.
5. **BSS Initialization**: Zeroes the ``.bss`` section using ``rep stosb``.
6. **Rust Entry Transition**: Pushes the Multiboot2 info pointer onto the stack and invokes ``kernel_main``.

2. High-Level Initialization Timeline
=====================================

.. code-block:: text

   graph TD
       BootASM["arch/x86/common/boot/multiboot2_header.asm & boot/entry.asm"] --> EarlySerial["UART 16550 Serial Init (0x3F8)"]
       EarlySerial --> EarlyVGA["VGA 80x25 Console Init"]
       EarlyVGA --> PMMBoot["PMM Bootstrap (Multiboot Memory Tag Parse)"]
       PMMBoot --> VMMInit["VMM Page Directory & Kernel Remap"]
       VMMInit --> RustMain["kernel_main() Entry in crates/kernel/"]
       RustMain --> CoreSubsystems["GDT, IDT, APIC and HPET Bringup"]
       CoreSubsystems --> DriversInit["PCI, Storage and Network Drivers"]
       DriversInit --> VFSInit["VFS Mount (/, /dev, /proc)"]
       VFSInit --> ShellTask["Spawn Shell / Init Process (PID 1)"]

3. Low-Level Subsystem Initialization Checklist
===============================================

Inside ``crates/kernel/src/lib.rs``:

1. **Early Diagnostic Output**:

   * UART 16550 configured to 115200 baud, 8N1 format.
   * Enables immediate kernel logging via ``early_printk!()``.

2. **Physical Frame Allocator (PMM)**:

   * Maps physical memory regions from Multiboot memory map.
   * Initializes frame allocation bitmap.

3. **Virtual Memory Manager (VMM)**:

   * Identity maps early boot pages and maps higher-half kernel space (``0xC0000000``).

4. **Segmentation & Interrupts**:

   * Installs Global Descriptor Table (GDT) and loads Task State Segment (TSS).
   * Installs Interrupt Descriptor Table (IDT) with 256 gates.

5. **Timer & Interrupt Controller**:

   * Disables legacy 8259 PIC.
   * Calibrates Local APIC timer using HPET or PIT.

6. **Task & Scheduler Subsystem**:

   * Initializes idle task (PID 0) and preemptive round-robin scheduler.

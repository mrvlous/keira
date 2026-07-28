# Bootstrapping and Trampolining

This document details the multi-stage initialization pipeline of Keira Kernel from the bootloader load phase to the transition into 64-bit Rust long mode execution.

## Stage 1: Multiboot2 Header and 32-bit Entry
When the GRUB bootloader loads the kernel, the CPU starts in 32-bit Protected Mode with interrupts disabled. The bootloader searches for the Multiboot2 header embedded at the very beginning of the kernel binary (`arch/x86/boot/entry.asm`).

Upon validating the magic value (`0x36D76289` in EAX), GRUB jumps to the entry point `_start` in `entry.asm`:
1.  **Register Validation**: Verifies that EAX contains the Multiboot2 magic code and EBX points to the Multiboot2 information structure in physical memory.
2.  **Stack Setup**: Configures a temporary boot stack to handle early register manipulations.
3.  **CPU Verification**: Executes CPUID checks to confirm that the CPU supports 64-bit Long Mode features.

## Stage 2: Page Table Setup and Long Mode Transition
To enter 64-bit Mode, paging must be enabled. The assembly trampoline constructs a basic identity-mapping page table mapping the first 1 GB of physical memory:
1.  **PML4 Entry**: Points the first entry of the Page Map Level 4 table to the physical address of the Page Directory Pointer Table (PDPT).
2.  **PDPT Entry**: Points the first entry of the PDPT to the physical Page Directory (PD).
3.  **PD Entry**: Maps the entries of the PD to physical addresses using 2 MB large pages (with the Page Size bit set to 1).

### Long Mode Activation Sequence
With page tables established, the assembly instructions proceed with the hardware configuration:
1.  **CR3 Load**: The physical address of the boot PML4 table is loaded into the CR3 register.
2.  **PAE Enable**: Physical Address Extension is enabled by setting the PAE bit (bit 5) in the CR4 control register.
3.  **IA32_EFER Enable**: The Long Mode Enable (LME) bit (bit 8) of the Extended Feature Enable Register (MSR `0xC0000080`) is set to 1.
4.  **Paging Enable**: Paging is activated by setting the PG bit (bit 31) and the PE bit (bit 0) of the CR0 control register.
5.  **Long Mode Jump**: A far jump is executed to load the 64-bit code segment descriptor, jumping into `_start64` within `arch/x86/boot/entry64.asm`.

## Stage 3: 64-bit Initialization and C/Rust Transition
Once the CPU executes in 64-bit Long Mode:
1.  **Segment Registers**: Clears data segment registers (DS, ES, FS, GS, SS) to 0.
2.  **Multiboot Pointer Preservation**: Moves the Multiboot2 pointer to RDI (the first parameter register in the System V AMD64 ABI).
3.  **C Hardware Initialization**: Jumps to `hw_init()` in the C driver architecture layer.
    *   Initializes the VGA text-mode display.
    *   Initializes the COM1 serial port for output logging.
    *   Clears the BSS segment to zero.
4.  **Rust Landing**: `hw_init()` returns, and `entry64.asm` calls `kernel_main` inside [entry.rs](../../kernel/src/entry.rs). The Multiboot2 information pointer is passed forward in RDI.

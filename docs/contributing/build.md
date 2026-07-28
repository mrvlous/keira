# Building and Running the Kernel

This document describes how to compile the kernel, package the boot image, and run the system using the QEMU emulator.

## 1. Compilation Pipeline
The build system ([Makefile](../../Makefile)) orchestrates the compilation of assembly, C, and Rust files:
1.  **NASM**: Compiles 32-bit and 64-bit boot trampoline code (`arch/x86/boot/`) and exception handler stubs into ELF64 object files.
2.  **GCC**: Compiles the hardware drivers (`drivers/`) and the memory heap manager (`mm/`) into ELF64 objects.
3.  **Cargo**: Compiles the core kernel Rust code (`kernel/`) into a static library (`libkeira_kernel.a`).
4.  **LD**: Combines the assembly objects, C objects, and the Rust static library using the custom linker script (`arch/x86/linker.ld`) to output the final kernel binary `keira.bin`.

---

## 2. Bootable ISO Generation
To run the kernel in an emulator or on physical hardware, we construct a bootable ISO image:
1.  **Directory Structure**: Prepares a staging folder (`build/isofiles/boot/grub/`).
2.  **GRUB Config**: Writes the boot menu options (`grub.cfg`) configuration.
3.  **Kernel Placement**: Copies `keira.bin` into `isofiles/boot/`.
4.  **USTAR Initrd**: Packages files (e.g. system binaries) into a USTAR tar archive (`initrd.tar`) and places it under `isofiles/boot/`.
5.  **Image Creation**: Invokes `grub-mkrescue` (which utilizes `xorriso`) to generate the bootable ISO file `build/keira-*.iso`.

---

## 3. Makefile Targets
The Makefile provides standard automation targets:
*   `make build`: Compiles all source files and generates the kernel binary and boot ISO.
*   `make run`: Compiles the system and launches the QEMU emulator.
*   `make clean`: Deletes the `build/` directory and intermediate compiler artifacts.
*   `make iso`: Re-runs the GRUB rescue ISO packaging process.

### QEMU Execution Configuration
The `make run` target configures QEMU with the following options:
*   `-cdrom build/keira-*.iso`: Mounts the bootable ISO.
*   `-serial stdio`: Redirects COM1 serial output directly to the terminal console.
*   `-device intel-hda -device hda-duplex`: Emulates a High Definition Audio controller for sound output.
*   `-m 128M`: Limits guest memory capacity to 128 megabytes.

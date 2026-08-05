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

## 3. FAT16 Disk Image
A 32MB FAT16 disk image (`build/disk.img`) is created and populated with the Keira directory hierarchy:
*   `/system/bin/`: 73 native shell command binaries.
*   `/system/drivers/`: Hardware driver descriptor files (`.sys`).
*   `/system/include/`: C header files for userland programs.
*   `/apps/bin/`: Compiled user-space ELF binaries (`user_test.elf`, `gcc.elf`).
*   `/users/`, `/temp/`, `/data/`: User directories, temporary storage, and persistent data.

---

## 4. Makefile Targets
The Makefile provides the following automation targets:

### Build Targets
| Target | Description |
| :--- | :--- |
| `make all` | Compile all sources and generate kernel binary, boot ISO, and FAT16 disk image. |
| `make iso` | Re-run the GRUB rescue ISO packaging process. |
| `make disk` | Create and populate the FAT16 hard disk image. |
| `make initrd` | Build the RAM Disk USTAR archive. |
| `make user` | Build user-space programs (`init`, `gcc`). |
| `make rust` | Build the Rust kernel static library. |

### Emulator Targets
| Target | Description |
| :--- | :--- |
| `make run` | Compile the system and launch the QEMU emulator. |
| `make debug` | Launch QEMU with GDB stub enabled on port `:1234`. |
| `make qemu-net` | Launch QEMU with Intel e1000 NIC emulation for network testing. |

### Diagnostics Targets
| Target | Description |
| :--- | :--- |
| `make help` | Display all available Makefile targets and configurable variables. |
| `make info` | Display kernel version, toolchain versions, and build configuration. |
| `make check` | Verify all required build dependencies are installed. |
| `make size` | Display kernel binary size and ELF section breakdown. |
| `make objdump` | Dump kernel ELF section headers and memory layout. |

### Maintenance Targets
| Target | Description |
| :--- | :--- |
| `make clean` | Delete the `build/` directory and intermediate compiler artifacts. |
| `make format` | Auto-format all Rust and C source code. |
| `make lint` | Run static analysis on C code using `clang-tidy`. |

---

## 5. Configurable Variables
Override default build parameters by passing variables on the command line:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `V=1` | *(off)* | Verbose mode: print raw compiler commands. |
| `COLOR=0` | *(on)* | Disable ANSI colored terminal output. |
| `DISK_SIZE=N` | `32` | FAT16 disk image size in megabytes. |
| `QEMU_MEM=NM` | `128M` | QEMU guest memory allocation. |

### Examples
```bash
# Full build with verbose output
make all V=1

# Build with a 64MB disk and 256MB QEMU RAM
make all DISK_SIZE=64 QEMU_MEM=256M

# Run with network emulation
make qemu-net

# Check all dependencies before first build
make check
```

---

## 6. QEMU Execution Configuration
The `make run` target configures QEMU with the following options:
*   `-cdrom build/keira-*.iso`: Mounts the bootable ISO.
*   `-device ahci,id=ahci0`: Emulates an AHCI SATA controller for disk access.
*   `-drive file=build/disk.img,format=raw`: Attaches the FAT16 disk image.
*   `-serial stdio`: Redirects COM1 serial output directly to the terminal console.
*   `-device intel-hda -device hda-duplex`: Emulates a High Definition Audio controller for sound output.
*   `-m 128M`: Limits guest memory capacity to 128 megabytes (configurable via `QEMU_MEM`).

The `make qemu-net` target adds:
*   `-device e1000,netdev=net0`: Emulates an Intel e1000 network interface card.
*   `-netdev user,id=net0`: Enables QEMU user-mode network stack for outbound connectivity.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Freestanding 64-bit ELF Binary Loader

Documentation for userland ELF loading in [`crates/fs/src/elf/`](../../../crates/fs/src/elf).

## Mechanism
1. Reads and validates ELF magic bytes (`\x7FELF`).
2. Validates 64-bit class (`ELFCLASS64`), little-endian (`ELFDATA2LSB`), and x86_64 machine architecture (`0x3E`).
3. Iterates over Program Headers (`PT_LOAD`) to map binary text and data segments into user virtual address space.
4. Clears BSS segments and maps Ring 3 user stack (`0x7FFFFFE00000`).
5. Returns entry point address for userland execution trampoline.

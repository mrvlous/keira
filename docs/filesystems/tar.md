<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# USTAR TAR Boot Archive Reader

This document details the read-only TAR filesystem parsing and directory querying implemented in Keira Kernel to read the initial ramdisk (initrd).

## 1. Initrd Memory Mapping
The bootloader (GRUB) loads the initial ramdisk archive (initrd) as a physical memory module and passes its start and end addresses in the Multiboot2 information structure.
*   **Initialization**: `tar::init` registers the physical address range (`initrd_start` to `initrd_end`) and maps it into the virtual address space.
*   **Access**: Since the initrd is already in RAM, file reads translate to direct memory copies from the mapped archive range.

---

## 2. USTAR Header Format Parsing
A tar archive is a sequential chain of 512-byte blocks. Each file begins with a 512-byte header block containing metadata, followed by the file's raw content blocks (aligned to 512-byte boundaries).

The driver ([reader.rs](../../kernel/src/fs/tar/reader.rs)) parses the USTAR metadata fields:
1.  **File Name (Bytes 0-99)**: Zero-terminated path string.
2.  **File Size (Bytes 124-135)**: Stored as a 12-byte octal ASCII string (e.g. `"00000000100\0"`). The parser converts this octal value to a standard binary integer.
3.  **Type Flag (Byte 156)**: Indicates the type of entry:
    *   `'0'` or `'\0'`: Regular file.
    *   `'5'`: Directory.
4.  **USTAR Magic (Bytes 257-262)**: Checked against `"ustar"` to confirm the archive complies with the POSIX USTAR standard.

---

## 3. Directory Query APIs
The reader traverses the archive sequentially, block by block, to resolve file requests:
*   `pub fn exists(path: &str) -> bool`: Traverses the headers. Returns true if a header's filename matches the target path.
*   `pub fn cat_file(path: &str)`: Locates the file header, extracts the file contents, and outputs them to the active VGA/serial console.
*   `pub fn read_file_content(path: &str) -> Option<&'static [u8]>`: Locates the file, parses its octal size, and returns a slice of the archive's memory containing the raw file data.
*   `pub fn list_files()`: Loops through all file headers in the archive and prints their names and sizes to the console.

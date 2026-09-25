<!-- SPDX-License-Identifier: GPL-2.0-only -->

# File System Permissions & Access Control

Access control evaluation follows standard POSIX discretionary access control (DAC).

---

## Permission Bitmask

Each VFS inode stores a 16-bit mode word:
* **File Type** (Bits 15--12): Regular file (`0100000`), Directory (`0040000`), Character device (`0020000`), Block device (`0060000`), FIFO (`0010000`).
* **Special Flags** (Bits 11--9): SUID (`04000`), SGID (`02000`), Sticky Bit (`01000`).
* **Owner Permissions** (Bits 8--6): Read (`r`), Write (`w`), Execute (`x`).
* **Group Permissions** (Bits 5--3): Read (`r`), Write (`w`), Execute (`x`).
* **Other Permissions** (Bits 2--0): Read (`r`), Write (`w`), Execute (`x`).

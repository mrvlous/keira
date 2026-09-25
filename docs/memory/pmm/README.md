<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Manager (PMM)

The PMM manages physical RAM frames (4096 bytes each) using an efficient bitmap allocator.

---

## PMM Index

| Document | Description |
| :--- | :--- |
| [`bitmap.md`](bitmap.md) | Frame allocation bitmap, bit manipulation, frame release |
| [`regions.md`](regions.md) | Multiboot memory map parsing, reserved memory, high memory |

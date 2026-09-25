<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Physical Memory Regions & Reservation

During boot, the PMM categorizes physical memory spans based on Multiboot2 memory tags.

---

## Reserved Regions

* Low 1 MiB (`0x00000000..0x00100000`): Preserved for BIOS data, IVT, and SMP AP trampolines.
* Kernel Binary: Physical space occupied by the kernel `.text`, `.rodata`, `.data`, and `.bss` segments.
* Usable RAM: All remaining conventional physical frames available for allocation.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# High-Precision Event Timer (HPET)

The HPET driver (`crates/arch/src/`) provides a sub-nanosecond monotonic clock source for high-resolution timing.

---

## Operating Invariants

* **Base Address**: Discovered via ACPI HPET table (typically `0xFED00000`).
* **Main Counter**: 64-bit continuous free-running counter operating at 10 MHz or higher.
* **POSIX Time Service**: Backs the `clock_gettime(CLOCK_MONOTONIC)` system call.

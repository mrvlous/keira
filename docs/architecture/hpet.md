<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# High Precision Event Timer (HPET) Subsystem

This document details hardware HPET MMIO register mapping, nanosecond timer resolution, and clock source selection in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel utilizes High Precision Event Timers for high-resolution timing (**Syscall 36 `sys_clock_gettime`**).

---

## 2. Register Layout

*   `HPET_GCAP_ID` (`0x000`): General Capabilities and ID Register.
*   `HPET_GEN_CONF` (`0x010`): General Configuration Register.
*   `HPET_MAIN_COUNTER` (`0x0F0`): 64-bit Main Counter Value Register.

---

## 3. Kernel APIs

*   `pub fn init()`: Parses ACPI HPET table and enables main counter.
*   `pub fn get_time_ns() -> u64`: Returns monotonic nanosecond timestamp.

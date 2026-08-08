<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# CMOS Real-Time Clock (RTC) Driver

This document details hardware RTC CMOS port communications and real-time clock parsing in Keira Kernel.

---

## 1. Driver Overview

Keira Kernel queries the CMOS Real-Time Clock ([rtc.c](../../drivers/rtc/rtc.c)) via I/O ports `0x70` and `0x71` (**Syscall 24 `sys_time`**).

---

## 2. CMOS Registers

*   `0x00`: Seconds
*   `0x02`: Minutes
*   `0x04`: Hours
*   `0x07`: Day of Month
*   `0x08`: Month
*   `0x09`: Year

---

## 3. Kernel APIs

*   `pub fn rtc_read_time() -> RtcTime`: Reads BCD time values and converts to UTC timestamp.

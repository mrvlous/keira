<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# PS/2 Mouse Driver

This document details the PS/2 mouse packet decoding, coordinate scaling, and interrupt handling in Keira Kernel.

---

## 1. Driver Overview

Keira Kernel provides a PS/2 mouse driver ([mouse.c](../../drivers/mouse/mouse.c), [mouse.rs](../../kernel/src/io/mouse.rs)) interfacing with the 8042 Keyboard Controller on IRQ 12.

---

## 2. Packet Format

The mouse sends 3-byte data packets per movement or button event:
*   **Byte 0**: Flags (Button status, Sign bits, Overflow bits).
*   **Byte 1**: X movement delta.
*   **Byte 2**: Y movement delta.

---

## 3. Kernel APIs

*   `pub fn mouse_init()`: Sends enabling command `0xF4` to PS/2 mouse device.
*   `pub fn get_mouse_state() -> (i32, i32, u8)`: Returns active (X, Y) cursor position and button status.

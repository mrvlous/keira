<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Serial Communications & 16550 UART Driver

This submodule details the 16550 UART serial driver providing early debug logging and headless COM1 communication in Keira Kernel.

---

## Serial Driver Index

| Driver | Base Port | Baud Rate | Document | Description |
| :--- | :--- | :--- | :--- | :--- |
| **16550 UART** | `0x3F8` (`COM1`) | 115,200 baud (8-N-1) | [`uart16550.md`](uart16550.md) | Standard PC COM1 serial communications port driver |

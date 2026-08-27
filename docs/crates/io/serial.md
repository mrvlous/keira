<!-- SPDX-License-Identifier: GPL-2.0-only -->

# 16550 UART COM1 Serial Port

Documentation for serial communication in [`crates/io/src/serial/`](../../../crates/io/src/serial).

## Configuration
- Port base: `0x3F8` (COM1).
- Baud rate: 115,200 bps (Divisor latch `0x0001`).
- Line Control: 8 data bits, 1 stop bit, no parity (8N1).
- Used for headless kernel debugging, automated test harness outputs, bidirectional serial interactive shell streaming, and serial kernel panic logging.
- Bidirectional non-blocking RX status polling via `has_byte()` and `read_byte()` on UART line status register `COM1 + 5`.

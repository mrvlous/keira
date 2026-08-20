<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Hardware & POSIX Interval Timers

Documentation for timers in [`crates/arch/src/timers/`](../../../crates/arch/src/timers).

## Subsystems

1. **HPET (`hpet.rs`)**:
   - High Precision Event Timer mapped via ACPI memory address (`0xFED00000`).
   - Nanosecond-precision counter for high-resolution timing.

2. **8253 PIT (`pit.rs`)**:
   - Legacy Programmable Interval Timer on I/O ports `0x40`-`0x43`.
   - Frequency configured to 1000 Hz (1ms tick rate) on IRQ 0.

3. **POSIX Timers (`posix.rs`)**:
   - Implements `sys_timer_create` (Syscall 45) and `sys_timer_settime` (Syscall 46).
   - Supports `CLOCK_REALTIME` and `CLOCK_MONOTONIC`.

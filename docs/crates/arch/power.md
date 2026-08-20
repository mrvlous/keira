<!-- SPDX-License-Identifier: GPL-2.0-only -->

# ACPI Power Management & Watchdog

Documentation for power management in [`crates/arch/src/power/acpi.rs`](../../../crates/arch/src/power/acpi.rs).

## Features
- **ACPI S5 Shutdown (`poweroff`)**: Emits sleep commands to QEMU/Bochs ACPI port `0x604` (`0x2000`) and VirtualBox port `0x4004` (`0x3400`).
- **Hardware Reboot (`reboot`)**: Pulses 8042 Keyboard Controller reset line via port `0x64` (`0xFE`) with PCI reset port fallback (`0xCF9`).
- **NMI Watchdog (`pet_watchdog`)**: Resets hardware watchdog timer to detect kernel deadlocks.

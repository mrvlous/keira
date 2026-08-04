# ACPI Hardware Power Management & NMI Watchdog Subsystem

This document details ACPI power state transitions (S0, S3, S5) and NMI hardware watchdog management in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel implements ACPI power management ([power.rs](../../kernel/src/arch/power.rs)) supporting system sleep states and NMI hardware watchdog timer pet operations.

---

## 2. Power States

| State | Mode | Description |
| :---: | :--- | :--- |
| `S0` | Working | System fully operational |
| `S3` | Sleep | Suspend to RAM |
| `S5` | Poweroff | System soft-off shutdown |

---

## 3. Kernel APIs

*   `pub fn set_power_state(state: u8) -> Result<(), &'static str>`: Performs ACPI sleep/shutdown transitions.
*   `pub fn pet_watchdog()`: Resets Non-Maskable Interrupt (NMI) hardware watchdog counters.

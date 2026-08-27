<!-- SPDX-License-Identifier: GPL-2.0-only -->

# USB Host Controller & HID Subsystem

This document specifies Universal Serial Bus (USB) host controllers and Human Interface Device (HID) drivers in Keira Kernel.

---

## Supported Host Interfaces

* **UHCI (Universal Host Controller Interface)**: USB 1.1 full-speed/low-speed frame list schedules.
* **EHCI (Enhanced Host Controller Interface)**: USB 2.0 high-speed asynchronous lists and queue heads.

---

## Core API (`crates/io/src/usb/mod.rs`)

```rust
pub unsafe fn init();
pub fn enumerate_devices();
pub fn poll_hid_events();
```

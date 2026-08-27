<!-- SPDX-License-Identifier: GPL-2.0-only -->

# VBE Linear Framebuffer (LFB) Graphical Display

This document specifies the high-resolution graphical framebuffer driver in Keira Kernel.

---

## Technical Specifications

* **Resolution**: 1024 $\times$ 768 pixels.
* **Color Depth**: 32 bits per pixel (Truecolor ARGB).
* **Double Buffering**: Back buffer memory rendering with V-Sync flip copy.

---

## Core API (`crates/io/src/framebuffer/mod.rs`)

```rust
pub unsafe fn init(phys_addr: usize, width: u32, height: u32, pitch: u32);
pub fn put_pixel(x: u32, y: u32, color: u32);
pub fn draw_rect(x: u32, y: u32, width: u32, height: u32, color: u32);
```

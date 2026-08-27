<!-- SPDX-License-Identifier: GPL-2.0-only -->

# PC Speaker Tone Driver

This document details hardware tone and frequency generation using PIT Channel 2.

---

## Hardware Control

* **PIT Channel 2 Port**: `0x42` (Frequency divisor input).
* **Gate Control Port**: `0x61` (Bits 0 and 1 enable timer gate and speaker output).

---

## Core API (`crates/io/src/sound/speaker.rs`)

```rust
pub unsafe fn play_tone(freq_hz: u32);
pub unsafe fn stop_tone();
pub fn beep(freq_hz: u32, duration_ms: u32);
```

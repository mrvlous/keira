<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Intel High Definition Audio (HDA) Driver

This document specifies the Intel High Definition Audio controller driver in Keira Kernel.

---

## Technical Specifications

* **PCI Class**: `0x040300` (HD Audio Controller).
* **Buffer Descriptor List (BDL)**: Memory-mapped DMA buffer rings for PCM audio samples.
* **Sample Formats**: 44.1 kHz / 48 kHz 16-bit stereo PCM.

---

## Core API (`crates/io/src/sound/hda.rs`)

```rust
pub unsafe fn init(bar0: usize) -> Result<(), &'static str>;
pub unsafe fn play_pcm(samples: &[u8]) -> Result<(), &'static str>;
```

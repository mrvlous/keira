<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Audio Drivers: Intel HDA & PC Speaker

Documentation for sound in [`crates/io/src/sound/`](../../../crates/io/src/sound).

## Features
- **Intel High Definition Audio (`hda.rs`)**: Discovers HDA controller on PCI, sets up DMA buffer descriptor list (BDL), and plays audio streams.
- **PC Speaker (`speaker.rs`)**: Generates square-wave frequencies using 8253 PIT Channel 2 and I/O port `0x61`.

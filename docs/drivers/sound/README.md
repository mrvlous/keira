<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Audio & Sound Hardware Drivers

This directory details audio synthesizer hardware, Intel High Definition Audio (HDA) codecs, and the Programmable Interval Timer (PIT) PC speaker in Keira Kernel.

---

## Sound Subsystem Architecture

```mermaid
graph TD
    SoundRequest["Kernel Audio / Beep API"] --> Dispatch{"Audio Device Selected"}
    Dispatch -->|Intel HDA| HDADriver["Intel HDA Controller (PCI 8086:2668)"]
    Dispatch -->|PC Speaker| PITDriver["8254 PIT Channel 2 (Ports 0x42 / 0x61)"]
    HDADriver --> Codec["High Definition Audio Codec & Stream DMA"]
    PITDriver --> Speaker["Square Wave Frequency Synthesis (Piezo Speaker)"]
```

---

## Sound Driver Index

| Document | Hardware Adapter | Capabilities |
| :--- | :--- | :--- |
| [`hda.md`](hda.md) | Intel High Definition Audio (HDA) | High-definition multi-channel PCM streaming and codec discovery |
| [`pc_speaker.md`](pc_speaker.md) | 8254 PIT Channel 2 PC Speaker | Square-wave tone generation and audible frequency playback |

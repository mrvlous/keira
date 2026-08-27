<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Audio Hardware Drivers

This submodule details audio output device drivers in Keira Kernel.

---

## Audio Drivers Index

| Driver | Hardware Target | Document | Description |
| :--- | :--- | :--- | :--- |
| **PC Speaker** | PIT Channel 2 + Port `0x61` | [`pc_speaker.md`](pc_speaker.md) | Programmable Interval Timer square wave tone generator |
| **Intel HDA** | High Definition Audio Controller | [`hda.md`](hda.md) | Intel HD Audio stream DMA and audio codec output |

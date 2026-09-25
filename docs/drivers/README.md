<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Hardware Device Drivers

The `drivers` domain contains low-level device drivers for block storage, video displays, serial ports, buses, and terminal line disciplines.

---

## Driver Submodules

```mermaid
graph TD
    Drivers["Driver Subsystems"] --> Storage["storage/<br/>AHCI, NVMe, IDE, RAM Disk"]
    Drivers --> Display["display/<br/>VGA 80x25 & VBE Framebuffer"]
    Drivers --> Serial["serial/<br/>16550 UART (COM1)"]
    Drivers --> Bus["bus/<br/>PCI & PCIe Discovery, USB Host"]
    Drivers --> TTY["tty/<br/>Line Discipline & Virtual Terminals"]
```

---

## Submodule Index

| Submodule | Focus Area | Description |
| :--- | :--- | :--- |
| [`storage/`](storage/README.md) | Block Storage | AHCI SATA, NVMe, legacy IDE, and RAM disk drivers |
| [`display/`](display/README.md) | Video Output | VGA text mode console and linear VBE framebuffer |
| [`serial/`](serial/README.md) | Serial Ports | Standard 16550 UART COM1 serial controller |
| [`bus/`](bus/README.md) | System Buses | PCI/PCIe enumeration, USB host controller interface |
| [`tty/`](tty/README.md) | Terminals | Line discipline, canonical/raw mode, virtual terminals |

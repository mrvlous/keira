<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `keira-io` - Hardware Drivers & Input/Output

The `keira-io` crate encapsulates all hardware drivers, console subsystems, serial ports, bus controllers, storage devices, and virtual terminals.

## Submodules

- [`vga.md`](vga.md): 80x25 text mode console driver and boot logging.
- [`framebuffer.md`](framebuffer.md): VBE Linear Framebuffer, font rendering, mouse cursor.
- [`serial.md`](serial.md): 16550 UART COM1 serial port driver.
- [`pci.md`](pci.md): PCI I/O ports & PCIe ECAM bus enumeration.
- [`storage.md`](storage.md): Block device abstraction, IDE, AHCI SATA, NVMe SSD, RAM disk.
- [`usb.md`](usb.md): USB 3.0 xHCI host controller & Mass Storage BOT driver.
- [`sound.md`](sound.md): Intel HDA audio codec & PC speaker driver.
- [`tty.md`](tty.md): Multi-virtual terminal subsystem (`tty1` to `tty4`).

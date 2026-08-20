<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Network Interface Drivers (NIC)

Documentation for network drivers in [`crates/net/src/driver/`](../../../crates/net/src/driver).

## Implemented Drivers
1. **Intel 82540EM Gigabit Ethernet (`e1000.rs`)**:
   - Discovered on PCI (Vendor ID `0x8086`, Device ID `0x100E`).
   - Uses DMA RX and TX circular descriptor rings for high-throughput packet transfers.
2. **Realtek RTL8139 (`rtl8139.rs`)**:
   - 10/100 Mbps PCI Fast Ethernet controller.
3. **VirtIO Network Device (`virtio_net.rs`)**:
   - Paravirtualized network driver for KVM/QEMU guest execution.

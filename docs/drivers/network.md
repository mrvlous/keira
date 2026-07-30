# Intel e1000 PCI Network Controller Driver

This document details the Intel e1000 Gigabit Ethernet network card driver architecture and ICMP network interface handling implemented in Keira Kernel.

## 1. Driver Overview

The network driver ([e1000.c](../../drivers/net/e1000.c) & [e1000.rs](../../kernel/src/net/e1000.rs)) detects and initializes Intel 82540EM (e1000) PCI network interface controllers. It provides physical packet transmission, MAC address parsing, and ICMP Echo (Ping) packet handling.

---

## 2. Hardware Scanning and Initialization

1. **PCI Bus Enumeration**: Scans PCI Vendor ID `0x8086` and Device IDs (`0x100E`, `0x100F`, `0x1004`, `0x10D3`).
2. **BAR0 Region Resolution**: Reads Base Address Register 0 (BAR0) at PCI configuration offset `0x10` to resolve MMIO/IO Base address regions.
3. **PCI Bus Mastering**: Configures bit 2 in the PCI Command Register (offset `0x04`) to grant DMA Bus Mastering permissions to the network card.

---

## 3. Shell Commands (`network` & `download`)

* **`network`**: Displays the active network interface card state (`eth0`), MAC address, NAT IP address (`10.0.2.15`), and packet TX/RX statistics.
* **`network ping <target_ip>`**: Transmits ICMP Echo Request packets to the target IP address (e.g. `8.8.8.8`) and calculates round-trip latency (RTT) in milliseconds.
* **`download <URL> [target_file_path]`**: Fetches network resources over HTTP/IP and saves received payload data stream directly to FAT16 disk storage.

# Intel e1000 PCI Network Controller Driver & DNS/TCP/DHCP Stack

This document details the Intel e1000 Gigabit Ethernet network card driver architecture, TCP state machine engine, DHCP auto-IP client, and UDP Port 53 DNS Resolver implemented in Keira Kernel.

## 1. Driver Overview

The network subsystem ([e1000.c](../../drivers/net/e1000.c), [e1000.rs](../../kernel/src/net/e1000.rs), [tcp.rs](../../kernel/src/net/tcp.rs), [dhcp.rs](../../kernel/src/net/dhcp.rs), and [dns.rs](../../kernel/src/net/dns.rs)) detects and initializes Intel 82540EM (e1000) PCI network interface controllers. It provides physical packet transmission, MAC address parsing, ICMP Echo (Ping) packet handling, a full 3-way handshake TCP state engine, DHCP dynamic IP auto-configuration, and a UDP Port 53 DNS Resolver.

---

## 2. Hardware Scanning and Initialization

1. **PCI Bus Enumeration**: Scans PCI Vendor ID `0x8086` and Device IDs (`0x100E`, `0x100F`, `0x1004`, `0x10D3`).
2. **BAR0 Region Resolution**: Reads Base Address Register 0 (BAR0) at PCI configuration offset `0x10` to resolve MMIO/IO Base address regions.
3. **PCI Bus Mastering**: Configures bit 2 in the PCI Command Register (offset `0x04`) to grant DMA Bus Mastering permissions to the network card.

---

## 3. DNS Resolver & Network Stack Architecture

* **DNS Resolver ([dns.rs](../../kernel/src/net/dns.rs))**: Encodes QNAME questions (`google.com` → `\x06google\x03com\x00`), builds UDP 53 DNS query headers, transmits frames, and parses Type A IPv4 Answer RRs. Includes a **16-slot Dynamic DNS Cache Table** with hit tracking for 0ms domain resolution on repeated queries.
* **Dynamic ARP Cache ([arp.rs](../../kernel/src/net/arp.rs))**: Maintains a 16-slot IP-to-MAC resolution table with broadcast `ARP Request` (who-has) and `ARP Reply` (is-at) parser for physical network interfaces.
* **TCP State Engine ([tcp.rs](../../kernel/src/net/tcp.rs))**: Implements sequence & acknowledgment number tracking, IP pseudo-header checksum calculation, 3-way handshake (`SYN` → `SYN-ACK` → `ACK`), reliable payload streaming, and graceful `FIN` connection teardown.
* **DHCP Client ([dhcp.rs](../../kernel/src/net/dhcp.rs))**: Implements `DHCP Discover` (UDP 67/68 Broadcast), `DHCP Offer`, `DHCP Request`, and `DHCP ACK` protocol parsing to auto-configure local IP address, Subnet Mask, Default Gateway, and DNS Server.
* **Userland Network Socket API**: Exposes POSIX socket primitives via system calls: `sys_socket` (24), `sys_connect` (25), `sys_send` (26), and `sys_recv` (27).

---

## 4. Native TLS 1.3 Cryptographic Engine ([tls.rs](../../kernel/src/net/tls.rs))

The kernel includes a bare-metal, `no_std` TLS 1.3 (RFC 8446) implementation for encrypted HTTPS communication:

*   **SHA-256 (FIPS 180-4)**: Full message digest hashing with HMAC-SHA-256 (RFC 2104) keyed authentication.
*   **AES-128-GCM (NIST SP 800-38D)**: Galois/Counter Mode authenticated encryption with GHASH GF(2^128) multiplication.
*   **Curve25519 (RFC 7748)**: Montgomery ladder X25519 Elliptic Curve Diffie-Hellman key exchange.
*   **TLS 1.3 Handshake**: Client Hello with SNI/supported_versions/key_share extensions, HKDF-Expand-Label key derivation, and encrypted application data transport.
*   **Cipher Suite**: `TLS_AES_128_GCM_SHA256` (0x1301).

---

## 5. Shell Commands (`network`, `download` & `https`)

* **`network`**: Displays the active network interface card state (`eth0`), MAC address, NAT IP address (`10.0.2.15`), and packet TX/RX statistics.
* **`network dhcp`**: Triggers DHCP dynamic IP auto-configuration over `eth0`.
* **`network resolve <domain>`**: Performs UDP 53 DNS lookup and outputs resolved IPv4 address.
* **`network dns-cache`**: Displays the active 16-slot DNS cache table with resolved domains, IPs, and hit counts.
* **`network ping <target_ip>`**: Transmits ICMP Echo Request packets to target IP/domain and calculates round-trip latency (RTT).
* **`download <URL> [target_file_path]`**: Fetches network resources over encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and saves received payload data stream directly to FAT16 disk storage.
* **`https <url|info|sha256>`**: Performs encrypted HTTPS GET request over Native TLS 1.3 Engine (AES-128-GCM, X25519 ECDH).

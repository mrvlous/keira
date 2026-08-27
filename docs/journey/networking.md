<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 5: Bare-Metal TCP/IP Stack & TLS 1.3

This journal entry details the implementation of bare-metal network interface drivers, packet parsing, TCP state machines, and TLS 1.3 cryptography in Keira Kernel.

---

## Engineering Challenges

1. **Hardware Packet Descriptors**: Writing PCI bus master DMA ring buffer drivers for Intel e1000 and Realtek RTL8139 cards without external runtime libraries.
2. **TCP State Machine & Packet Loss**: Implementing stateful connection handshakes, sequence number tracking, and retransmission timers directly in bare-metal Rust.
3. **TLS 1.3 Crypto Complexity**: Writing X25519 key exchange, SHA-256 HKDF key derivation, and AES-128-GCM record layer encryption completely from scratch.

---

## Solutions & Design Choices

* **Layered Modular Architecture**: Decoupled Ethernet, ARP, IPv4, UDP, TCP, and TLS into distinct Rust modules with explicit packet boundaries.
* **Monotonic Ephemeral Ports**: Implemented atomic monotonic port allocation (`49152`–`65000`) preventing port collision under high-frequency sequential connections.

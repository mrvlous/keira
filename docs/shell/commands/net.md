<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Network & Transport Commands

The `net` command suite provides network interface inspection, socket telemetry, and transport protocol utilities.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `network` | `network` | Query network interface cards, MAC, IP configuration, and sockets | `-h, --help` |
| `download` | `download <url> <dst>` | Download file via HTTP GET request over TCP socket | `-h, --help` |
| `https` | `https <domain>` | Establish secure TLS 1.3 encrypted handshake and query page | `-h, --help` |
| `firewall` | `firewall` | Inspect and configure stateful IPv4 packet filtering rules | `-h, --help` |

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Networking & Socket Shell Commands

This document details all native commands in Keira Kernel related to network interfaces, packet inspection, Internet downloads, and firewall administration.

---

## Command Reference Table

| Command | Syntax | Status | Description |
| :--- | :--- | :--- | :--- |
| `network` | `network [dhcp \| resolve <domain> \| ping <ip>]` | `[Active]` | Display Intel e1000 NIC state, configure DHCP, resolve DNS, or send ICMP ping |
| `download` | `download <url> [dest_path]` | `[Active]` | Fetch network payload over HTTP/HTTPS and save directly to FAT16 storage |
| `https` | `https <url>` | `[Active]` | Securely fetch remote HTTPS payload using native bare-metal TLS 1.3 |
| `firewall` | `firewall [status \| enable \| disable]` | `[Active]` | Display Netfilter packet filter status and drop/accept statistics |
| `iptables` | `iptables [list \| add <rule> \| flush]` | `[Active]` | Inspect and configure Netfilter packet filtering rules |
| `ipcs` | `ipcs [-m] [-s] [-q] [-a]` | `[Preview]` | Query status of System V and POSIX IPC facilities (Syscall 38-40) |
| `ipcrm` | `ipcrm [-m <id>] [-s <id>] [-q <id>]` | `[Preview]` | Remove System V and POSIX IPC facilities from kernel memory (Syscall 41 & 42) |
| `mqueue` | `mqueue [list \| status]` | `[Preview]` | Inspect POSIX Message Queue descriptors interface (Syscall 60 & 61) |

---

## Detailed Usage

### `network` & `network dhcp`
Inspects network interface cards (Intel e1000) and triggers DHCP lease acquisition:
```bash
keira> network
Network Interface eth0:
  Driver        : Intel 82540EM (e1000)
  MAC Address   : 52:54:00:12:34:56
  IPv4 Address  : 10.0.2.15
  Subnet Mask   : 255.255.255.0
  Gateway IP    : 10.0.2.2
  DNS Server    : 10.0.2.3
  Link Status   : Connected (1000 Mbps Full-Duplex)
```

### `download <url>`
Performs DNS resolution and initiates an HTTP/HTTPS stream fetch directly into FAT16 disk storage:
```bash
keira> download http://208.95.112.1/json /data/ip.json
Downloading  128.0 KiB / 128.0 KiB  100% [====================] Finished
Saved to /data/ip.json (128 bytes)
```

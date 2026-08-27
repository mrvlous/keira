<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Networking & Socket Shell Commands

This document details all native commands in Keira Kernel related to network interfaces, packet inspection, Internet downloads, and firewall administration.

---

## Command Reference Table

| Command | Syntax | Description |
| :--- | :--- | :--- |
| `network` | `network [status \| up \| down \| dhcp]` | Display interface status, IP configuration, and trigger DHCP negotiation |
| `download` | `download <url> [output_file]` | Download remote HTTP resource over TCP/IP stack |
| `https` | `https <url>` | Securely fetch remote HTTPS payload using native TLS 1.3 |
| `iptables` | `iptables [list \| add <rule> \| flush]` | Inspect and configure Netfilter packet filtering rules |
| `firewall` | `firewall [status \| enable \| disable]` | Display firewall engine status and drop/accept statistics |
| `ipcs` | `ipcs [all \| pipes \| shm \| queues]` | List active IPC shared memory segments, pipes, and message queues |
| `ipcrm` | `ipcrm <shm \| pipe \| mqueue> <id>` | Destroy and release active IPC channels |
| `mqueue` | `mqueue [list \| create <name> \| send]` | Manage POSIX message queues |

---

## Detailed Usage

### `network status` & `network dhcp`
Inspects network interface cards (Intel e1000 or Realtek RTL8139) and triggers DHCP lease acquisition:
```bash
keira> network status
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
Performs DNS resolution and initiates an HTTP GET request over raw TCP:
```bash
keira> download http://api.github.com/zen /data/zen.txt
Resolving api.github.com... 140.82.121.3
Connecting to 140.82.121.3:80... Connected.
Sending HTTP GET request...
[200 OK] Received 42 bytes.
Saved to /data/zen.txt
```

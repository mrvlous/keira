<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `arp.elf` Freestanding Core Utility

The `arp.elf` executable (`/bin/arp.elf` and `/bin/arp`) is a freestanding Ring 3 core network utility that queries and displays the operating system Address Resolution Protocol (ARP) neighbor cache.

---

## 1. Architectural Purpose

ARP neighbor table inspection in Keira adheres to strict separation of concerns and standard virtual filesystem telemetry:

1. **ProcFS Telemetry Interface**: The utility accesses `/proc/net/arp`, which reflects active hardware-to-network protocol address bindings.
2. **Tabular Formatting**: Displays IP addresses, hardware link layer types (`ether`), ARP cache flags, resolved MAC hardware addresses, netmasks, and interface names.
3. **Neighbor Cache Telemetry**: Facilitates diagnosing Layer 2 link-layer resolution without kernel-debugging consoles or third-party userspace daemons.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-a` | None | Displays all neighbor cache entries (default) |
| `-n` | None | Displays numerical IP addresses |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display ARP neighbor cache:
keira:/# arp
IP address       HW type     Flags       HW address            Mask     Device
10.0.2.2         0x1         0x2         52:54:00:12:34:56     *        eth0
10.0.2.3         0x1         0x2         52:54:00:12:34:57     *        eth0
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/arp/main.c`
- **Binary Locations**: `/bin/arp.elf`, `/bin/arp`
- **Telemetry Source**: `/proc/net/arp` virtual filesystem node
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<unistd.h>`)

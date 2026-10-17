<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `netstat.elf` Freestanding Core Utility

The `netstat.elf` executable (`/bin/netstat.elf` and `/bin/netstat`) is a freestanding Ring 3 core network utility that queries kernel virtual filesystem telemetry to display active Internet connections, socket states, and statistical summaries.

---

## 1. Architectural Purpose

Socket and connection inspection in the Keira pure kernel strictly decouples unprivileged display logic from Ring 0:

1. **ProcFS Telemetry Aggregation**: Interrogates `/proc/net/tcp`, `/proc/net/udp`, `/proc/net/raw`, `/proc/net/route`, and `/proc/net/sockstat` directly.
2. **Hex-to-Dotted-Quad Resolution**: Converts little-endian 32-bit hex IP addresses and 16-bit hex port numbers into human-readable `IP:PORT` format.
3. **State Decoding**: Maps raw kernel connection states into standardized TCP state descriptors (`ESTABLISHED`, `LISTEN`, `TIME_WAIT`, `CLOSE_WAIT`, etc.).
4. **Zero Daemon Overhead**: Operates without background daemons, socket caches, or foreign dependencies.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-a` | `--all` | Displays all sockets (both listening and non-listening) |
| `-t` | `--tcp` | Displays only TCP sockets |
| `-u` | `--udp` | Displays only UDP sockets |
| `-w` | `--raw` | Displays only RAW sockets |
| `-l` | `--listening` | Displays only listening sockets |
| `-r` | `--route` | Displays kernel IPv4 routing table |
| `-s` | `--statistics` | Displays socket allocation statistics |
| `-n` | `--numeric` | Displays numeric addresses and ports without hostname resolution (default) |
| `-tuln` | None | Combined shortcut: listening TCP and UDP sockets |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display active network connections:
keira:/# netstat
Active Internet connections (servers and established)
Proto Recv-Q Send-Q Local Address           Foreign Address         State
tcp        0      0 127.0.0.1:8080          127.0.0.1:80            ESTABLISHED
udp        0      0 0.0.0.0:68              0.0.0.0:*
udp        0      0 127.0.0.1:53            0.0.0.0:*
raw        0      0 0.0.0.0:1               0.0.0.0:*

# Display only listening sockets:
keira:/# netstat -tuln
Active Internet connections (servers and established)
Proto Recv-Q Send-Q Local Address           Foreign Address         State
udp        0      0 0.0.0.0:68              0.0.0.0:*
udp        0      0 127.0.0.1:53            0.0.0.0:*

# Display socket allocation statistics:
keira:/# netstat -s
sockets: used 12
TCP: inuse 1 orphan 0 tw 0 alloc 1 mem 1
UDP: inuse 2 mem 1
RAW: inuse 1
FRAG: inuse 0 memory 0
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/netstat/main.c`
- **Binary Locations**: `/bin/netstat.elf`, `/bin/netstat`
- **Telemetry Sources**: `/proc/net/tcp`, `/proc/net/udp`, `/proc/net/raw`, `/proc/net/sockstat`, `/proc/net/route`
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<fcntl.h>`, `<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<unistd.h>`)

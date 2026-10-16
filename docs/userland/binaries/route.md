<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `route.elf` Freestanding Core Utility

The `route.elf` executable (`/bin/route.elf` and `/bin/route`) is a freestanding Ring 3 core network routing utility that inspects the in-kernel Forwarding Information Base (FIB) and displays IPv4 routing table entries.

---

## 1. Architectural Purpose

Routing table telemetry in the Keira pure kernel follows UNIX/Linux architectural principles:

1. **ProcFS Telemetry Interface**: The utility consumes `/proc/net/route`, reading kernel-synthesized route entries without requiring kernel-level syscall side-channels or daemon dependencies.
2. **Numeric Dotted-Quad Formatting**: Decodes 8-digit hexadecimal IPv4 representations from kernel little-endian format into standard human-readable dotted-quad decimal notation.
3. **Route Flag Decoding**: Maps kernel route flags to standard representations:
   - `U`: Route is active and up (`RTF_UP`, `0x0001`).
   - `G`: Route uses a next-hop gateway (`RTF_GATEWAY`, `0x0002`).
   - `H`: Target is a specific host (`RTF_HOST`, `0x0004`).

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-n` | None | Displays numerical addresses instead of trying to resolve names (default) |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display IPv4 kernel routing table:
keira:/# route
Kernel IP routing table
Destination     Gateway         Genmask         Flags Metric Ref    Use Iface
0.0.0.0         10.0.2.2        0.0.0.0         UG    100    0        0 eth0
10.0.2.0        0.0.0.0         255.255.255.0   U     0      0        0 eth0
127.0.0.0       0.0.0.0         255.0.0.0       U     0      0        0 lo
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/route/main.c`
- **Binary Locations**: `/bin/route.elf`, `/bin/route`
- **Telemetry Source**: `/proc/net/route` virtual filesystem node
- **Privilege Level**: Ring 3 unprivileged
- **Dependencies**: Freestanding `libc.a` (`<stdio.h>`, `<stdlib.h>`, `<string.h>`, `<unistd.h>`)

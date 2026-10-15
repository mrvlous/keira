<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `ifconfig.elf` Freestanding Core Utility

The `ifconfig.elf` executable (`/bin/ifconfig.elf` and `/bin/ifconfig`) is a freestanding Ring 3 core network utility that enumerates network interfaces and displays configuration and packet telemetry.

---

## 1. Architectural Purpose

Network interface telemetry and configuration inspection is decoupled from Ring 0:

1. **Interface Enumeration**: Inspects active network adapters registered in the kernel.
2. **ProcFS Telemetry**: Parses `/proc/net/dev` to report live receive and transmit packet and byte counters.
3. **Addressing Inspection**: Displays IPv4 addresses, netmasks, broadcast addresses, and hardware MAC addresses.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-a` | None | Displays all network interfaces (default) |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# Display all configured network interfaces:
keira:/# ifconfig
lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536
        inet 127.0.0.1  netmask 255.0.0.0
        loop  txqueuelen 1000  (Local Loopback)
        RX packets 32  bytes 4096 (4.0 KiB)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 32  bytes 4096 (4.0 KiB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
        inet 10.0.2.15  netmask 255.255.255.0  broadcast 10.0.2.255
        ether 52:54:00:12:34:56  txqueuelen 1000  (Ethernet)
        RX packets 512  bytes 131072 (128.0 KiB)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 256  bytes 65536 (64.0 KiB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0
```

---

## 4. Technical Specifications

- **Source Path**: `userland/bin/ifconfig/main.c`
- **Binary Locations**: `/bin/ifconfig.elf`, `/bin/ifconfig`
- **Telemetry Source**: `/proc/net/dev` virtual filesystem node
- **Privilege Level**: Ring 3 unprivileged

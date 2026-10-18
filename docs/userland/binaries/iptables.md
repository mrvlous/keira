<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `iptables.elf` Freestanding Core Utility

The `iptables.elf` executable (`/bin/iptables.elf` and `/bin/iptables`) is a freestanding Ring 3 core network security utility that queries kernel virtual filesystem telemetry and invokes Syscall 76 (`SYS_NETFILTER`) to inspect, append, delete, flush, and monitor in-kernel Netfilter packet filtering rules.

---

## 1. Architectural Purpose

In accordance with Keira's pure freestanding kernel ideology, network packet filtering and connection inspection strictly decouple unprivileged presentation and administrative control from Ring 0:

1. **ProcFS Rule Set Virtualization**: Reads `/proc/net/iptables` (and `/proc/net/ip_tables_rules`) to parse active rules, default chain policies, and match counters in standard canonical format.
2. **Table Existence Validation**: Queries `/proc/net/ip_tables_names` to verify supported Netfilter tables (`filter`).
3. **Connection Tracking Telemetry**: Inspects `/proc/net/stat/nf_conntrack` to monitor stateful connection tracking metrics.
4. **Deterministic Syscall Control**: Dispatches administrative operations (`SYS_NETFILTER`, Vector 76) to dynamically append rules, delete rules by index, flush rule tables, or toggle filtering.
5. **Zero External Dependencies**: Implemented in ISO C99 linked against Keira's freestanding libc with zero background daemons.

---

## 2. Command-Line Options

| Option | Long Option | Description |
| :--- | :--- | :--- |
| `-L [chain]` | `--list` | Lists all active packet filtering rules in chains |
| `-S [chain]` | `--list-rules` | Displays rules in command-line specification format |
| `-F [chain]` | `--flush` | Flushes all rules and connection tracking entries |
| `-A <chain>` | `--append` | Appends a new filtering rule to the specified chain |
| `-D <chain> <n>` | `--delete` | Deletes rule number `n` (1-indexed) from chain |
| `--status` | None | Displays in-kernel packet inspection telemetry and conntrack stats |
| `-p <proto>` | `--proto` | Matches protocol (`tcp`, `udp`, `icmp`, `any`) |
| `-s <ip/mask>` | `--source` | Matches source IP address or CIDR mask |
| `-d <ip/mask>` | `--destination` | Matches destination IP address or CIDR mask |
| `--dport <port>` | None | Matches destination TCP/UDP port number |
| `-j <target>` | `--jump` | Target policy action (`ACCEPT`, `DROP`, `REJECT`) |
| `-t <table>` | `--table` | Selects table (default: `filter`) |
| `-h` | `--help` | Displays usage summary and exits with status 0 |

---

## 3. Usage Examples

```bash
# List active rules in canonical formatted table:
keira:/# iptables -L
Chain INPUT (policy ACCEPT)
target     prot opt source               destination         
ACCEPT     tcp  --  0.0.0.0/0            0.0.0.0/0           tcp dpt:80
ACCEPT     tcp  --  0.0.0.0/0            0.0.0.0/0           tcp dpt:443
ACCEPT     icmp --  0.0.0.0/0            0.0.0.0/0           
DROP       tcp  --  0.0.0.0/0            0.0.0.0/0           tcp dpt:23

# Display rules in specification format:
keira:/# iptables -S
-P INPUT ACCEPT
-P FORWARD ACCEPT
-P OUTPUT ACCEPT
-A INPUT -p TCP -s 0.0.0.0/0 -d 0.0.0.0/0 --dport 80 -j ACCEPT
-A INPUT -p TCP -s 0.0.0.0/0 -d 0.0.0.0/0 --dport 443 -j ACCEPT
-A INPUT -p ICMP -s 0.0.0.0/0 -d 0.0.0.0/0 -j ACCEPT
-A INPUT -p TCP -s 0.0.0.0/0 -d 0.0.0.0/0 --dport 23 -j DROP

# Append a rule to drop inbound traffic on port 8080:
keira:/# iptables -A INPUT -p tcp --dport 8080 -j DROP
iptables: Rule 5 added to INPUT chain.

# Delete a rule by 1-indexed number:
keira:/# iptables -D INPUT 5
iptables: Rule 5 deleted from chain INPUT.

# Display in-kernel telemetry and connection tracking stats:
keira:/# iptables --status
Keira Netfilter Kernel Telemetry:
Stateful IPv4 Netfilter Firewall Status:
Engine State: ENABLED (Active Packet Inspection & Filtering)
  Packets Inspected : 142
  Packets Dropped   : 0

Connection Tracking Statistics (/proc/net/stat/nf_conntrack):
entries  searched     found       new   invalid    ignore    delete delete_list    insert insert_failed      drop early_drop     error search_restart
     16        64        58        16         0         0         0           0        16             0         0          0         0              0

# Flush all active firewall rules and conntrack entries:
keira:/# iptables -F
iptables: Flushed rules in table `filter`.
```

---

## 4. Verification and Security

The `iptables.elf` utility is verified across both `x86_64` and `i686` target architectures through:
- Host unit tests verifying `/proc/net/ip_tables_names`, `/proc/net/stat/nf_conntrack`, and `/proc/net/iptables` synthesis.
- Syscall 76 validation confirming safe pointer bounds checking with `read_user_string`.
- Full live execution in headless QEMU testing packet filtering, rule append, deletion, and flush cycles.

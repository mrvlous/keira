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
| `ipcs` | `ipcs [-m] [-s] [-q] [-a]` | `[Active]` | Query status of System V and POSIX IPC facilities (Syscall 38-40, 75) |
| `ipcrm` | `ipcrm [-m <id>] [-s <id>] [-q <id>]` | `[Active]` | Remove System V and POSIX IPC facilities from kernel memory (Syscall 41-42, 75) |
| `mqueue` | `mqueue <status \| list \| create \| send \| recv \| unlink>` | `[Active]` | Inspect and manage in-kernel POSIX Message Queue descriptors (Syscall 58) |

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

### `firewall`
Inspects, toggles, or flushes the stateful IPv4 Netfilter firewall engine:
```bash
keira> firewall status
Stateful IPv4 Netfilter Firewall Status:
Engine State: ENABLED (Active Packet Inspection & Filtering)
  Packets Inspected : 0
  Packets Dropped   : 0

Active Firewall Chain Rules:
  [Rule 1] Chain INPUT | Proto: TCP | Src: 0.0.0.0/0 -> Dst: 0.0.0.0/0:80 => ACCEPT (Matches: 0)
  [Rule 2] Chain INPUT | Proto: TCP | Src: 0.0.0.0/0 -> Dst: 0.0.0.0/0:443 => ACCEPT (Matches: 0)
  [Rule 3] Chain INPUT | Proto: ICMP | Src: 0.0.0.0/0 -> Dst: 0.0.0.0/0:0 => ACCEPT (Matches: 0)
  [Rule 4] Chain INPUT | Proto: TCP | Src: 0.0.0.0/0 -> Dst: 0.0.0.0/0:23 => DROP (Matches: 0)
```

### `iptables`
Dynamically appends, deletes, or flushes firewall chain rules:
```bash
keira> iptables -A INPUT -p tcp --dport 8080 -j DROP
[IPTABLES] Added rule 5 to INPUT (Port 8080/TCP DROP) [OK]

keira> iptables -D 5
[IPTABLES] Deleted rule 5 from chain [OK]
```

### `ipcs`
Queries all active in-kernel System V and POSIX Inter-Process Communication facilities:
```bash
keira> ipcs -a
------ Shared Memory Segments ------
ID   KEY          BYTES   PHYS_FRAME   ATTACHES  OWNER
0    0x12344321   4096    0x70000000   1         PID 1
1    0x56788765   8192    0x70001000   2         PID 2

------ Semaphore Arrays ------
ID   KEY          VALUE   WAITERS
0    0x10002000   1       0
1    0x30004000   5       0

------ POSIX Message Queues ------
ID   NAME             MSGS   MAX_MSGS  MSG_SIZE
0    /keira_sys_mq    1      8         128 B
```

### `ipcrm`
Explicitly deallocates and removes shared memory, semaphore arrays, or message queues from kernel memory:
```bash
keira> ipcrm -m 0
[OK] Removed Shared Memory segment #0

keira> ipcrm -s 1
[OK] Removed Semaphore array #1

keira> ipcrm -q /keira_sys_mq
[OK] Unlinked POSIX Message Queue '/keira_sys_mq'
```

### `mqueue`
Creates, inspects, enqueues, and dequeues messages from POSIX priority message queues:
```bash
keira> mqueue status
POSIX Message Queue Subsystem Status:
  Subsystem Engine : Active (In-Kernel Priority Queue Engine)
  Active Queues    : 1 / 8 allocated
  Queued Messages  : 1 messages total
  Queue Capacity   : 8 msgs per queue
  Max Message Size : 128 bytes
  Syscall Vector   : Syscall 58 (mq_open)

keira> mqueue create /my_queue
[OK] Created POSIX Message Queue '/my_queue' (MQID #1)

keira> mqueue send /my_queue "Payload Alpha"
[OK] Enqueued 13 bytes into '/my_queue' (Priority: 10)

keira> mqueue recv /my_queue
[OK] Dequeued (Priority 10, 13 bytes): "Payload Alpha"
```

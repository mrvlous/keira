<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ring 3 Userland Binaries

Keira includes standard native Ring 3 executable binaries (`userland/bin/`) compiled to static ELF binaries.

---

## Binary Inventory

| Binary | Source Path | Description |
| :--- | :--- | :--- |
| [`init.elf`](init.md) | `userland/bin/init/` | Canonical userspace init executable (PID 1) |
| [`sh.elf`](sh.md) | `userland/bin/sh/` | Canonical Ring 3 POSIX userspace shell executable |
| [`cat.elf`](cat.md) | `userland/bin/cat/` | Freestanding core file concatenator and stream display utility |
| [`ls.elf`](ls.md) | `userland/bin/ls/` | Freestanding core directory enumeration and listing utility |
| [`fetch.elf`](fetch.md) | `userland/bin/fetch/` | Freestanding HTTP client utility for web resource retrieval |
| [`kcc.elf`](kcc.md) | `userland/bin/kcc/` | Freestanding native Ring 3 C compiler executable |
| [`ps.elf`](ps.md) | `userland/bin/ps/` | Freestanding core process table inspection utility |
| [`kill.elf`](kill.md) | `userland/bin/kill/` | Freestanding POSIX signal delivery and process termination utility |
| [`hostname.elf`](hostname.md) | `userland/bin/hostname/` | Freestanding system network node hostname query and setting utility |
| [`clear.elf`](clear.md) | `userland/bin/clear/` | Freestanding ANSI terminal screen reset and cursor positioning utility |
| [`dmesg.elf`](dmesg.md) | `userland/bin/dmesg/` | Freestanding kernel ring buffer and system log inspection utility |
| [`df.elf`](df.md) | `userland/bin/df/` | Freestanding filesystem mount table and disk space usage utility |
| [`ping.elf`](ping.md) | `userland/bin/ping/` | Freestanding ICMP network reachability and round-trip latency testing utility |
| [`ifconfig.elf`](ifconfig.md) | `userland/bin/ifconfig/` | Freestanding network interface enumeration, addressing and telemetry utility |
| [`route.elf`](route.md) | `userland/bin/route/` | Freestanding network routing table inspection and FIB telemetry utility |
| [`arp.elf`](arp.md) | `userland/bin/arp/` | Freestanding Address Resolution Protocol neighbor cache inspection utility |
| [`netstat.elf`](netstat.md) | `userland/bin/netstat/` | Freestanding network socket statistics and connection inspection utility |
| [`uname.elf`](uname.md) | `userland/bin/uname/` | Freestanding POSIX system identity and kernel version inspection utility |
| [`mkdir.elf`](mkdir.md) | `userland/bin/mkdir/` | Freestanding directory node creation utility with recursive parent support |
| [`touch.elf`](touch.md) | `userland/bin/touch/` | Freestanding file node instantiation and timestamp update utility |
| [`rm.elf`](rm.md) | `userland/bin/rm/` | Freestanding file unlinking and recursive directory removal utility |
| [`sysinfo.elf`](sysinfo.md) | `userland/bin/sysinfo/` | System telemetry, CPU, memory, uptime and hardware inspection utility |
| [`test_abi.elf`](test_abi.md) | `userland/bin/test_abi/` | Kernel ABI and system call compliance test suite |
| [`fuzz_abi.elf`](fuzz_abi.md) | `userland/bin/fuzz_abi/` | Differential and boundary fuzz testing utility for kernel system call vectors |

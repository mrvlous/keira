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
| [`iptables.elf`](iptables.md) | `userland/bin/iptables/` | Freestanding Netfilter packet filtering rule management and telemetry utility |
| [`uname.elf`](uname.md) | `userland/bin/uname/` | Freestanding POSIX system identity and kernel version inspection utility |
| [`whoami.elf`](whoami.md) | `userland/bin/whoami/` | Freestanding effective user identity inspection utility |
| [`id.elf`](id.md) | `userland/bin/id/` | Freestanding real and effective UID/GID identity inspection utility |
| [`su.elf`](su.md) | `userland/bin/su/` | Freestanding switch user and kernel privilege transition utility |
| [`mkdir.elf`](mkdir.md) | `userland/bin/mkdir/` | Freestanding directory node creation utility with recursive parent support |
| [`touch.elf`](touch.md) | `userland/bin/touch/` | Freestanding file node instantiation and timestamp update utility |
| [`rm.elf`](rm.md) | `userland/bin/rm/` | Freestanding file unlinking and recursive directory removal utility |
| [`ln.elf`](ln.md) | `userland/bin/ln/` | Freestanding symbolic link creation and path aliasing utility |
| [`sysinfo.elf`](sysinfo.md) | `userland/bin/sysinfo/` | System telemetry, CPU, memory, uptime and hardware inspection utility |
| [`test_abi.elf`](test_abi.md) | `userland/bin/test_abi/` | Kernel ABI and system call compliance test suite |
| [`fuzz_abi.elf`](fuzz_abi.md) | `userland/bin/fuzz_abi/` | Differential and boundary fuzz testing utility for kernel system call vectors |
| [`test_threads.elf`](test_threads.md) | `userland/bin/test_threads/` | Userland multithreading, mutex synchronization and thread join test suite |
| [`mount.elf`](mount.md) | `userland/bin/mount/` | Filesystem mount point attachment and active mount inspection utility |
| [`umount.elf`](umount.md) | `userland/bin/umount/` | Filesystem mount point detachment and resource reclamation utility |
| [`test_pipe.elf`](test_pipe.md) | `userland/bin/test_pipe/` | Anonymous pipe subsystem, multi-channel concurrency and POSIX stream test suite |
| [`grep.elf`](grep.md) | `userland/bin/grep/` | Freestanding regular pattern searching and stream filtering utility |
| [`wc.elf`](wc.md) | `userland/bin/wc/` | Freestanding word, line, byte and character counting utility |
| [`head.elf`](head.md) | `userland/bin/head/` | Freestanding first lines/bytes stream head truncation utility |
| [`tail.elf`](tail.md) | `userland/bin/tail/` | Freestanding last lines/bytes stream tail extraction utility |
| [`echo.elf`](echo.md) | `userland/bin/echo/` | Freestanding string argument printing and escape sequence interpolation utility |
| [`sleep.elf`](sleep.md) | `userland/bin/sleep/` | Freestanding execution delay and interval suspension utility |
| [`chmod.elf`](chmod.md) | `userland/bin/chmod/` | Freestanding file mode bits and Discretionary Access Control modification utility |
| [`chown.elf`](chown.md) | `userland/bin/chown/` | Freestanding file owner and group identity modification utility |

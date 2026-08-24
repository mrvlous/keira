<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Native Shell Commands Catalog

Keira Kernel provides **74 built-in native shell commands** organized by subsystem category, featuring POSIX/GNU flag parsing (`CliArgs`), structured monochrome table output, and `rustc`/`cargo` styled compiler progress bars:

---

## 1. System & Hardware Telemetry
- `system`: Display kernel build telemetry, target architecture, and uptime.
  - Options: `-v, --version` (detailed build info), `-u, --uptime` (uptime only), `-s, --summary` (one-line summary).
- `cpu`: Query processor architecture and CPUID registers.
  - Options: `-f, --features` (extended CPU instruction flags: SSE, AVX2, AES-NI, VMX, NX, KASLR, FSGSBASE, RDRAND), `-r, --raw` (dump leaf 0 CPUID registers), `-s, --summary`.
- `memory`: Inspect physical frame allocator and kernel heap regions.
  - Options: `-m, --mega` (format in MB), `-k, --kilo` (format in KB), `-b, --bytes` (raw bytes), `-s, --summary` (compact summary).
- `drives`: List registered block devices and mount points.
  - Options: `-d, --detailed` (sector counts, 512B block size, mount state), `-s, --summary` (aggregate capacity).
- `devices`: Scan and list all detected PCI hardware devices and vendor IDs.
- `time`: Real-time clock (RTC) date and time in UTC.
  - Options: `-d, --date` (calendar date only), `-t, --time` (time only).
- `runtime`: Display milliseconds elapsed since boot (executes immediately).
- `disk`, `framebuffer`, `usb`, `perf`, `power`, `unwind`.

---

## 2. Process & Multitasking Control
- `tasks`: List running Ring 0 kernel threads and Ring 3 userland processes.
  - Options: `-a, --all` (all tasks and workers), `-s, --summary` (process count).
- `kill`: Terminate active processes by PID.
  - Options: `-9, --kill` (SIGKILL), `-15, --term` (SIGTERM).
- `jobs`: List background job tasks.
- `fg`, `bg`, `stop`, `wait`, `cgroups`, `seccomp`, `mac`.

---

## 3. Filesystem & Storage Operations
- `list`: List directory contents.
  - Options: `-l, --long` (Linux `ls -l` format with `drwxr-xr-x` permissions, types, exact bytes, and filenames), `-a, --all` (show hidden files and `.` / `..`), `-c, --count` (show total entry count).
- `view`, `create`, `folder`, `delete`, `edit`, `copy`, `move`, `search`, `fileinfo`, `protect`, `wipe`, `sync`, `ext4`, `lvm`, `raid`, `swap`, `initrd`, `ramdisk`.
  - `sync`: Flush filesystem buffers and invalidate dirty cache clusters immediately.
  - `initrd`: Inspect preloaded read-only RAM disk entries. Options: `-c, --count`.

---

## 4. Userland & Binary Execution
- `env`: View or modify active environment variables (`$USER`, `$HOME`, `$PATH`, `$SHELL`, `$TERM`, `$LANG`, `$KERNEL`, `$OSTYPE`, `$HOSTTYPE`).
  - Options: `-l, --list` (list all), `-u, --unset <key>` (unset variable), inline `KEY=VAL` assignment.
- `run`, `script`, `go`, `hostname`, `user`, `login`, `please`, `reset`.

---

## 5. Network & Cryptography
- `network`: Interface state, MAC/IP configuration, and link telemetry.
  - Options: `-s, --stats` (extended Intel e1000 Gigabit NIC statistics), `-a, --arp` (neighbor ARP cache), `-c, --cache` (DNS cache).
  - Subcommands: `dhcp`, `resolve <domain>`, `ping <target_ip>`, `dns-cache`.
- `download`: Stream network resources and binary payloads over HTTPS (Native TLS 1.3 on port 443) or HTTP with `rustc`/`cargo` compiler-styled progress bars and save directly to FAT16 storage.
- `https`: Query TLS 1.3 engine info, run SHA-256 self-test, or perform encrypted GET requests.
- `firewall`: Manage stateful IPv4 netfilter packet filtering rules.
  - Options: `-L, --list` (list rules & conntrack), `-F, --flush` (flush rules), `-t, --toggle` (enable/disable engine).
- `iptables`: Netfilter rule table manipulation (`-L`, `-A`, `-D`, `-F`).
- `bpf`: In-kernel eBPF bytecode execution and packet filters.
- `tpm`: TPM 2.0 security enclave hardware interface.

---

## 6. IPC & Diagnostics
- `ipcs`: POSIX Inter-Process Communication telemetry.
  - Options: `-m, --shm` (shared memory segments), `-s, --sem` (counting semaphores), `-a, --all` (all IPC resources).
- `ipcrm`, `futex`, `eventfd`, `epoll`, `mqueue`, `lkm`, `syslog`, `timer`, `kvm`, `guide`, `help`, `history`, `drivers`, `use`, `write`.
  - `history`: View shell command history. Options: `-n, --limit <count>` (limit view), `-c, --clear` (clear history).

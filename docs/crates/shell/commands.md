<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Native Shell Commands Catalog

Keira Kernel provides **75 built-in native shell commands** organized across 7 functional domain submodules (`crates/shell/src/cmds/`), featuring POSIX/GNU flag parsing (`CliArgs`), structured monochrome table output, and `rustc`/`cargo` styled compiler progress bars:

---

## 1. Filesystem & Storage (`cmds/fs/`)
- `list`: List directory contents (`-l, --long`, `-a, --all`, `-c, --count`).
- `view`: Display text or binary file contents.
- `write`: Write content or append data to a file.
- `create`: Create a new empty regular file.
- `folder`: Create a directory or inspect directory hierarchy.
- `delete`: Delete files or directories (`-r, --recursive`, `-f, --force`).
- `edit` / `nano`: In-terminal interactive modal text editor.
- `copy`: Copy files and directory structures.
- `move`: Move or rename files and directories.
- `fileinfo`: Inspect file metadata, permissions, and sector allocations.
- `drives`: List registered block devices and mount points (`-d, --detailed`, `-s, --summary`).
- `use`: Change active working drive or volume.
- `ramdisk`: Create and inspect high-speed memory-backed block devices.
- `disk`: Inspect raw block devices, geometry, and partition tables.
- `initrd`: Inspect preloaded read-only USTAR RAM disk entries (`-c, --count`).
- `ext4`: Extended filesystem diagnostic inspector and metadata parser.

---

## 2. System & Telemetry (`cmds/sys/`)
- `system`: Display kernel build telemetry, target architecture, and uptime (`-v, --version`, `-u, --uptime`, `-s, --summary`).
- `cpu`: Query processor architecture, leaf 0 CPUID registers, and feature flags (`-f, --features`, `-r, --raw`, `-s, --summary`).
- `memory`: Inspect physical frame allocator and kernel heap regions (`-m, --mega`, `-k, --kilo`, `-b, --bytes`, `-s, --summary`).
- `runtime`: Display milliseconds elapsed since system initialization.
- `time`: Real-time clock (RTC) date and time in UTC (`-d, --date`, `-t, --time`).
- `env`: View or modify active environment variables (`-l, --list`, `-u, --unset <key>`).
- `hostname`: Query or configure the kernel system hostname (`-s, --set <name>`).
- `power` / `poweroff`: Shutdown or power off the machine.
- `reset`: Soft reboot the system via keyboard controller / ACPI reset.
- `sync`: Flush filesystem buffers and invalidate dirty cache clusters immediately.
- `syslog`: Inspect kernel system logs and daemon ring buffer records.
- `service` / `ksvc`: Manage background system daemons (httpd, syslogd, syncd, watchdogd).
- `unwind`: Trigger stack trace unwinding diagnostics.

---

## 3. Process & Task Scheduling (`cmds/proc/`)
- `tasks`: List running Ring 0 kernel threads and Ring 3 userland processes (`-a, --all`, `-s, --summary`).
- `run`: Execute userland ELF binaries or launch native applications (`-b, --bg`).
- `kill`: Terminate active processes by PID (`-9, --kill`, `-15, --term`).
- `jobs`: List background job tasks and worker threads.
- `fg`: Bring background job to foreground execution.
- `bg`: Send job to background worker execution.
- `stop`: Pause or suspend an active task.
- `cgroups`: Control group resource quotas and execution limits.
- `futex`: Fast user-space locking and synchronization telemetry.
- `eventfd`: Event notification file descriptor monitor.
- `perf`: Performance counters and CPU profiling statistics.
- `timer`: High-resolution PIT/APIC timer telemetry.

---

## 4. Networking & IPC (`cmds/net/`)
- `network`: Interface state, MAC/IP configuration, and link telemetry (`-s, --stats`, `-a, --arp`, `-c, --cache`).
- `download`: Stream network resources and binary payloads over HTTPS/HTTP with compiler-styled progress bars.
- `https`: Query TLS 1.3 engine info, run SHA-256 self-test, or perform encrypted GET requests.
- `iptables`: Netfilter rule table manipulation (`-L`, `-A`, `-D`, `-F`).
- `firewall`: Manage stateful IPv4 netfilter packet filtering rules (`-L, --list`, `-F, --flush`, `-t, --toggle`).
- `ipcs`: POSIX Inter-Process Communication telemetry (`-m, --shm`, `-s, --sem`, `-a, --all`).
- `ipcrm`: Remove POSIX IPC shared memory or semaphore resources.
- `mqueue`: POSIX message queue telemetry and channel inspector.

---

## 5. Security & Authentication (`cmds/sec/`)
- `login`: Authenticate and switch active session user (`-u, --user <name>`).
- `user`: Manage user accounts and passwords in `/config/sys/passwd` (`-a, --add`, `-d, --delete`, `-l, --list`).
- `protect`: Configure file permission attributes and access control.
- `tpm`: TPM 2.0 security enclave hardware interface.
- `seccomp`: Secure computing mode syscall filter telemetry.
- `bpf`: In-kernel eBPF bytecode execution and packet filters.
- `mac`: Mandatory Access Control security policy manager.

---

## 6. Hardware & Peripheral Drivers (`cmds/dev/`)
- `devices`: Scan and list all detected PCI hardware devices and vendor IDs.
- `framebuffer`: Inspect VESA/GOP framebuffer resolution, pitch, and color format.
- `usb`: Scan USB controller interfaces and attached human interface devices.
- `nvme`: Query NVM Express storage controller namespaces and SMART telemetry.
- `kvm`: Kernel-based virtual machine hardware virtualization support query.
- `drivers`: Inspect registered kernel driver descriptors (`/system/drivers/`).
- `lkm` / `lsmod`: List and manage dynamically loaded kernel modules.
- `lvm`: Logical Volume Manager partition and volume group inspector.
- `raid`: Software RAID array telemetry and mirror status.
- `swap`: Swap space memory paging configuration.
- `epoll`: I/O event notification facility telemetry.

---

## 7. Utilities & Manuals (`cmds/util/`)
- `guide`: Display interactive system manual and quick reference cards.
- `help`: Quick help summary and syntax guide.
- `history`: View shell command history (`-n, --limit <count>`, `-c, --clear`).
- `search`: Search for text patterns across filesystem hierarchies.
- `go`: Navigate across directories and path bookmarks.
- `script`: Execute batch shell script files (`.sh`).
- `wait`: Pause execution for specified duration.
- `wipe`: Clear the active terminal console screen.

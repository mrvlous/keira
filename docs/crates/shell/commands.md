<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Native Shell Commands Catalog

Keira Kernel provides **74 built-in native shell commands** organized by category:

## 1. System & Hardware Telemetry
- `system`, `cpu`, `memory`, `devices`, `runtime`, `time`, `drives`, `disk`, `framebuffer`, `usb`, `perf`, `power`, `unwind`.

## 2. Process & Multitasking Control
- `tasks`, `kill`, `jobs`, `fg`, `bg`, `stop`, `wait`, `cgroups`, `seccomp`, `mac`.

## 3. Filesystem & Storage Operations
- `list`, `view`, `create`, `folder`, `delete`, `edit`, `copy`, `move`, `search`, `fileinfo`, `protect`, `wipe`, `sync`, `ext4`, `lvm`, `raid`, `swap`, `initrd`, `ramdisk`.

## 4. Userland & Binary Execution
- `run`, `script`, `go`, `env`, `hostname`, `user`, `login`, `please`, `reset`.

## 5. Network & Cryptography
- `network`: Interface status, MAC/IP telemetry, and link state.
- `download`: Stream network resources and executable binaries over HTTPS (Native TLS 1.3 on port 443) or HTTP with continuous packet reassembly, live progress bar, and direct FAT16 disk storage.
- `https`: Query TLS 1.3 engine info, run SHA-256 self-test, or perform encrypted GET requests.
- `firewall`: Manage stateful IPv4 netfilter packet filtering rules.
- `iptables`: Netfilter rule table manipulation.
- `bpf`: In-kernel eBPF bytecode execution and packet filters.
- `tpm`: TPM 2.0 security enclave hardware interface.

## 6. IPC & Diagnostics
- `ipcs`, `ipcrm`, `futex`, `eventfd`, `epoll`, `mqueue`, `lkm`, `syslog`, `timer`, `kvm`, `guide`, `help`, `history`, `drivers`, `use`, `write`.

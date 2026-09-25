<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ring 3 Userland Binaries

Keira includes standard native Ring 3 executable binaries (`userland/bin/`) compiled to static ELF binaries.

---

## Binary Inventory

| Binary | Source Path | Description |
| :--- | :--- | :--- |
| [`kcc.elf`](kcc.md) | `userland/bin/kcc/` | Native in-kernel C compiler executable |
| [`sysinfo.elf`](sysinfo.md) | `userland/bin/sysinfo/` | System telemetry, CPU, memory, uptime, and hardware inspection utility |
| [`test_abi.elf`](test_abi.md) | `userland/bin/test_abi/` | Kernel ABI and system call compliance test suite |
| [`fuzz_abi.elf`](fuzz_abi.md) | `userland/bin/fuzz_abi/` | Differential and boundary fuzz testing utility for kernel system call vectors |

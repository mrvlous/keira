<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process & Task Control Commands

The `proc` command suite provides direct binary launching from the emergency console.

> [!NOTE]
> **Pure Kernel Demarcation**: High-level C compilation is handled strictly in Ring 3 userspace via the freestanding binary [`/bin/kcc.elf`](../../userland/binaries/kcc.md). Standard process inspection (`ps`) and signal delivery (`kill`) are handled via freestanding Ring 3 utilities [`/bin/ps.elf`](../../userland/binaries/ps.md) and [`/bin/kill.elf`](../../userland/binaries/kill.md). The Ring 0 supervisor console provides emergency binary launching (`run`).

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `run` | `run <binary>` | Execute an ELF binary from storage or launch background worker | `-h, --help` |

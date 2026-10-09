<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `init.elf` Canonical Userspace Init Binary (PID 1)

The canonical Ring 3 init executable (`/bin/init.elf` and `/bin/init`) is the first userland process spawned by the Keira kernel upon completing hardware initialization and mounting the root Virtual Filesystem (VFS).

---

## 1. Architectural Purpose

Keira is designed as a freestanding monolithic kernel. Under this architecture, the kernel isolates hardware abstraction, memory paging, task scheduling and interrupt handling inside Ring 0, delegating early userspace coordination to Ring 3 PID 1:

1. **Ring 3 Entry Verification**: Validates that privilege level transitions (via `sysret`, `sysexit` or `iret`) succeed cleanly from Ring 0 to Ring 3.
2. **Standard POSIX Execution**: Runs as a standard static ELF executable compiled with `<stdio.h>` and `<unistd.h>` from Keira's freestanding libc.
3. **Graceful Handover**: Completes userspace diagnostics and exits cleanly with status 0, allowing the kernel supervisor loop to drop into the interactive console (`keira:/# `).

---

## 2. Invocation & Shell Integration

While automatically executed at boot time as PID 1, `init.elf` can also be inspected and re-executed dynamically:

```bash
# Execute via the built-in shell init wrapper:
keira:/# init --help
Usage: init [run|status]
Controls or executes canonical userspace init (PID 1).

keira:/# init
[INIT] Launching canonical userspace init from /bin/init.elf...
[init] Starting Keira Canonical Userspace Init (PID 1)...
[init] System initialization sequence completed successfully.
[init] Transferring control to supervisor console.
[INIT] Userspace init completed successfully.

# Execute directly via the ELF runner:
keira:/# run /bin/init.elf
Loading ELF binary: /bin/init.elf
[init] Starting Keira Canonical Userspace Init (PID 1)...
[init] System initialization sequence completed successfully.
[init] Transferring control to supervisor console.
Program exited normally.
```

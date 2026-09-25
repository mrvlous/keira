<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Initialization & PID 1

Describes the transition from kernel boot to userland initialization.

---

## Initialization Sequence

1. The kernel finishes driver bringup, mounts the root filesystem (`/`), and initializes the process table.
2. The kernel spawns the first userland process (PID 1) executing `/system/bin/init` or the interactive shell `/system/bin/sh`.
3. PID 1 configures the system hostname, runs startup scripts from `/etc/rc.d`, mounts virtual filesystems (`/proc`, `/dev`), and launches login terminals.
4. PID 1 adopts orphaned processes and reaps terminated zombie tasks via `waitpid()`.

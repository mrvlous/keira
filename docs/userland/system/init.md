<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Initialization & PID 1

Describes the transition from kernel boot to userland initialization.

---

## Initialization Sequence

1. The kernel finishes driver bringup, mounts the root filesystem (`/`) and initializes the process table.
2. The kernel spawns the first userland process (PID 1) executing `/bin/init` or the interactive shell `/bin/sh`.
3. PID 1 or the kernel monitor mounts virtual filesystems (`/proc`, `/dev`), initializes primary storage volumes and loads core configurations from `/etc/`.
4. The system provides the direct Ring 0 kernel control plane or executes userland worker sandboxes.
5. Orphaned processes are re-parented and reaped via `waitpid()`.

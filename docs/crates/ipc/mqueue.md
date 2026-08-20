<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX Message Queue Subsystem

Documentation for message queues in [`crates/ipc/src/mqueue/`](../../../crates/ipc/src/mqueue).

## System Call (`sys_mq_open` - Syscall 55)
- Priority-ordered message delivery between processes.
- Fixed-capacity kernel queue buffers without heap allocation.

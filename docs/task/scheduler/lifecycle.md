<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Task Lifecycle & Process Management

1. **Creation**: New tasks are spawned via `fork()` or `sys_exec()`.
2. **Execution**: Scheduled according to state and available CPU time slice.
3. **Termination**: Upon `sys_exit()`, memory and file descriptors are reclaimed, and the task becomes a `Zombie` until reaped by `waitpid()`.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Process Control Blocks & Task States

Documentation for task structures in [`crates/task/src/types.rs`](../../../crates/task/src/types.rs).

## Task States
- `TaskState::Ready`: Task is ready to be scheduled on CPU.
- `TaskState::Running`: Task is currently executing on active core.
- `TaskState::Blocked`: Task is waiting on I/O, timer, or lock.
- `TaskState::Terminated`: Task has exited; resources pending reclamation.

## Task Struct Fields
- `pid`: Process identifier (0 = kernel idle task).
- `name`: ASCII process name.
- `pml4_phys`: Physical PML4 CR3 register address.
- `rsp`: Saved stack pointer during context switches.
- `program_break`: User heap allocation break pointer.
- `fds`: Static 8-slot open file descriptor table.

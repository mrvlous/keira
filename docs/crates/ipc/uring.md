<!-- SPDX-License-Identifier: GPL-2.0-only -->

# `io_uring` Asynchronous Ring Queues

Documentation for high-performance I/O in [`crates/ipc/src/uring/`](../../../crates/ipc/src/uring).

## Architecture
- Shared-memory ring buffers between user space and kernel:
  - **Submission Queue (SQ)**: Userland submits I/O requests (`SubmissionQueueEntry`).
  - **Completion Queue (CQ)**: Kernel posts completion events (`CompletionQueueEntry`).
- Eliminates system call invocation overhead for high-frequency disk and network I/O.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Anonymous Pipes & Dynamic Multi-Channel FIFO Subsystem

Anonymous pipes provide unidirectional byte-stream inter-process communication between related processes across parent and child task boundaries.

---

## 1. Multi-Channel Ring Buffer Architecture (`crates/ipc/src/pipe/fifo/`)

Each pipe is managed inside `PIPE_TABLE` (supporting up to 32 concurrent channels) and backed by a 4096-byte kernel ring buffer:

```rust
pub struct PipeChannel {
    pub id: u32,
    pub in_use: bool,
    pub data: [u8; 4096],
    pub head: usize,
    pub tail: usize,
    pub count: usize,
    pub readers_count: u32,
    pub writers_count: u32,
    pub flags: u32,
}
```

---

## 2. Synchronization & POSIX Semantics

* **Streaming Read**: If `count > 0`, reads up to `count` bytes from the circular buffer and advances `head`.
* **EOF Condition**: If `count == 0` and `writers_count == 0`, `read()` returns `0` immediately (POSIX EOF).
* **Non-Blocking Read**: If `count == 0`, `writers_count > 0` and `flags & O_NONBLOCK != 0`, `read()` returns `-EAGAIN`.
* **Streaming Write**: If space remains (`count < 4096`), writes data into the buffer and advances `tail`.
* **Broken Pipe (`EPIPE`)**: If `readers_count == 0`, `write()` delivers `SIGPIPE` (signal 13) to the caller and returns `-EPIPE`.
* **Non-Blocking Write**: If `count == 4096` and `flags & O_NONBLOCK != 0`, `write()` returns `-EAGAIN`.

---

## 3. System Call Interfaces

* **`SYS_PIPE` (Vector 23)**: Standard `pipe(int pipefd[2])` allocating read/write descriptors.
* **`SYS_PIPE2` (Vector 96)**: Extended `pipe2(int pipefd[2], int flags)` supporting `O_CLOEXEC` and `O_NONBLOCK`.

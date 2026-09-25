<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Anonymous Pipes & Ring Buffer Synchronization

Anonymous pipes provide unidirectional byte-stream inter-process communication between related processes.

---

## 1. Circular Buffer Implementation (`crates/ipc/src/pipe/`)

Each pipe is backed by a 4096-byte kernel ring buffer:

```rust
pub struct PipeBuffer {
    buffer: [u8; 4096],
    read_pos: usize,
    write_pos: usize,
    count: usize,
    readers_open: usize,
    writers_open: usize,
    read_waiters: WaitQueue,
    write_waiters: WaitQueue,
}
```

---

## 2. Synchronization & POSIX Semantics

* **Blocking Read**: If `count == 0` and `writers_open > 0`, the reading task blocks on `read_waiters`.
* **EOF Condition**: If `count == 0` and `writers_open == 0`, `read()` returns `0` immediately.
* **Blocking Write**: If `count == 4096` and `readers_open > 0`, the writing task blocks on `write_waiters`.
* **Broken Pipe (`EPIPE`)**: If `readers_open == 0`, `write()` delivers `SIGPIPE` to the caller and returns `-EPIPE`.

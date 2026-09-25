<!-- SPDX-License-Identifier: GPL-2.0-only -->

# POSIX I/O Architecture & Stream Semantics

Covers the stream abstraction connecting userland processes to kernel filesystems and character devices.

---

## File Descriptor Tables

Each process maintains an array of open file description pointers:
* Index 0: `stdin` (Standard Input)
* Index 1: `stdout` (Standard Output)
* Index 2: `stderr` (Standard Error)

---

## Non-Blocking & Asynchronous Modes

* `O_NONBLOCK`: `read` and `write` return `EAGAIN` or `EWOULDBLOCK` instead of suspending the task when data is unavailable.
* File status flags can be manipulated at runtime via `fcntl(fd, F_SETFL, flags)`.

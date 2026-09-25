<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Fast Userspace Mutex (Futex) Engine

Futexes provide zero-syscall userland lock acquisition in uncontended cases, invoking the kernel only when thread contention requires sleep or wakeup.

---

## 1. Futex Syscall ABI

```c
int futex(int *uaddr, int op, int val, const struct timespec *timeout);
```

* `FUTEX_WAIT`: If `*uaddr == val`, the kernel puts the calling task to sleep until awakened.
* `FUTEX_WAKE`: Wakes up to `val` tasks blocked on `uaddr`.

---

## 2. Kernel Wait Queue Hash Table

In `crates/ipc/src/futex/`:
* The kernel maintains a global hash table of wait queues keyed by `(Task.AddressSpace, uaddr_phys)`.
* Memory comparisons are atomic with respect to task descheduling, preventing sleep/wakeup lost update race conditions.

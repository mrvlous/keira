# Fast Userspace Mutex (Futex), POSIX Threading, & Resource Control Groups (cgroups)

This document details the Fast Userspace Mutex (`sys_futex`) locking mechanism, POSIX thread creation (`sys_clone_thread`), and Resource Control Groups (`cgroups`) PID namespace isolation in Keira Kernel.

---

## 1. Fast Userspace Mutex (Futex) & POSIX Threading

Keira Kernel provides high-performance userland thread synchronization ([futex.rs](../../kernel/src/syscall/futex.rs)):

*   **Atomic Wait Queues (`FUTEX_WAIT`)**: Userland processes perform atomic checks on lock variables. If a lock is contended, the task registers a `FutexWaitSlot` and suspends execution without spin-locking CPU cycles.
*   **Waking Thread Notifications (`FUTEX_WAKE`)**: Unlocking threads notify the kernel to wake up waiting tasks queued on the target atomic address.

### System Call Interface

```c
// Syscall 40: Fast Userspace Mutex operation (FUTEX_WAIT, FUTEX_WAKE)
long sys_futex(uint64_t uaddr, uint32_t op, uint32_t val, uint32_t val2);

// Syscall 41: Clone execution context for POSIX userland thread
long sys_clone_thread(uint64_t fn_ptr, uint64_t stack_ptr, uint64_t flags);
```

---

## 2. Resource Control Groups (cgroups) & PID Namespaces

Implemented in [cgroup.rs](../../kernel/src/task/cgroup.rs):

*   **Memory Quota Enforcement**: Tracks memory consumption per process control group and enforces limits (`max_memory_bytes = 64MB`) to prevent memory exhaustion by rogue tasks.
*   **Isolated PID Namespaces**: Translates host Process IDs (PIDs) to isolated container namespace PIDs, enabling containerized process isolation.

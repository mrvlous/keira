<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Journey Milestone 15: Userland Multi-Threading & v0.6.0 Release

Milestone 15 crowns the Keira Kernel `v0.6.0` production release. It introduces the full userland multi-threading architecture through the `clone` system call (`SYS_CLONE_THREAD`), support for shared virtual address spaces (`CLONE_VM`), thread groups (`CLONE_THREAD`), Thread Local Storage (`CLONE_SETTLS`), user TID lifecycle management (`CLONE_PARENT_SETTID` and `CLONE_CHILD_CLEARTID`) and an autonomous freestanding POSIX Threads (`pthread`) library.

---

## 1. Architectural Motivation

Prior to Milestone 15, Keira supported process isolation through Copy-on-Write (COW) forking (`sys_fork`) and ELF binary execution (`sys_exec`). However, modern concurrent software architectures require multiple threads of execution within the *same* virtual address space to cooperatively process parallel data streams with shared heap and synchronization primitives without incurring page-table isolation overhead.

```mermaid
graph TD
    Parent["Process Leader (PID 1, TGID 1)"] -->|clone(CLONE_VM)| Thread1["Worker Thread 1 (TID 2, TGID 1)"]
    Parent -->|clone(CLONE_VM)| Thread2["Worker Thread 2 (TID 3, TGID 1)"]
    Parent -->|clone(CLONE_VM)| Thread3["Worker Thread 3 (TID 4, TGID 1)"]
    Thread1 -.->|Shared CR3 / PML4| VMM["Unified Virtual Memory Space"]
    Thread2 -.->|Shared CR3 / PML4| VMM
    Thread3 -.->|Shared CR3 / PML4| VMM
    Thread1 <-->|Atomic CAS & Futex| Mutex["Shared pthread_mutex_t"]
    Thread2 <-->|Atomic CAS & Futex| Mutex
    Thread3 <-->|Atomic CAS & Futex| Mutex
```

---

## 2. Kernel `clone` Syscall Engine

Syscall vector 41 (`SYS_CLONE_THREAD`) provides a fine-grained cloning interface modeled after the POSIX Linux `clone(2)` specification.

### A. Clone Flags Matrix

| Flag | Numeric Value | Functional Semantics |
| :--- | :--- | :--- |
| `CLONE_VM` | `0x00000100` | Child shares parent's physical PML4 root directly without copying page tables or marking pages COW. |
| `CLONE_FS` | `0x00000200` | Child shares filesystem context (current working directory). |
| `CLONE_FILES` | `0x00000400` | Child shares the active file descriptor table. |
| `CLONE_SIGHAND` | `0x00000800` | Child shares signal action handlers. |
| `CLONE_THREAD` | `0x00010000` | Child inherits parent's Thread Group ID (`tgid`) and marks `is_thread = true`. |
| `CLONE_SETTLS` | `0x00080000` | Kernel configures child thread's `tls` pointer and `IA32_FS_BASE_MSR`. |
| `CLONE_PARENT_SETTID` | `0x00100000` | Kernel writes the newly assigned child thread ID (TID) to the user-space `ptid` pointer. |
| `CLONE_CHILD_CLEARTID` | `0x00200000` | Kernel registers `ctid` user address to automatically zero and wake futex waiters upon thread termination. |
| `CLONE_CHILD_SETTID` | `0x01000000` | Kernel writes child TID to `ctid` user memory in the child's address space. |

### B. Task Control Block (TCB) Enhancements

The kernel's `Task` descriptor in [`crates/task/src/types/task/descriptor.rs`](../../crates/task/src/types/task/descriptor.rs) tracks thread group metadata:

```rust
pub struct Task {
    pub id: usize,
    pub name: &'static str,
    pub rsp: u64,
    pub stack_addr: u64,
    pub state: TaskState,
    pub fds: [FileDescriptor; MAX_FDS],
    pub program_break: u64,
    pub program_break_start: u64,
    pub cwd: [u8; 128],
    pub cwd_len: usize,
    pub parent_id: usize,
    pub pml4_phys: u64,
    pub exit_code: i32,
    pub is_user: bool,
    pub uid: u32,
    pub gid: u32,
    pub euid: u32,
    pub egid: u32,
    pub saved_sigcontext: Option<InterruptContext>,
    pub signal_mask: u32,
    pub pending_signals: u32,
    pub is_orphan: bool,
    pub tgid: usize,
    pub is_thread: bool,
    pub clear_child_tid: u64,
    pub tls: u64,
    pub cpu_ticks: u64,
    pub switches: u64,
}
```

### C. Reference-Counted Address Space Reclamation

When multiple threads execute within the same address space (`pml4_phys`), destroying a thread must not deallocate page tables still referenced by living sibling threads or the thread group leader. In [`crates/task/src/scheduler/lifecycle/wait.rs`](../../crates/task/src/scheduler/lifecycle/wait.rs) and [`reap.rs`](../../crates/task/src/scheduler/lifecycle/reap.rs), the scheduler performs reference checks across all active slots before releasing user pages:

```rust
let is_last_pml4_user = !TASKS
    .iter()
    .take(MAX_TASKS)
    .flatten()
    .any(|t| t.id != id && t.pml4_phys == child.pml4_phys);

if child.stack_addr != 0 {
    if is_last_pml4_user {
        vmm::free_user_pages(child.pml4_phys, child.program_break);
    }
    pmm::free_frame(child.stack_addr);
} else if child.pml4_phys != 0 && is_last_pml4_user {
    vmm::cleanup_vmas_for_pml4(child.pml4_phys);
}
```

---

## 3. Freestanding POSIX Pthreads Library

The userland runtime provides a POSIX-compliant multi-threading library integrated into `libc.a`.

### A. Thread Spawning & Stack Trampoline

```c
int pthread_create(pthread_t *thread, const pthread_attr_t *attr,
                   void *(*start_routine)(void *), void *arg);
```

When creating a thread:
1. A private stack (default 64 KiB) is dynamically allocated via `malloc()`.
2. A trampoline stub prepares the stack frame, aligning to 16 bytes for System V AMD64 ABI compliance.
3. The `clone()` assembly wrapper invokes `SYS_CLONE_THREAD` passing `CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID`.
4. In the child context, execution begins at the target routine and cleanly terminates via `sys_exit()`.

### B. Fast Userspace Mutex (`pthread_mutex_t`)

Thread synchronization implements a two-tier locking strategy:
1. **Fast Uncontended Path**: Atomic Compare-and-Swap (`__atomic_compare_exchange_n`) acquires the mutex in user space with zero kernel transitions.
2. **Contended Path**: When contention occurs, the thread enqueues onto the kernel futex wait queue (`SYS_FUTEX` with `FUTEX_WAIT`). Upon unlock, `FUTEX_WAKE` releases sleeping threads.

### C. Thread Termination & Futex Join Synchronization

When `pthread_join(thread, &retval)` is invoked:
1. The joining thread queries the thread's completion status.
2. If active, the caller sleeps on the child's `clear_child_tid` memory address via `FUTEX_WAIT`.
3. When the child thread terminates via `handle_exit`, the kernel clears the user pointer (`*clear_child_tid = 0`) and triggers `futex_wake(clear_child_tid, 1)`.
4. The caller awakens, retrieves the returned exit pointer and reclaims allocated stack frames.

---

## 4. Verification & Certification Battery

The multi-threading subsystem was verified through the dedicated userland diagnostic suite [`userland/bin/test_threads/main.c`](../../userland/bin/test_threads/main.c):

```text
Keira Multi-Threading Test Suite

[TEST] Spawning 4 concurrent POSIX threads...
  [OK] Thread 0 spawned successfully
  [OK] Thread 1 spawned successfully
  [OK] Thread 2 spawned successfully
  [OK] Thread 3 spawned successfully
[TEST] Joining worker threads via futex wait...
  [OK] Thread 0 joined with exit code: 1
  [OK] Thread 1 joined with exit code: 2
  [OK] Thread 2 joined with exit code: 3
  [OK] Thread 3 joined with exit code: 4
[TEST] Validating CLONE_VM shared address space mutations...
  [OK] All worker mutations verified in shared address space
[TEST] Validating mutex critical section protection...
  [OK] Counter value = 1000 matches expected 1000

[DONE] Multi-Threading Test Battery Passed.
```

---

## 5. Summary of Deliverables in Keira v0.6.0

With Milestone 15 complete, Keira Kernel `v0.6.0` delivers:
- **Type-Safe Memory Reclamation**: Epoch-Based Reclamation (EBR) and Copy-on-Write address space cloning (Milestone 10).
- **Lock-Free Work-Stealing**: Per-CPU Chase-Lev deques and SMP load balancing (Milestone 11).
- **Hierarchical Magazine Slab Allocator**: Bonwick magazine caching and depot exchange (Milestone 12).
- **SMP Inter-Processor Interrupts**: Cross-core TLB shootdown rendezvous (Milestone 13) and Vector 0xFE preemption IPIs (Milestone 14).
- **Userland Multi-Threading**: Full `clone` syscall with `CLONE_VM`, thread group coordination and freestanding POSIX Pthreads (Milestone 15).

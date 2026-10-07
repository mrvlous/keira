<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Per-CPU Lock-Free Work-Stealing Scheduler

This document details the architecture and operational mechanics of Keira's decentralized per-CPU work-stealing scheduler (`crates/task/src/scheduler/work_stealing/`).

---

## 1. Overview & Design Rationale

Traditional monolithic kernels rely on a centralized runqueue protected by a global scheduler spinlock. Under multicore SMP workloads with frequent preemptive interrupts (1000 Hz PIT/HPET), CPUs suffer severe lock contention and cacheline bouncing.

Keira replaces the global runqueue bottleneck with a decentralized **Chase-Lev work-stealing topology**:
* **Per-CPU Runqueues**: Each processor possesses an independent lock-free `ChaseLevDeque`.
* **Zero Contention Local Execution**: The owning core pushes and pops from the bottom of its deque without locks or atomic CAS.
* **Lock-Free Stealing**: Remote idle cores steal work from the top of victim deques using atomic CAS, protected by Epoch-Based Reclamation (EBR).
* **Decentralized Load Balancing**: Each core autonomously balances work without acquiring central locks.

---

## 2. Component Architecture

### A. Chase-Lev Circular Deque (`deque.rs`)
* **Capacity**: Power-of-two capacity `DEQUE_CAPACITY = 64` mapped by `DEQUE_MASK`.
* **Pointers**: `bottom: AtomicUsize` and `top: AtomicUsize`.
* **Operations**:
  - `push(task_idx)`: Owner stores at `buffer[b & DEQUE_MASK]`, then stores `b + 1` with `Ordering::Release`.
  - `pop()`: Owner decrements `bottom`, issues `Ordering::SeqCst` memory fence, and checks remaining slots. Resolves single-element races against remote thieves via atomic CAS on `top`.
  - `steal()`: Remote thieves pin the active epoch via `keira_core::sync::ebr::pin()` and perform `compare_exchange` on `top` (`Ordering::SeqCst`).

### B. Per-CPU State & Deduplication (`percpu.rs`)
* **Runqueue Mapping**: Static array `CPU_RUNQUEUES[MAX_CPU_CORES]` across up to 16 SMP cores.
* **Deduplication Bitset**: `TASK_QUEUED: [AtomicBool; MAX_TASKS]`. Prevents duplicate queue entries when tasks are scheduled or woken.
* **Core APIs**:
  - `enqueue_task(core_id, task_idx)`: Atomic CAS on `TASK_QUEUED[task_idx]`, then pushes to target core's deque.
  - `pop_local(core_id)`: Pops from local runqueue and clears task queued bit.
  - `steal_from(victim_core)`: Steals from remote runqueue and clears task queued bit.

### C. Decentralized Task Balancer (`balancer.rs`)
The function `pick_next_task(core_id, current_task)` selects the next runnable task:
1. **Local Runqueue**: Pops ready tasks from the core's local Chase-Lev runqueue.
2. **Work Stealing**: Iteratively inspects victim cores `(core_id + offset) % MAX_CPU_CORES` and steals ready tasks.
3. **Fallback Scan**: Graceful linear fallback scanning for unqueued ready tasks, strictly excluding the current yielding task.

---

## 3. Preemptive Tick Dispatch & Task 0 Pinning

In `crates/task/src/scheduler/dispatch/tick.rs`:
* **Task 0 (Bootstrap / Shell)**: Pinned strictly to Core 0 (BSP) to protect the bootstrap stack.
* **Fair Preemptive Alternation**: When a userland worker task yields or exhausts its time quantum on Core 0, the scheduler alternates execution with Task 0 if Task 0 is ready. The outgoing worker task is placed back onto the local runqueue (`push_local`), ensuring fair time-slicing between user processes and the shell.

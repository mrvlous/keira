<!-- SPDX-License-Identifier: GPL-2.0-only -->

# SMP Concurrency & Deadlock Defense Architecture

Keira Kernel implements a deterministic, multi-core concurrency control architecture engineered to eliminate race conditions, circular-wait deadlocks, and interrupt inversion bugs across freestanding kernel subsystems.

---

## Architecture Overview

```mermaid
graph TD
    UserLand["Ring 3 Multi-Process Userland"] --> Syscall["Syscall Interface"]
    Syscall --> VFS["VFS / Descriptor Locks (Rank 4)"]
    VFS --> Scheduler["Scheduler & Task Table Lock (Rank 3)"]
    Scheduler --> Heap["Heap Allocator & Slab Cache Lock (Rank 2)"]
    Heap --> PMM["Physical Memory Manager Lock (Rank 1)"]

    subgraph LockDefense["Deadlock & Concurrency Defense Engine"]
        Hierarchy["Strict Descending Rank Hierarchy"]
        Recursion["CPUID Core Recursion Detection"]
        IrqDisable["Atomic Local IRQ Disabling (CLI/STI)"]
        Watchdog["Soft Lockup Heartbeat Watchdog"]
    end

    VFS -.-> LockDefense
    Scheduler -.-> LockDefense
    Heap -.-> LockDefense
    PMM -.-> LockDefense
```

---

## 1. Interrupt-Safe Spinlocks (`IrqSpinLock`)

Standard spinlocks in operating system kernels are vulnerable to interrupt handler inversion deadlocks: if CPU Core $N$ acquires a spinlock while interrupts are enabled, and a hardware timer interrupt fires on Core $N$ whose ISR attempts to acquire the same lock, Core $N$ will spin indefinitely waiting for itself to release the lock.

Keira solves this through `IrqSpinLock`:
- **Atomic IRQ Disabling**: Saves the local CPU `RFLAGS.IF` interrupt enable flag and executes `cli` before attempting to acquire the lock.
- **Interrupt State Preservation**: Tracks whether interrupts were previously enabled in `IrqState`. Unlocking the lock restores `IF` only if it was enabled prior to lock acquisition, safely allowing arbitrary lock nesting without prematurely re-enabling interrupts.
- **RAII Scoped Guard**: `IrqSpinLock::lock()` yields an `IrqSpinLockGuard` implementing `Drop`, guaranteeing lock release and IRQ restoration even on early returns or error paths.

```rust
pub struct IrqSpinLock {
    locked: AtomicBool,
    saved_irq: AtomicBool,
    saved_rank: AtomicU8,
    holder_core: AtomicI32,
    rank: LockRank,
}
```

---

## 2. Lock Ordering Hierarchy (`LockRank`)

To eliminate Coffman circular wait deadlocks, Keira establishes a formal lock ranking hierarchy. Subsystems must acquire locks strictly in descending order:

$$\text{LockRank::Vfs (4)} \longrightarrow \text{LockRank::Scheduler (3)} \longrightarrow \text{LockRank::Heap (2)} \longrightarrow \text{LockRank::Pmm (1)}$$

| Rank | Identifier | Subsystem / Protected Resource | Acquisition Policy |
| :--- | :--- | :--- | :--- |
| **0** | `LockRank::None` | Unranked leaf locks | No nested lock dependencies |
| **1** | `LockRank::Pmm` | Physical Frame Allocator (`PMM_LOCK`) | Lowest ranked primitive; holds no other locks |
| **2** | `LockRank::Heap` | Kernel Heap (`HEAP_LOCK`) & Slab (`KmemCache`) | May acquire Pmm, but never Scheduler or Vfs |
| **3** | `LockRank::Scheduler` | Multitasking Scheduler (`SCHEDULER_LOCK`) | May acquire Heap or Pmm during task allocation |
| **4** | `LockRank::Vfs` | Virtual File System & Descriptor Locks | May acquire Scheduler, Heap, or Pmm |

Any attempt to acquire a lock whose numerical rank is greater than or equal to the currently held rank triggers a lock order inversion error:

```rust
if current != 0 && target >= current {
    return Err("Lock order inversion: attempted to acquire higher or equal rank while holding lock");
}
```

---

## 3. Core ID Resolution & Recursion Detection

Keira detects intra-core re-entrancy bugs by inspecting hardware core IDs:
- **Zero Dependencies**: Core ID resolution is performed directly via CPUID leaf 1 (`ebx >> 24`) in freestanding mode, allowing `keira-core` to operate without dependencies on architecture or scheduler crates.
- **Recursion Panic**: If Core $N$ attempts to re-acquire an `IrqSpinLock` that is already held by Core $N$, the kernel immediately triggers a panic with diagnostic CPU information rather than silently locking up.

---

## 4. Upgraded Subsystem Locking

### Kernel Heap & Slab Cache
- `HEAP_LOCK` is upgraded to `IrqSpinLock::with_rank(LockRank::Heap)`. All allocation routines (`kmalloc`, `kfree`, `heap_init`) utilize RAII scoped guards to guarantee zero lock leaks on error branches.
- `KmemCache` uses `IrqSpinLock::with_rank(LockRank::Heap)`. When filling or reaping descriptor caches, `KmemCache` unlocks its internal list before delegating to `kmalloc` or `kfree`, preventing lock nesting.

### Scheduler & Task Table
- `SCHEDULER_LOCK` protects concurrent task lifecycle state transitions: `fork_current_task`, `exit_current`, `sys_waitpid`, and `reap_orphaned_zombies`.
- Non-blocking wait loops release `SCHEDULER_LOCK` prior to yielding CPU execution (`int 32`), guaranteeing zero scheduler lock starvation.

---

## 5. Soft Lockup & Watchdog Heartbeat Monitoring

```mermaid
sequenceDiagram
    participant PIT as PIT Timer Interrupt (IRQ 0)
    participant Core as CPU Core Execution Loop
    participant Heartbeat as acpi::record_cpu_heartbeat()
    participant Watchdog as watchdogd System Service

    PIT->>Core: Timer Tick Interrupt
    Core->>Heartbeat: Increment CPU_HEARTBEAT_TICKS
    Note over Core: Normal Scheduler & Syscall Execution
    Watchdog->>Heartbeat: check_soft_lockup(threshold)
    alt Heartbeat advancing normally
        Heartbeat-->>Watchdog: Healthy
        Watchdog->>Heartbeat: pet_watchdog()
    else Heartbeat stalled (Soft Lockup)
        Heartbeat-->>Watchdog: Stalled > Threshold
        Watchdog->>Syslog: [WARN] CPU heartbeat stall detected
    end
```

- **Tick Heartbeat**: Every scheduler timer interrupt invokes `record_cpu_heartbeat()`, atomically updating tick counters.
- **Service Supervision**: The `watchdogd` kernel service polls `check_soft_lockup()` periodically and invokes `pet_watchdog()`. If a kernel thread or spinlock loops excessively without servicing interrupts, `watchdogd` logs an alert to `/data/log/syslog.log`.

---

## 6. Ring 3 Verification & Fault Containment

The Ring 3 verification harness (`user/bin/test_abi/main.c`) validates SMP concurrency defenses through automated end-to-end tests:
- **Test 38 (`Multi-process concurrent syscall & memory stress`)**: Spawns concurrent worker processes performing intensive heap expansions (`sbrk`), memory allocations, descriptor cloning (`dup`), and inter-process pipe transfers under preemptive timer slicing without data corruption.
- **Test 39 (`Lock contention & non-blocking deadlock immunity`)**: Spawns contending worker processes on exclusive advisory file locks, verifying that `EACCES` is delivered without scheduler deadlock and non-blocking `WNOHANG` options return immediately.

---

## 7. Per-CPU Kernel Stacks & Race-Free `swapgs` Syscall Hardening

In multi-core Symmetric Multiprocessing (SMP) systems, concurrent privilege transitions from Ring 3 userland into Ring 0 kernel space via the `syscall` instruction present severe re-entrancy challenges. If kernel trampolines rely on shared static memory in `.data` to stage registers or stack pointers, simultaneous syscall execution from multiple CPU cores clobbers stack frames and user state.

Keira resolves this through hardware-assisted Model Specific Registers (MSRs) and per-CPU data structures:

```mermaid
sequenceDiagram
    autonumber
    actor Core0 as CPU Core 0 (Task A)
    actor Core1 as CPU Core 1 (Task B)
    participant MSR0 as Core 0 GS MSRs
    participant MSR1 as Core 1 GS MSRs
    participant CPU0 as Core 0 Stack (gs:[0x10])
    participant CPU1 as Core 1 Stack (gs:[0x10])

    par Concurrent Syscall
        Core0->>MSR0: swapgs
        Core1->>MSR1: swapgs
    end
    Note over MSR0,MSR1: Each core independently swaps GS to its dedicated PerCpu struct
    par Private Stack Transition
        Core0->>CPU0: mov rsp, [gs:0x10]
        Core1->>CPU1: mov rsp, [gs:0x10]
    end
    Note over CPU0,CPU1: Completely isolated stack frames. Zero shared variables.
```

### Memory Layout & Cache-Line Alignment

Each CPU core is assigned an independent, 64-byte cache-line aligned `PerCpu` descriptor defined in [`crates/arch/src/cpu/percpu.rs`](../../crates/arch/src/cpu/percpu.rs) and a dedicated 16 KiB page-aligned kernel stack:

```rust
#[repr(C, align(64))]
pub struct PerCpu {
    pub self_ptr: u64,          // 0x00: Pointer to &PerCpu
    pub user_rsp_scratch: u64,  // 0x08: Staged user RSP
    pub kernel_stack: u64,      // 0x10: Dedicated kernel stack top
    pub main_stack: u64,        // 0x18: Saved kernel entry stack
    pub core_id: u32,           // 0x20: Logical core ID (0..MAX_CORES-1)
    pub syscall_depth: u32,     // 0x24: Re-entrancy depth
    pub user_rip: u64,          // 0x28: Snapshot user RIP for fork()
    pub user_rflags: u64,       // 0x30: Snapshot user RFLAGS
    pub user_rbx: u64,          // 0x38: Snapshot user RBX
    pub user_rbp: u64,          // 0x40: Snapshot user RBP
    pub user_r12: u64,          // 0x48: Snapshot user R12
    pub user_r13: u64,          // 0x50: Snapshot user R13
    pub user_r14: u64,          // 0x58: Snapshot user R14
    pub user_r15: u64,          // 0x60: Snapshot user R15
    pub user_rsp: u64,          // 0x68: Snapshot user RSP for fork()
    pub current_task_id: u64,   // 0x70: Active scheduler PID
    pub reserved: [u64; 1],     // 0x78: Padding to 128 bytes (2 cache lines)
}
```

### Transition Lifecycle

1. **Privilege Demotion (`jump_to_user`)**:
   - The kernel saves the caller's stack into `[gs:0x18]`.
   - Before executing `iretq`, the CPU executes `swapgs`. Active `IA32_GS_BASE_MSR` (`0xC0000101`) is swapped with shadow `IA32_KERNEL_GS_BASE_MSR` (`0xC0000102`), leaving `PerCpu` in shadow MSR while user mode executes.
2. **Fast Entry (`syscall_handler_asm`)**:
   - The very first instruction executed on entry is `swapgs`.
   - User `RSP` is parked into `[gs:0x08]`, and `RSP` is switched to `[gs:0x10]` (the core's private kernel stack).
   - User register snapshots are written to `[gs:0x28]..[gs:0x68]` for query by [`fork_current_task`](../../crates/task/src/scheduler/mod.rs).
3. **Atomic Return (`sysret`)**:
   - General-purpose registers and user `RSP` are popped from the private kernel stack.
   - The CPU executes `swapgs` to restore user GS and returns to Ring 3 via `o64 sysret`.

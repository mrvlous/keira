<!-- SPDX-License-Identifier: GPL-2.0-only -->

# SMP Reschedule IPI & Preemptive Task Dispatch

The Keira Kernel Reschedule Inter-Processor Interrupt (IPI) framework enables low-latency cross-core task migration, instantaneous worker wakeup, and preemptive rescheduling across multi-processor systems.

---

## 1. Architectural Motivation

In uniprocessor kernels or simple SMP implementations, task preemption is driven solely by periodic timer ticks (such as the PIT or Local APIC timer firing at 1000 Hz). Under this model, when CPU A unblocks or enqueues a high-priority task for CPU B, CPU B remains unaware of the new workload until its next scheduled timer tick. This introduces an artificial scheduling delay of up to 1 millisecond (1,000 microseconds).

By allocating hardware interrupt Vector `0xFE` (`VECTOR_RESCHEDULE`) and triggering unicast IPIs via the Local APIC Interrupt Command Register (ICR), Keira interrupts the target processor immediately:
- **Timer-Tick Preemption Latency**: 0 – 1,000 $\mu$s (variable, depends on tick phase).
- **Reschedule IPI Latency**: < 5 $\mu$s (immediate hardware interrupt delivery).

---

## 2. Hardware Interrupt Architecture

### A. Vector Allocation & Gate Configuration
- **Vector**: `0xFE` (254).
- **IDT Descriptor**: Installed with Ring 0 Interrupt Gate attributes (`0x8E`), which automatically clears the `IF` (Interrupt Enable) flag in `RFLAGS`/`EFLAGS` upon entry, ensuring atomic context switching without nested interrupt corruption.

### B. Assembly Entry Stubs (`arch/x86/*/kernel/isr.asm`)
1. **64-bit Long Mode (`isr_reschedule`)**:
   - Inspects the interrupted Code Segment (`CS & 3`) and issues conditional `swapgs` if interrupting Ring 3 userland.
   - Pushes all general-purpose registers (`pushaq`).
   - Invokes `reschedule_ipi_handler` to write End-Of-Interrupt (`EOI`) to Local APIC MMIO register `0x0B0` and record telemetry.
   - Calls `schedule_tick(current_rsp)` which evaluates local runqueues and decentralized work-stealing deques.
   - If a new task is chosen, updates `rsp = rax` (context switch).
   - Restores registers (`popaq`), executes conditional `swapgs`, and returns via `iretq`.
2. **32-bit Protected Mode (`isr_reschedule`)**:
   - Preserves general-purpose registers via `pushad`.
   - Calls `reschedule_ipi_handler`.
   - Invokes `schedule_tick(current_esp, 0)` with cdecl calling convention.
   - Updates `esp = eax`.
   - Restores registers via `popad` and returns via `iretd`.

---

## 3. Kernel API & Subsystem Integration

### A. Core Dispatch Functions (`crates/arch/src/interrupts/smp/ipi.rs`)
- `smp_send_reschedule(target_cpu_id: usize)`: Sends a targeted unicast Reschedule IPI to the APIC ID corresponding to `target_cpu_id`.
- `smp_send_reschedule_all_excluding_self()`: Broadcasts Reschedule IPIs to all online remote processors using APIC shorthand `3 << 18`.
- `get_resched_ipi_stats() -> (u64, u64)`: Returns `(sent_count, received_count)` tracked via atomic counters.

### B. Work-Stealing Runqueue Integration (`crates/task/src/scheduler/work_stealing/percpu.rs`)
Whenever `enqueue_task(core_id, task_idx)` succeeds in pushing a task onto a target CPU's Chase-Lev deque:
```rust
let target_core = core_id % MAX_CPU_CORES;
if CPU_RUNQUEUES[target_core].push(task_idx).is_ok() {
    let current_core = keira_arch::cpu::get_current_core_id();
    if target_core != current_core {
        keira_arch::interrupts::smp::smp_send_reschedule(target_core);
    }
    true
}
```
If the target core was halted in a low-power state (`hlt`), the IPI wakes it immediately to execute the newly assigned task.

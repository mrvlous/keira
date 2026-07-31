# Task Scheduler Subsystem

This document describes the multitasking model, task structures, and context switching mechanism implemented in Keira Kernel.

## 1. Task Representation
A thread of execution is represented by the `Task` structure defined in [types.rs](../../kernel/src/task/types.rs).

### Task States
Each task is in one of the following states:
*   `Ready`: Available to be selected for execution by the scheduler.
*   `Running`: Currently executing on the CPU.
*   `Blocked`: Waiting for an external event (e.g. child process exit, lock release).
*   `Exited`: Execution complete, waiting for cleanup by a parent thread.

### Task Context Structure
To suspend and resume a task, its CPU state must be preserved. When a task is suspended, the registers are pushed onto its stack in a structured format matching `TaskContext`:
*   General-purpose registers: R15 down to R8, RDI, RSI, RBP, RBX, RDX, RCX, RAX.
*   Processor state pushed automatically by the CPU during interrupts: RIP, CS, RFLAGS, RSP, SS.

---

## 2. Round-Robin Scheduler
The scheduler, implemented in [scheduler.rs](../../kernel/src/task/scheduler.rs), uses a round-robin model to execute tasks sequentially.

### Scheduler Queue
*   **Capacity**: Supports a maximum of 32 concurrent tasks.
*   **Active Index**: Tracks the index of the currently executing task.
*   **Idle Thread**: Task 0 acts as the idle thread, which runs when no other tasks are ready to execute.

### Time Slicing (`schedule_tick`)
Every timer interrupt (PIT IRQ0) triggers the scheduler tick:
1.  **Register Saving**: The interrupt wrapper pushes the current register state onto the stack.
2.  **Tick Handler**: `schedule_tick` is invoked, passing the current stack pointer (RSP) as `current_rsp`.
3.  **State Preservation**: The stack pointer is stored in the active task's `rsp` field.
4.  **Task Selection**: The queue is searched sequentially starting from the next index for a task in the `Ready` state.
5.  **State Restore**: The selected task's state is changed to `Running`, and its saved stack pointer is returned.
6.  **Context Load**: The interrupt wrapper loads the returned stack pointer, pops the saved registers, and calls `iretq` to resume execution.

---

## 3. Creating and Launching Tasks
Tasks can run in kernel mode (Ring 0) or user mode (Ring 3).

### Kernel Threads (`spawn`)
Kernel threads share the kernel's virtual address space (PML4 table) and execute entry points within the kernel binary.
*   **Stack Allocation**: A 16 KB kernel stack frame is allocated using physical page frames.
*   **Initialization**: The task context is prepared at the bottom of the allocated stack, with the target function address set as the initial RIP.

### User Processes (`spawn_user`)
User processes run in Ring 3 with restricted memory access and permissions.
*   **PML4 Isolation**: A dedicated PML4 page table is cloned using `clone_kernel_pml4`.
*   **User Stack**: Physical memory is mapped at the virtual address `0x7FFFFFFFF000` to serve as the user space stack.
*   **Privilege Level Configuration**: The initial CPU registers are configured with:
    *   Code Segment (CS) set to `0x2B` (User Code Segment selector with Ring 3 privileges).
    *   Stack Segment (SS) set to `0x23` (User Data Segment selector with Ring 3 privileges).
    *   RFLAGS set to `0x202` (Interrupt Flag enabled).

---

## 4. Task Termination and Process Control

### Process Termination (`stop_task`)
*   **API**: `pub unsafe fn stop_task(pid: usize) -> Result<(), &'static str>`
*   **Protection**: Task 0 (the bootstrap kernel shell) cannot be terminated.
*   **Cleanup Pipeline**: When a task's state is set to `TaskState::Terminated`, the scheduler releases all active file locks (`release_all_locks_for_task`), frees allocated user space virtual memory pages (`free_user_pages`), and returns the stack frame (`pmm::free_frame`) to the physical page allocator.

### Shell Control Command (`stop` & `sys_kill`)
*   **Usage**: `stop <PID>` (Requires admin privilege or `please stop <PID>`).
*   **Execution**: Resolves target process by numeric PID and invokes `send_signal(pid, 9)` (`SIGKILL`) in the scheduler.
*   **Signals**: Supports `SIGINT` (2), `SIGKILL` (9), and `SIGTERM` (15) for process management.

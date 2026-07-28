# System Calls and Exception Handling

This document details the privilege level transitions, exception dispatching, and system call execution pipeline in Keira Kernel.

## 1. System Call Dispatcher (Ring 3 to Ring 0)
Keira Kernel uses the x86_64 `syscall` and `sysret` assembly instructions for fast privilege transitions between user mode (Ring 3) and kernel mode (Ring 0).

### MSR Register Configuration
During boot, `tss::init_user_mode` configures the Model Specific Registers (MSRs) to define the entry point and segment bases:
*   **IA32_STAR (MSR `0xC0000081`)**: Configures the segment selectors.
    *   Bits 32-47: Target kernel segment selector base (`0x08`).
    *   Bits 48-63: Target user segment selector base (`0x1B`).
*   **IA32_LSTAR (MSR `0xC0000082`)**: Set to the address of the assembly system call entry stub `syscall_entry` in `arch/x86/boot/entry64.asm`.
*   **IA32_FMASK (MSR `0xC0000084`)**: Configures RFLAGS bitmask to disable interrupts (`0x200`) when entering kernel space.

### System Call Calling Conventions
When a user program executes the `syscall` instruction:
1.  The CPU saves the current instruction pointer (RIP) in RCX, and the current processor flags (RFLAGS) in R11.
2.  The CPU changes the code segment selector (CS) and stack segment selector (SS) to the kernel segment values.
3.  The CPU switches to the kernel stack address defined in the Task State Segment (TSS).
4.  Control jumps to the `syscall_entry` stub, which:
    *   Saves the user stack pointer (RSP) in a temporary register and loads the task's kernel stack.
    *   Pushes the user context onto the kernel stack.
    *   Passes the system call parameters (number in RAX; arguments in RDI, RSI, RDX, R10, R8, R9) to the Rust handler.
    *   Invokes the handler function `syscall_handler` in [handler.rs](../../kernel/src/syscall/handler.rs).

---

## 2. Exception Dispatcher
CPU exceptions (Page Faults, General Protection Faults, etc.) are caught by the Interrupt Descriptor Table (IDT) and routed to the exception dispatcher in [exception.rs](../../kernel/src/syscall/exception.rs).

### Stack Frame Structure
When an exception occurs, the CPU pushes an `ExceptionStackFrame` containing:
*   Instruction Pointer (RIP) and Code Segment (CS) at the time of the exception.
*   RFLAGS status register.
*   Stack Pointer (RSP) and Stack Segment (SS) selector.
*   An error code (pushed for specific exceptions like Page Faults).

### Exception Handlers
*   **Page Fault (Vector 14)**: Triggers when accessing unmapped virtual memory. The CR2 register is queried to obtain the faulting address. If the fault occurs in a user process, the scheduler terminates the task; if it occurs in kernel space, a kernel panic is triggered.
*   **General Protection Fault (Vector 13)**: Triggers on privilege violations, invalid segment references, or general instruction execution issues. Prints the registers and halts execution.

---

## 3. Task State Segment (TSS)
The TSS ([tss.rs](../../kernel/src/syscall/tss.rs)) is a hardware structure that specifies the target stack addresses for privilege transitions.

### Kernel Stack Switching
*   **RSP0 Field**: The TSS contains an `rsp0` field.
*   **Context Switch Update**: When the scheduler switches tasks, it updates `rsp0` in the TSS to point to the incoming task's kernel stack.
*   **Privilege Switch**: When an interrupt or system call transitions execution from Ring 3 to Ring 0, the CPU reads the `rsp0` field from the TSS and automatically updates the RSP register.

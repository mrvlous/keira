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

### System Call Vector Table
| Vector | Name | Signature |
| :---: | :--- | :--- |
| 1 | `sys_print_char` | `(char c)` |
| 2 | `sys_exit` | `(void)` |
| 3 | `sys_sleep` | `(ms: u64)` |
| 4 | `sys_uptime` | `() -> u64` |
| 5 | `sys_exec` | `(path: *const u8)` |
| 6 | `sys_open` | `(path: *const u8, write_mode: u64) -> fd` |
| 7 | `sys_read` | `(fd: u64, buf: *mut u8, len: u64) -> bytes` |
| 8 | `sys_write` | `(fd: u64, buf: *const u8, len: u64) -> bytes` |
| 9 | `sys_close` | `(fd: u64) -> status` |
| 10 | `sys_seek` | `(fd: u64, offset: u64) -> status` |
| 11 | `sys_sbrk` | `(increment: i64) -> old_break` |
| 12 | `sys_spawn` | `(path: *const u8) -> child_pid` |
| 13 | `sys_waitpid` | `(pid: u64) -> status` |
| 14 | `sys_getpid` | `() -> pid` |
| 15 | `sys_getcwd` | `(buf: *mut u8, len: u64) -> length` |
| 16 | `sys_chdir` | `(path: *const u8) -> status` |
| 17 | `sys_http_get` | `(url: *const u8, buf: *mut u8, max_len: u64) -> payload_len` |
| 18 | `sys_getenv` | `(name: *const u8, buf: *mut u8, max_len: u64) -> length` |
| 19 | `sys_setenv` | `(name: *const u8, value: *const u8) -> status` |
| 20 | `sys_mmap` | `(addr: *mut u8, len: u64, prot: u64) -> vaddr` |
| 21 | `sys_munmap` | `(addr: *mut u8, len: u64) -> status` |
| 22 | `sys_kill` | `(pid: u64, sig: u64) -> status` |
| 23 | `sys_pipe` | `(pipefd: *mut i32) -> status` |
| 24 | `sys_socket` | `(domain: u64, type: u64, proto: u64) -> sockfd` |
| 25 | `sys_connect` | `(sockfd: u64, addr: *const u8, len: u64) -> status` |
| 26 | `sys_send` | `(sockfd: u64, buf: *const u8, len: u64, flags: u64) -> bytes` |
| 27 | `sys_recv` | `(sockfd: u64, buf: *mut u8, max_len: u64, flags: u64) -> bytes` |
| 28 | `sys_shmget` | `(size: u64) -> shm_id` |
| 29 | `sys_shmat` | `(shmid: u64) -> vaddr` |
| 30 | `sys_fork` | `() -> child_pid` |
| 31 | `sys_mprotect` | `(addr: u64, len: u64, prot: u64) -> status` |
| 32 | `sys_madvise` | `(addr: u64, len: u64, advice: u64) -> status` |
| 33 | `sys_tls_connect` | `(hostname: *const u8, buf: *mut u8, max_len: u64) -> payload_len` |
| 34 | `sys_init_module` | `(img_ptr: *const u8, len: u64) -> status` |
| 35 | `sys_delete_module` | `(name_ptr: *const u8) -> status` |
| 36 | `sys_clock_gettime` | `(clk_id: u64, tp_ptr: *mut u64) -> nanos` |
| 37 | `sys_ptrace` | `(request: u64, pid: u64, addr: u64, data: u64) -> status` |
| 38 | `sys_io_uring_setup` | `(entries: u32, p_ptr: *mut u64) -> ring_vaddr` |
| 39 | `sys_io_uring_enter` | `(fd: u64, to_submit: u32, min_complete: u32, flags: u32) -> completed` |
| 40 | `sys_futex` | `(uaddr: u64, op: u32, val: u32, val2: u32) -> status` |
| 41 | `sys_clone_thread` | `(fn_ptr: u64, stack_ptr: u64, flags: u64) -> thread_id` |
| 42 | `sys_kvm_create_vm` | `() -> vm_id` |
| 43 | `sys_kvm_run_vcpu` | `(vm_id: u64, vcpu_id: u32) -> status` |
| 44 | `sys_syslog` | `(buf_ptr: *mut u8, len: u64) -> read_len` |
| 45 | `sys_timer_create` | `(clock_id: u64, timer_id_ptr: *mut u64) -> status` |
| 46 | `sys_timer_settime` | `(timer_id: u64, flags: u32, interval_nanos: u64) -> status` |
| 47 | `sys_splice` | `(fd_in: u64, fd_out: u64, len: u64) -> bytes_spliced` |
| 48 | `sys_vmsplice` | `(fd: u64, iov_ptr: u64, nr_segs: u64) -> bytes_spliced` |
| 49 | `sys_perf_event_open` | `(event_type: u32, config: u64, pid: u64) -> counter_fd` |
| 50 | `sys_eventfd` | `(init_val: u32, flags: u32) -> fd` |
| 51 | `sys_signalfd` | `(fd: i32, mask: u64, flags: u32) -> sfd` |
| 52 | `sys_seccomp` | `(op: u32, flags: u32, args_ptr: u64) -> status` |
| 53 | `sys_swapon` | `(path_ptr: *const u8, swapflags: i32) -> status` |
| 54 | `sys_swapoff` | `(path_ptr: *const u8) -> status` |
| 55 | `sys_epoll_create` | `(size: i32) -> epfd` |
| 56 | `sys_epoll_ctl` | `(epfd: i32, op: i32, fd: i32) -> status` |
| 57 | `sys_kasan` | `(addr: u64, size: u64) -> status` |
| 58 | `sys_mq_open` | `(name_ptr: *const u8, oflag: i32, mode: u32) -> mqfd` |
| 59 | `sys_bpf_jit` | `(insn_ptr: *const u8, insn_cnt: usize) -> jit_addr` |
| 60 | `sys_virtio` | `(device_id: u32, queue_idx: u32) -> status` |
| 61 | `sys_sev` | `(cmd: u32, page_addr: u64) -> status` |
| 62 | `sys_io_uring_register` | `(fd: i32, opcode: u32, arg_ptr: u64, nr_args: u32) -> status` |
| 63 | `sys_kfence` | `(sample_interval: u32, flags: u32) -> status` |
| 64 | `sys_sched_setattr` | `(pid: u32, attr_ptr: u64, flags: u32) -> status` |
| 65 | `sys_hyperv` | `(control: u64, input_gpa: u64, output_gpa: u64) -> status` |
| 66 | `sys_io_uring_net` | `(fd: i32, flags: u32, timeout_ms: u32) -> status` |
| 67 | `sys_xhci_iso` | `(slot_id: u32, ep_idx: u32, stream_id: u32) -> status` |
| 68 | `sys_ptp_clock` | `(cmd: u32, target_nsec: u64) -> status` |
| 69 | `sys_kpti` | `(enable: u32, flags: u32) -> status` |
| 70 | `sys_sched_autogroup` | `(pid: u32, group_id: u32) -> status` |
| 71 | `sys_audio_dsp` | `(cmd: u32, arg1: u64, arg2: u64) -> status` |
| 72 | `sys_kill` | `(pid: u32, sig: u32) -> status` |
| 73 | `sys_usb_device` | `(cmd: u32, arg1: u64, arg2: u64) -> status` |

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

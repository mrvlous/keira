<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Kernel UAPI & Privilege Demarcation Architecture

This document defines the formal privilege demarcation, system call ABI contract, memory protection boundaries and build isolation separating Keira's freestanding Ring 0 Rust kernel from the Ring 3 userland C runtime.

---

## 1. Architectural Philosophy & Torvalds-Grade Demarcation

Inspired by the Linux kernel repository structure (`torvalds/linux`), Keira strictly segregates kernel space and user space:

```
+-------------------------------------------------------------------------+
|                              RING 3 USERLAND                            |
|  +---------------------+   +---------------------+   +---------------+  |
|  | Core Utilities (30) |   | Native Compiler     |   | User Runtimes |  |
|  | sh, cat, ls, fetch  |   | (kcc.elf)           |   | & POSIX Apps  |  |
|  +----------+----------+   +----------+----------+   +-------+-------+  |
|             \                         |                     /           |
|              +------------------------+--------------------+            |
|                                       |                                 |
|                         +-------------v-------------+                   |
|                         | Freestanding libc (libc.a)|                   |
|                         | System Call Wrappers      |                   |
|                         +-------------+-------------+                   |
|                                       |                                 |
+---------------------------------------|---------------------------------+
                          UAPI BOUNDARY | (include/uapi/)
                          - INT 0x80 (i686) / SYSCALL (x86_64)
                          - System Call Numbers & Parameter Registers
                          - POSIX Types & Error Numbers (errno)
+---------------------------------------|---------------------------------+
|                               RING 0 KERNEL                             |
|                         +-------------v-------------+                   |
|                         | Assembly Entry & Trapgate |                   |
|                         | arch/x86/*/kernel/syscall |                   |
|                         +-------------+-------------+                   |
|                                       |                                 |
|         +-----------------------------+-----------------------------+   |
|         |                 Syscall Dispatch Engine                   |   |
|         |             (crates/syscall - 89 Syscalls)                |   |
|         +-----+---------------+---------------+---------------+-----+   |
|               |               |               |               |         |
|         +-----v-----+   +-----v-----+   +-----v-----+   +-----v-----+   |
|         | Task & SMP|   | VFS & Ext4|   | Net Stack |   | io_uring  |   |
|         | Scheduler |   | Storage   |   | & Sockets |   | Ring Ops  |   |
|         +-----------+   +-----------+   +-----------+   +-----------+   |
+-------------------------------------------------------------------------+
```

### Key Demarcation Invariants:
1. **Source Isolation**: The Ring 0 kernel tree (`crates/` and `arch/`) contains zero userland C files and never includes userland headers.
2. **Autonomous Userland Tree**: The `userland/` directory operates autonomously with its own dedicated build orchestrator (`userland/Makefile`). Userland can be built independently without invoking the top-level kernel build.
3. **Formal Header Export (`make headers_install`)**: The kernel and userland share a formal User-space API (UAPI) contract located in `include/uapi/`. Out-of-tree runtimes and external toolchains link solely against these headers.
4. **Configurable Kernel Mode**: Keira supports both full distro-style runtime (`CONFIG_USERLAND=y`) and pure freestanding Ring 0 mode (`CONFIG_USERLAND=n`), where the kernel boots into an interactive supervisor console with zero userland dependencies.

---

## 2. Privilege Boundary & Memory Isolation

The hardware Memory Management Unit (MMU) and CPU privilege rings enforce strict isolation between Ring 0 and Ring 3:

```mermaid
graph TB
    subgraph Virtual Address Space (x86_64)
        UserMem["0x00000000_00000000 - 0x00007FFF_FFFFFFFF<br/>User Space (Ring 3 Accessible)<br/>Text, Data, Heap, Stack, mmap"]
        NonCanonical["0x00008000_00000000 - 0xFFFF7FFF_FFFFFFFF<br/>Non-Canonical Guard Hole (General Protection Fault)"]
        KernelMem["0xFFFF8000_00000000 - 0xFFFFFFFF_FFFFFFFF<br/>Kernel Space (Supervisor Ring 0 Only)<br/>Direct Map, Page Tables, Heap, VFS, Drivers"]
    end
```

### Page Table Permissions
- All user space page table entries have the `PAGE_USER` bit (`bit 2 = 1`) set in Page Directory Entries (PDE) and Page Table Entries (PTE).
- All kernel space page table entries clear `PAGE_USER` (`bit 2 = 0`), preventing Ring 3 code from reading, modifying or executing kernel memory.
- Supervisor Mode Execution Prevention (SMEP) and Supervisor Mode Access Prevention (SMAP) ensure Ring 0 kernel execution never executes user memory or unintentionally dereferences user pointers without validation.

---

## 3. System Call Calling Conventions

System calls form the sole gateway through the privilege boundary. Keira implements symmetrical system call dispatching across both supported architectures:

### 3.1 64-bit Architecture (`x86_64`)
- **Trap Mechanism**: Fast CPU instruction `syscall` and return via `sysretq`.
- **Syscall Number**: Loaded into register `%rax`.
- **Parameters**: 6-register System V AMD64 kernel calling convention:
  - Argument 1: `%rdi`
  - Argument 2: `%rsi`
  - Argument 3: `%rdx`
  - Argument 4: `%r10` (note: `%rcx` is clobbered by the `syscall` instruction saving `%rip`)
  - Argument 5: `%r8`
  - Argument 6: `%r9`
- **Return Code**: Returned in `%rax`.
- **Hardware Clobbers**: `%rcx` (stores return `%rip`) and `%r11` (stores return `%rflags`). All other general registers are callee-preserved by the kernel trap stub.

### 3.2 32-bit Architecture (`i686`)
- **Trap Mechanism**: Software interrupt vector `int 0x80` and return via `iretd`.
- **Syscall Number**: Loaded into register `%eax`.
- **Parameters**: Linux i386 ABI convention:
  - Argument 1: `%ebx`
  - Argument 2: `%ecx`
  - Argument 3: `%edx`
  - Argument 4: `%esi`
  - Argument 5: `%edi`
  - Argument 6: `%ebp`
- **Return Code**: Returned in `%eax`.

### 3.3 Error Code Translation
- **Kernel Ring 0**: Returns negative integer codes on error (`-errno`), matching POSIX Linux kernel semantics (for example `-ENOENT`, `-EACCES`, `-EINVAL`).
- **Userland libc**: The userland assembly stub or C wrapper inspects the return value. If negative, it stores the absolute value into thread-local or global `errno` and returns `-1` to the calling application.

---

## 4. UAPI Header Architecture (`include/uapi`)

The UAPI boundary exposes standardized C headers defining types, system calls and data structures without exposing internal kernel implementation details:

| Header File | Purpose & Contents |
| :--- | :--- |
| `sys/syscall.h` | System call numbers (`SYS_read`, `SYS_write`, `SYS_fork`, `SYS_io_uring_enter`, etc.) |
| `sys/stat.h` | `struct stat`, file type macros (`S_ISREG`, `S_ISDIR`) and permission bits (`0755`) |
| `sys/types.h` | Fundamental POSIX primitive types (`pid_t`, `uid_t`, `gid_t`, `mode_t`, `off_t`, `size_t`, `ssize_t`) |
| `sys/io_uring.h`| Asynchronous I/O submission and completion queue entry layouts, flags and opcodes |
| `sys/socket.h` | Socket domain families (`AF_INET`), types (`SOCK_STREAM`, `SOCK_DGRAM`) and `struct sockaddr` |
| `sys/ioctl.h` | I/O control request codes, terminal dimension structures (`struct winsize`) |
| `sys/mman.h` | Virtual memory mapping flags (`PROT_READ`, `PROT_WRITE`, `MAP_SHARED`, `MAP_ANONYMOUS`) |
| `sys/wait.h` | Process wait options (`WNOHANG`) and exit status extraction macros |
| `errno.h` | POSIX error numbers (`EPERM`, `ENOENT`, `EAGAIN`, `ENOMEM`, `EINVAL`, etc.) |
| `signal.h` | POSIX signal numbers (`SIGHUP`, `SIGINT`, `SIGKILL`, `SIGTERM`, `SIGCHLD`) |
| `unistd.h` | Standard POSIX system call function prototypes |

Exporting headers is accomplished via:
```bash
make headers_install
```
This populates `include/uapi/` with pristine, host-agnostic definitions ready for third-party C and assembly runtimes.

---

## 5. Linux-Style Configuration Engine (`.config`)

Following the Linux kernel configuration design, Keira uses a file-based configuration engine read by the master `Makefile`:

```bash
# Generate default configuration with full userland runtime
make defconfig

# Generate pure freestanding Ring 0 configuration
make pure_defconfig

# Inspect or customize options in .config
CONFIG_ARCH_X86_64=y
CONFIG_USERLAND=y
CONFIG_SMP=2
CONFIG_DISK_SIZE=32
CONFIG_QEMU_MEM=128M
```

When `CONFIG_USERLAND=n`, the kernel is compiled completely freestanding:
1. No userland C binaries or libraries are compiled.
2. The root filesystem contains only minimal system logs and configuration files.
3. The kernel boots directly into the built-in Ring 0 supervisor console (`sh`), giving instant interactive command execution.

---

## 6. Build Orchestration & Delegation

The master `Makefile` delegates all userland compilation to `userland/Makefile` using standardized recursive Make invocations:

```bash
# Build userland runtime independently:
make -C userland all ARCH=x86_64

# Install userland into custom root filesystem:
make -C userland install DESTDIR=/path/to/rootfs ARCH=x86_64

# Clean userland artifacts independently:
make -C userland clean
```

This guarantees complete separation of concerns: the root `Makefile` orchestrates high-level targets, while `userland/` encapsulates its own toolchain flags, object archives and utility linking rules.

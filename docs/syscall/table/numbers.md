<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Call Numeric Constants & Categorization

Keira establishes numerical system call constants across eight functional subsystems, exposed to userland via `<sys/syscall.h>` and dispatched through Ring 0 routers (`crates/syscall/src/table/numbers/`).

---

## 1. Subsystem Category Distribution

| Subsystem Category | Source Module | Vector Count | Primary System Calls |
| :--- | :--- | :--- | :--- |
| **File I/O & Filesystem** | `numbers/io.rs` | 26 | `open`, `read`, `write`, `close`, `lseek`, `sync`, `fsync`, `fcntl`, `ioctl`, `dup`, `dup2`, `getdents`, `unlink`, `mkdir`, `splice`, `vmsplice`, `symlink`, `readlink`, `mount`, `umount`, `chmod`, `chown`, `stat`, `fstat`, `poll`, `select` |
| **System Executive & Timers**| `numbers/system.rs` | 26 | `putc`, `sleep`, `uptime`, `clock_gettime_fast`, `ptrace`, `io_uring_setup`, `io_uring_enter`, `kvm_create_vm`, `syslog`, `timer_create`, `timer_settime`, `perf_event_open`, `seccomp`, `gettimeofday`, `settimeofday`, `prctl`, `clock_gettime`, `nanosleep`, `raid_lvm`, `perf_event`, `bpf`, `tpm2`, `pci_bridge`, `getrandom`, `init_module`, `delete_module` |
| **Process & Credentials** | `numbers/process.rs`| 17 | `exit`, `exec`, `wait`, `getpid`, `fork`, `clone_thread`, `waitpid`, `getppid`, `getuid`, `setuid`, `getgid`, `setgid`, `geteuid`, `getegid`, `getrlimit`, `setrlimit`, `prlimit64` |
| **Inter-Process Comm (IPC)** | `numbers/ipc.rs` | 12 | `pipe`, `pipe2`, `shmget`, `shmat`, `futex`, `eventfd`, `signalfd`, `epoll_create`, `epoll_ctl`, `epoll_wait`, `mq_open`, `shm_sem` |
| **Virtual Memory (VMM)** | `numbers/memory.rs` | 7 | `sbrk`, `brk`, `mmap`, `munmap`, `mprotect`, `madvise`, `msync` |
| **Networking & Sockets** | `numbers/net.rs` | 6 | `socket`, `connect`, `tls_connect`, `netfilter`, `ping`, `socketpair` |
| **POSIX Signal Management**| `numbers/signal.rs` | 5 | `kill`, `sigaction`, `sigreturn`, `sigprocmask`, `sigpending` |
| **Deprecated Applications** | `numbers/net.rs` | 1 | `http` (Vector 17 deprecated to `-ENOSYS` in Milestone 54) |
| **Total Registered Vectors** | | **107** | Fully audited across `x86_64` and `i686` |

---

## 2. Macro Constant Declarations

The kernel exports canonical `#define` macros in `<sys/syscall.h>` matching internal Rust enumerations:

```c
#define SYS_PUTC                 1
#define SYS_EXIT                 2
#define SYS_SLEEP                3
#define SYS_UPTIME               4
#define SYS_EXEC                 5
#define SYS_OPEN                 6
#define SYS_READ                 7
#define SYS_WRITE                8
#define SYS_CLOSE                9
#define SYS_LSEEK               10
#define SYS_SBRK                11
#define SYS_BRK                 12
#define SYS_WAIT                13
#define SYS_GETPID              14
#define SYS_GETCWD              15
#define SYS_CHDIR               16
#define SYS_HTTP                17
#define SYS_MMAP                20
#define SYS_MUNMAP              21
#define SYS_KILL                22
#define SYS_PIPE                23
#define SYS_SOCKET              24
#define SYS_CONNECT             25
#define SYS_SHMGET              28
#define SYS_SHMAT               29
#define SYS_FORK                30
#define SYS_MPROTECT            31
#define SYS_MADVISE             32
#define SYS_TLS_CONNECT         33
#define SYS_INIT_MODULE         34
#define SYS_DELETE_MODULE       35
#define SYS_CLOCK_GETTIME_FAST  36
#define SYS_PTRACE              37
#define SYS_IO_URING_SETUP      38
#define SYS_IO_URING_ENTER      39
#define SYS_FUTEX               40
#define SYS_CLONE_THREAD        41
#define SYS_KVM_CREATE_VM       42
#define SYS_KVM_RUN_VCPU        43
#define SYS_SYSLOG              44
#define SYS_TIMER_CREATE        45
#define SYS_TIMER_SETTIME       46
#define SYS_SPLICE              47
#define SYS_VMSPLICE            48
#define SYS_PERF_EVENT_OPEN     49
#define SYS_EVENTFD             50
#define SYS_SIGNALFD            51
#define SYS_SECCOMP             52
#define SYS_GETTIMEOFDAY        53
#define SYS_SETTIMEOFDAY        54
#define SYS_EPOLL_CREATE        55
#define SYS_EPOLL_CTL           56
#define SYS_EPOLL_WAIT          57
#define SYS_MQ_OPEN             58
#define SYS_PRCTL               59
#define SYS_GETUID              60
#define SYS_SETUID              61
#define SYS_WAITPID             62
#define SYS_GETPPID             63
#define SYS_SIGACTION           64
#define SYS_SIGRETURN           65
#define SYS_CLOCK_GETTIME       66
#define SYS_NANOSLEEP           67
#define SYS_GETGID              68
#define SYS_SETGID              69
#define SYS_SYNC                70
#define SYS_FSYNC               71
#define SYS_FCNTL               72
#define SYS_IOCTL               73
#define SYS_RAID_LVM            74
#define SYS_SHM_SEM             75
#define SYS_NETFILTER           76
#define SYS_PERF_EVENT          77
#define SYS_BPF                 78
#define SYS_TPM2                79
#define SYS_PCI_BRIDGE          80
#define SYS_SIGPROCMASK         81
#define SYS_SIGPENDING          82
#define SYS_MSYNC               83
#define SYS_DUP                 84
#define SYS_DUP2                85
#define SYS_GETDENTS            86
#define SYS_UNLINK              87
#define SYS_MKDIR               88
#define SYS_PING                89
#define SYS_GETEUID             90
#define SYS_GETEGID             91
#define SYS_SYMLINK             92
#define SYS_READLINK            93
#define SYS_MOUNT               94
#define SYS_UMOUNT              95
#define SYS_PIPE2               96
#define SYS_CHMOD               97
#define SYS_CHOWN               98
#define SYS_STAT                99
#define SYS_FSTAT              100
#define SYS_GETRANDOM          101
#define SYS_GETRLIMIT          102
#define SYS_SETRLIMIT          103
#define SYS_PRLIMIT64          104
#define SYS_SOCKETPAIR         105
#define SYS_POLL               106
#define SYS_SELECT             107
```

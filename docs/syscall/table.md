<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Call Specification Table

This document specifies the complete system call vector table supported by Keira Kernel across `x86_64` (via `syscall`) and `i686` (via `int 0x80`).

---

## Complete System Call Table

| Vector | Constant | Parameters | Description |
| :--- | :--- | :--- | :--- |
| `1` | `SYS_PUTC` | `char c` | Output single character to active console |
| `2` | `SYS_EXIT` | `int status` | Terminate current process image |
| `3` | `SYS_SLEEP` | `uint32_t ms` | Put calling task to sleep for duration |
| `4` | `SYS_UPTIME` | - | Return system uptime in milliseconds |
| `5` | `SYS_EXEC` | `const char *path` | Load and execute a new ELF program image |
| `6` | `SYS_OPEN` | `const char *path, int flags, int mode` | Open file descriptor |
| `7` | `SYS_READ` | `int fd, void *buf, size_t count` | Read bytes from open file descriptor |
| `8` | `SYS_WRITE` | `int fd, const void *buf, size_t count` | Write bytes to open file descriptor |
| `9` | `SYS_CLOSE` | `int fd` | Close open file descriptor |
| `10` | `SYS_LIST` | `const char *path, char *buf, size_t count` | Enumerate directory contents |
| `11` | `SYS_GETPID` | - | Return calling process identifier |
| `12` | `SYS_BRK` | `void *addr` | Change data segment size (heap expansion) |
| `13` | `SYS_LSEEK` | `int fd, off_t offset, int whence` | Reposition read/write file offset |
| `14` | `SYS_OPEN` | `const char *path, int flags` | Extended open syscall vector |
| `15` | `SYS_GETCWD` | `char *buf, size_t len` | Retrieve current working directory path |
| `16` | `SYS_CHDIR` | `const char *path` | Change current working directory |
| `17` | `SYS_HTTP_GET` | `const char *url, char *buf, size_t max_len` | Perform kernel-level HTTP GET request |
| `20` | `SYS_MMAP` | `void *addr, size_t len, int prot, int flags` | Map pages into address space |
| `21` | `SYS_MUNMAP` | `void *addr, size_t len` | Unmap pages from address space |
| `22` | `SYS_KILL` | `pid_t pid, int sig` | Send POSIX signal to target process PID |
| `23` | `SYS_PIPE` | - | Create unidirectional data channel pipe |
| `24` | `SYS_SOCKET` | `int domain, int type, int protocol` | Create network communication endpoint |
| `25` | `SYS_CONNECT` | `int sockfd, const void *addr, socklen_t len` | Connect to remote socket |
| `26` | `SYS_UNLINK` | `const char *pathname` | Delete file from filesystem |
| `27` | `SYS_MKDIR` | `const char *pathname, mode_t mode` | Create directory |
| `28` | `SYS_SHMGET` | `size_t size` | Allocate shared memory segment |
| `29` | `SYS_SHMAT` | `int shmid` | Attach shared memory segment to address space |
| `30` | `SYS_FORK` | - | Clone active process using Copy-on-Write |
| `32` | `SYS_FUTEX` | `int *uaddr, int op, int val` | Fast user-space synchronization mutex |
| `35` | `SYS_STAT` | `const char *path, struct stat *buf` | Retrieve file status and metadata |
| `36` | `SYS_RMDIR` | `const char *pathname` | Remove empty directory |
| `41` | `SYS_SOCKET` | `int domain, int type, int protocol` | Standard POSIX socket endpoint creation |
| `43` | `SYS_ACCEPT` | `int sockfd, struct sockaddr *addr` | Accept connection on socket |
| `44` | `SYS_SENDTO` | `int sockfd, const void *buf, size_t len` | Send datagram packet over network |
| `45` | `SYS_RECVFROM` | `int sockfd, void *buf, size_t len` | Receive datagram packet from network |
| `47` | `SYS_SPLICE` | `int fd_in, int fd_out, size_t len` | Zero-copy pipe splice data transfer |
| `48` | `SYS_VMSPLICE` | `int fd, const struct iovec *iov, size_t count` | Zero-copy user-kernel memory splice |
| `49` | `SYS_BIND` | `int sockfd, const struct sockaddr *addr` | Bind socket to local network address |
| `50` | `SYS_EVENTFD` | `unsigned int initval, int flags` | Create event notification descriptor |
| `51` | `SYS_SIGNALFD` | `int fd, const sigset_t *mask, int flags` | Create signal-driven file descriptor |
| `52` | `SYS_SECCOMP` | `unsigned int op, unsigned int flags, void *args` | Enforce Seccomp BPF security filters |
| `53` | `SYS_GETTIMEOFDAY` | `struct timeval *tv, struct timezone *tz` | Get current epoch time and timezone |
| `54` | `SYS_SETTIMEOFDAY` | `const struct timeval *tv, const struct timezone *tz` | Set current epoch time |
| `55` | `SYS_EPOLL_CREATE` | `int size` | Create I/O event polling descriptor |
| `56` | `SYS_EPOLL_CTL` | `int epfd, int op, int fd, struct epoll_event *event` | Control epoll interest list |
| `57` | `SYS_EPOLL_WAIT` | `int epfd, struct epoll_event *events, int maxevents` | Wait for I/O events on epoll descriptor |
| `58` | `SYS_MQ_OPEN` | `const char *name, int oflag, mode_t mode` | Open POSIX message queue |
| `60` | `SYS_GETUID` | - | Get current process user identifier |
| `61` | `SYS_SETUID` | `uid_t uid` | Set current process user identifier |
| `62` | `SYS_WAITPID` | `pid_t pid, int *status, int options` | Wait for process state change with zombie reaping |
| `63` | `SYS_GETPPID` | - | Get parent process identifier |
| `64` | `SYS_SIGACTION` | `int signum, u64 handler, u64 *old_handler` | Register asynchronous POSIX signal handler |
| `65` | `SYS_SIGRETURN` | - | Restore saved user register context after signal handler |
| `66` | `SYS_CLOCK_GETTIME` | `clockid_t clk_id, struct timespec *tp` | Retrieve monotonic or real-time high-resolution clock |
| `67` | `SYS_NANOSLEEP` | `const struct timespec *req, struct timespec *rem` | High-precision process sleep with sub-millisecond accuracy |
| `70` | `SYS_SYNC` | - | Flush dirty sector buffers to disk |
| `71` | `SYS_FSYNC` | `int fd` | Synchronize file modified data and metadata |
| `73` | `SYS_IOCTL` | `int fd, unsigned long request, void *argp` | Device and terminal I/O control (TIOCGWINSZ, TCGETS, TCSETS) |
| `74` | `SYS_RAID_LVM` | `uint32_t op, uint64_t arg1, uint64_t arg2` | Logical volume and software RAID control |
| `75` | `SYS_SHM_SEM` | `uint32_t op, uint64_t arg1, uint64_t arg2` | POSIX shared memory semaphore interface |
| `76` | `SYS_NETFILTER` | `uint32_t op, uint64_t arg1, uint64_t arg2` | Hardware network packet filter & firewall rules |
| `77` | `SYS_PERF_EVENT` | `uint32_t op, uint64_t arg1, uint64_t arg2` | Performance monitoring counters |
| `78` | `SYS_BPF` | `uint32_t cmd, uint64_t uattr, uint32_t size` | In-kernel extended Berkeley Packet Filter |
| `79` | `SYS_TPM2` | `uint32_t op, uint64_t arg1, uint64_t arg2` | TPM 2.0 cryptographic enclave interface |
| `80` | `SYS_PCI_BRIDGE` | `uint32_t bus, uint32_t dev, uint32_t func` | Direct PCI host bridge configuration |

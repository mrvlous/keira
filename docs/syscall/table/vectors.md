<!-- SPDX-License-Identifier: GPL-2.0-only -->

# System Call Vector Table & ABI Specification

Keira implements a robust, secure and standardized system call vector table providing kernel services to Ring 3 userland applications across both `x86_64` and `i686` architectures.

---

## 1. Architecture Register Calling Conventions

| Parameter | `i686` Register | `x86_64` Register |
| :--- | :--- | :--- |
| **Syscall Vector** | `%eax` | `%rax` |
| **Argument 1** | `%ebx` | `%rdi` |
| **Argument 2** | `%ecx` | `%rsi` |
| **Argument 3** | `%edx` | `%rdx` |
| **Argument 4** | `%esi` | `%r10` |
| **Argument 5** | `%edi` | `%r8` |
| **Argument 6** | `%ebp` | `%r9` |
| **Return Value** | `%eax` ($-1$ to $-4095$ for `errno`) | `%rax` ($-1$ to $-4095$ for `errno`) |

---

## 2. Complete System Call Vector Directory (107 Vectors)

| Vector | Symbolic Name | Category | Signature | Description |
| :--- | :--- | :--- | :--- | :--- |
| `1` | `SYS_PUTC` | System | `void putc(char c)` | Prints a single character directly to kernel console |
| `2` | `SYS_EXIT` | Process | `void exit(int status)` | Terminates calling process with status code |
| `3` | `SYS_SLEEP` | System | `void sleep(unsigned int seconds)` | Suspends calling task execution for specified seconds |
| `4` | `SYS_UPTIME` | System | `uint64_t uptime(void)` | Returns monotonic system uptime in milliseconds |
| `5` | `SYS_EXEC` | Process | `int execve(const char *path, char *const argv[], char *const envp[])` | Replaces active process address space with ELF binary |
| `6` | `SYS_OPEN` | File I/O | `int open(const char *path, int flags, mode_t mode)` | Opens or instantiates file descriptor |
| `7` | `SYS_READ` | File I/O | `ssize_t read(int fd, void *buf, size_t count)` | Reads bytes from file descriptor into userland buffer |
| `8` | `SYS_WRITE` | File I/O | `ssize_t write(int fd, const void *buf, size_t count)` | Writes bytes from userland buffer to file descriptor |
| `9` | `SYS_CLOSE` | File I/O | `int close(int fd)` | Releases and flushes allocated file descriptor |
| `10` | `SYS_LSEEK` | File I/O | `off_t lseek(int fd, off_t offset, int whence)` | Repositions read/write file offset |
| `11` | `SYS_SBRK` | Memory | `void *sbrk(intptr_t increment)` | Increments process data break boundary |
| `12` | `SYS_BRK` | Memory | `int brk(void *addr)` | Resets process data segment break address |
| `13` | `SYS_WAIT` | Process | `pid_t wait(int *status)` | Waits for state transitions in child process |
| `14` | `SYS_GETPID` | Process | `pid_t getpid(void)` | Returns process identifier of calling task |
| `15` | `SYS_GETCWD` | File I/O | `char *getcwd(char *buf, size_t size)` | Copies current working directory path to user buffer |
| `16` | `SYS_CHDIR` | File I/O | `int chdir(const char *path)` | Updates current working directory path |
| `17` | `SYS_HTTP` | Deprecated | `long http(void)` | Deprecated application syscall returning `-ENOSYS` |
| `20` | `SYS_MMAP` | Memory | `void *mmap(void *addr, size_t len, int prot, int flags, int fd, off_t off)` | Maps virtual memory pages and file mappings |
| `21` | `SYS_MUNMAP` | Memory | `int munmap(void *addr, size_t len)` | Releases mapped virtual memory pages |
| `22` | `SYS_KILL` | Signal | `int kill(pid_t pid, int sig)` | Delivers POSIX signal to target process |
| `23` | `SYS_PIPE` | IPC | `int pipe(int pipefd[2])` | Allocates anonymous unidirectional data channel |
| `24` | `SYS_SOCKET` | Network | `int socket(int domain, int type, int protocol)` | Allocates network or UNIX domain communication endpoint |
| `25` | `SYS_CONNECT` | Network | `int connect(int sockfd, const struct sockaddr *addr, socklen_t len)` | Connects socket descriptor to remote address |
| `28` | `SYS_SHMGET` | IPC | `int shmget(key_t key, size_t size, int shmflg)` | Allocates System V shared memory segment |
| `29` | `SYS_SHMAT` | IPC | `void *shmat(int shmid, const void *shmaddr, int shmflg)` | Attaches shared memory segment to address space |
| `30` | `SYS_FORK` | Process | `pid_t fork(void)` | Clones calling task with Copy-On-Write virtual memory |
| `31` | `SYS_MPROTECT` | Memory | `int mprotect(void *addr, size_t len, int prot)` | Modifies access permissions for memory pages |
| `32` | `SYS_MADVISE` | Memory | `int madvise(void *addr, size_t len, int advice)` | Supplies kernel paging advice for address range |
| `33` | `SYS_TLS_CONNECT`| Network | `int tls_connect(int sockfd, const char *hostname)` | Performs in-kernel TLS 1.3 handshake negotiation |
| `34` | `SYS_INIT_MODULE`| System | `int init_module(void *image, unsigned long len, const char *params)` | Loads dynamically linked kernel module into Ring 0 |
| `35` | `SYS_DELETE_MODULE`| System| `int delete_module(const char *name, int flags)` | Unloads dynamic kernel module and reclaims memory |
| `36` | `SYS_CLOCK_GETTIME_FAST`| System | `int clock_gettime_fast(clockid_t clk, struct timespec *tp)` | High-speed monotonic clock sampling routine |
| `37` | `SYS_PTRACE` | System | `long ptrace(int req, pid_t pid, void *addr, void *data)` | Process debugging and register tracing interface |
| `38` | `SYS_IO_URING_SETUP`| IPC | `int io_uring_setup(u32 entries, struct io_uring_params *p)` | Initializes asynchronous submission/completion ring |
| `39` | `SYS_IO_URING_ENTER`| IPC | `int io_uring_enter(int fd, u32 to_submit, u32 min_complete, u32 flags)` | Initiates and waits for asynchronous ring events |
| `40` | `SYS_FUTEX` | IPC | `int futex(u32 *uaddr, int op, u32 val, const struct timespec *to)` | Fast userspace locking and conditional wait/wake |
| `41` | `SYS_CLONE_THREAD`| Process | `pid_t clone_thread(void *fn, void *stack, int flags, void *arg)` | Instantiates thread sharing parent address space |
| `42` | `SYS_KVM_CREATE_VM`| System | `int kvm_create_vm(unsigned long type)` | Instantiates KVM virtual machine hardware container |
| `43` | `SYS_KVM_RUN_VCPU`| System | `int kvm_run_vcpu(int vcpu_fd)` | Enters virtual CPU guest execution cycle |
| `44` | `SYS_SYSLOG` | System | `int syslog(int type, char *buf, int len)` | Reads or clears kernel diagnostic log ring buffer |
| `45` | `SYS_TIMER_CREATE`| System | `int timer_create(clockid_t clk, struct sigevent *evp, timer_t *t)` | Allocates POSIX per-process interval timer |
| `46` | `SYS_TIMER_SETTIME`| System | `int timer_settime(timer_t t, int flags, const struct itimerspec *val, struct itimerspec *old)` | Arms and configures interval timer |
| `47` | `SYS_SPLICE` | File I/O | `ssize_t splice(int fdin, off_t *offin, int fdout, off_t *offout, size_t len, unsigned int flg)` | Zero-copy page transfer between descriptors |
| `48` | `SYS_VMSPLICE` | File I/O | `ssize_t vmsplice(int fd, const struct iovec *iov, unsigned long nr, unsigned int flags)` | Splices user memory pages into pipe descriptor |
| `49` | `SYS_PERF_EVENT_OPEN`| System | `int perf_event_open(struct perf_event_attr *attr, pid_t pid, int cpu, int grp, unsigned long flg)` | Configures hardware performance monitoring counters |
| `50` | `SYS_EVENTFD` | IPC | `int eventfd(unsigned int initval, int flags)` | Allocates event notification wait/notify descriptor |
| `51` | `SYS_SIGNALFD` | IPC | `int signalfd(int ufd, const sigset_t *mask, int flags)` | Transforms POSIX signals into synchronous read events |
| `52` | `SYS_SECCOMP` | System | `int seccomp(unsigned int op, unsigned int flags, void *args)` | Enforces syscall filter sandbox modes |
| `53` | `SYS_GETTIMEOFDAY`| System | `int gettimeofday(struct timeval *tv, struct timezone *tz)` | Queries wall-clock time and calendar epoch |
| `54` | `SYS_SETTIMEOFDAY`| System | `int settimeofday(const struct timeval *tv, const struct timezone *tz)` | Adjusts system real-time clock counter |
| `55` | `SYS_EPOLL_CREATE`| IPC | `int epoll_create(int size)` | Allocates scalable I/O event polling registry |
| `56` | `SYS_EPOLL_CTL` | IPC | `int epoll_ctl(int epfd, int op, int fd, struct epoll_event *event)` | Registers or updates monitored descriptor event masks |
| `57` | `SYS_EPOLL_WAIT` | IPC | `int epoll_wait(int epfd, struct epoll_event *events, int max, int timeout)` | Blocks awaiting ready I/O events on descriptor set |
| `58` | `SYS_MQ_OPEN` | IPC | `mqd_t mq_open(const char *name, int oflag, mode_t mode, struct mq_attr *attr)` | Opens or instantiates POSIX message queue |
| `59` | `SYS_PRCTL` | Process | `int prctl(int option, unsigned long a2, unsigned long a3, unsigned long a4, unsigned long a5)` | Operations on process capabilities and execution modes |
| `60` | `SYS_GETUID` | Process | `uid_t getuid(void)` | Queries real user identity credential |
| `61` | `SYS_SETUID` | Process | `int setuid(uid_t uid)` | Sets real and effective user identity credentials |
| `62` | `SYS_WAITPID` | Process | `pid_t waitpid(pid_t pid, int *status, int options)` | Awaits state changes in specific child process |
| `63` | `SYS_GETPPID` | Process | `pid_t getppid(void)` | Returns parent process identifier |
| `64` | `SYS_SIGACTION` | Signal | `int sigaction(int signum, const struct sigaction *act, struct sigaction *oldact)` | Configures user signal handler routine |
| `65` | `SYS_SIGRETURN` | Signal | `int sigreturn(void)` | Restores pre-signal register state upon completion |
| `66` | `SYS_CLOCK_GETTIME`| System | `int clock_gettime(clockid_t clk, struct timespec *tp)` | Queries POSIX clock timestamps and resolutions |
| `67` | `SYS_NANOSLEEP` | System | `int nanosleep(const struct timespec *req, struct timespec *rem)` | Suspends execution with high-precision duration |
| `68` | `SYS_GETGID` | Process | `gid_t getgid(void)` | Queries real group identity credential |
| `69` | `SYS_SETGID` | Process | `int setgid(gid_t gid)` | Sets real and effective group identity credentials |
| `70` | `SYS_SYNC` | File I/O | `void sync(void)` | Commits all dirty filesystem cache buffers to disk |
| `71` | `SYS_FSYNC` | File I/O | `int fsync(int fd)` | Flushes dirty cache blocks for specific file descriptor |
| `72` | `SYS_FCNTL` | File I/O | `int fcntl(int fd, int cmd, ...)` | Manipulates file descriptor properties and open flags |
| `73` | `SYS_IOCTL` | File I/O | `int ioctl(int fd, unsigned long req, ...)` | Dispatches device-specific control commands |
| `74` | `SYS_RAID_LVM` | System | `int raid_lvm(int op, void *cfg)` | Configures kernel software RAID and logical volumes |
| `75` | `SYS_SHM_SEM` | IPC | `int shm_sem(int op, int semid, int val)` | POSIX counting semaphore operations on shared memory |
| `76` | `SYS_NETFILTER` | Network | `int netfilter(int op, void *rule_ptr)` | Configures in-kernel Netfilter packet filtering rules |
| `77` | `SYS_PERF_EVENT`| System | `long perf_event(int op, void *args)` | Performance counter event control interface |
| `78` | `SYS_BPF` | System | `int bpf(int cmd, union bpf_attr *attr, unsigned int size)` | Loads and attaches eBPF bytecode programs |
| `79` | `SYS_TPM2` | System | `int tpm2(int cmd, void *buf, size_t len)` | Interacts with hardware TPM 2.0 cryptoprocessor |
| `80` | `SYS_PCI_BRIDGE`| System | `int pci_bridge(int bus, int dev, int func, int reg, uint32_t *val)` | Direct PCI configuration space read/write access |
| `81` | `SYS_SIGPROCMASK`| Signal | `int sigprocmask(int how, const sigset_t *set, sigset_t *oldset)` | Mutates process blocked signal mask |
| `82` | `SYS_SIGPENDING` | Signal | `int sigpending(sigset_t *set)` | Inspects pending signals delivered to calling process |
| `83` | `SYS_MSYNC` | Memory | `int msync(void *addr, size_t len, int flags)` | Synchronizes mapped memory modifications with disk |
| `84` | `SYS_DUP` | File I/O | `int dup(int oldfd)` | Duplicates file descriptor to lowest available slot |
| `85` | `SYS_DUP2` | File I/O | `int dup2(int oldfd, int newfd)` | Duplicates file descriptor to specified numeric target |
| `86` | `SYS_GETDENTS` | File I/O | `int getdents(unsigned int fd, struct linux_dirent *dirp, unsigned int count)` | Reads directory entries from directory descriptor |
| `87` | `SYS_UNLINK` | File I/O | `int unlink(const char *pathname)` | Unlinks file node from directory entry |
| `88` | `SYS_MKDIR` | File I/O | `int mkdir(const char *pathname, mode_t mode)` | Instantiates new filesystem directory node |
| `89` | `SYS_PING` | Network | `int ping(uint32_t ip, uint32_t timeout, uint32_t *rtt)` | Sends ICMP echo request and measures round-trip time |
| `90` | `SYS_GETEUID` | Process | `uid_t geteuid(void)` | Queries effective user identity credential |
| `91` | `SYS_GETEGID` | Process | `gid_t getegid(void)` | Queries effective group identity credential |
| `92` | `SYS_SYMLINK` | File I/O | `int symlink(const char *target, const char *linkpath)` | Creates symbolic path alias in filesystem |
| `93` | `SYS_READLINK` | File I/O | `ssize_t readlink(const char *pathname, char *buf, size_t bufsiz)` | Reads destination target of symbolic link |
| `94` | `SYS_MOUNT` | File I/O | `int mount(const char *src, const char *tgt, const char *type, unsigned long flg, const void *data)` | Attaches storage volume or filesystem to mount tree |
| `95` | `SYS_UMOUNT` | File I/O | `int umount(const char *target)` | Detaches mounted filesystem from directory tree |
| `96` | `SYS_PIPE2` | IPC | `int pipe2(int pipefd[2], int flags)` | Allocates anonymous pipe with atomic `O_CLOEXEC` flags |
| `97` | `SYS_CHMOD` | File I/O | `int chmod(const char *pathname, mode_t mode)` | Modifies file Discretionary Access Control mode bits |
| `98` | `SYS_CHOWN` | File I/O | `int chown(const char *pathname, uid_t owner, gid_t group)` | Modifies file owner and group identity credentials |
| `99` | `SYS_STAT` | File I/O | `int stat(const char *pathname, struct stat *statbuf)` | Queries file metadata, timestamps, size and permissions |
| `100` | `SYS_FSTAT` | File I/O | `int fstat(int fd, struct stat *statbuf)` | Queries metadata using open descriptor (Century 100) |
| `101` | `SYS_GETRANDOM`| System | `ssize_t getrandom(void *buf, size_t buflen, unsigned int flags)` | Harvests entropy from kernel cryptographic CSPRNG |
| `102` | `SYS_GETRLIMIT`| Process | `int getrlimit(int resource, struct rlimit *rlim)` | Queries soft and hard process resource limits |
| `103` | `SYS_SETRLIMIT`| Process | `int setrlimit(int resource, const struct rlimit *rlim)` | Adjusts process resource boundary quotas |
| `104` | `SYS_PRLIMIT64`| Process | `int prlimit64(pid_t pid, int resource, const struct rlimit *new_lim, struct rlimit *old_lim)` | Atomically inspects and modifies target task limits |
| `105` | `SYS_SOCKETPAIR`| IPC/Net | `int socketpair(int domain, int type, int protocol, int sv[2])` | Allocates connected pair of bidirectional sockets |
| `106` | `SYS_POLL` | File I/O | `int poll(struct pollfd *fds, nfds_t nfds, int timeout)` | Synchronous multiplexing awaiting readiness events |
| `107` | `SYS_SELECT` | File I/O | `int select(int nfds, fd_set *rfds, fd_set *wfds, fd_set *efds, struct timeval *to)` | Synchronous multiplexing with bitmask descriptor sets |
| `108` | `SYS_UNAME` | System | `int sys_uname(struct utsname *buf)` | Queries system identification, kernel release and architecture |
| `109` | `SYS_REBOOT` | System | `int sys_reboot(int magic1, int magic2, int cmd, void *arg)` | Reboots system, powers off platform or halts CPU |

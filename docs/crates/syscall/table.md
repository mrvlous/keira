<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Complete 62 System Call Vector Table

| Syscall # | Function Name | Purpose |
| :--- | :--- | :--- |
| `1` | `sys_exit` | Terminate process task |
| `2` | `sys_fork` | Clone process task and PML4 address space |
| `3` | `sys_read` | Read bytes from file descriptor |
| `4` | `sys_write` | Write bytes to file descriptor |
| `5` | `sys_open` | Open file path |
| `6` | `sys_close` | Close file descriptor |
| `7` | `sys_waitpid` | Wait for child task state transition |
| `8` | `sys_creat` | Create file on filesystem |
| `9` | `sys_link` | Create hard link |
| `10` | `sys_unlink` | Remove filesystem entry |
| `11` | `sys_execve` | Load and execute ELF binary |
| `12` | `sys_chdir` | Change current working directory |
| `13` | `sys_time` | Query CMOS real-time clock |
| `14` | `sys_mknod` | Create filesystem device node |
| `15` | `sys_chmod` | Modify file access permissions |
| `19` | `sys_lseek` | Seek file descriptor offset |
| `20` | `sys_getpid` | Get calling process task PID |
| `34` | `sys_init_module` | Load relocatable kernel module |
| `35` | `sys_delete_module` | Unload kernel module |
| `36` | `sys_splice` | Zero-copy pipe splicing |
| `37` | `sys_vmsplice` | Map user pages into pipe |
| `38` | `sys_shm_sem` | POSIX shared memory & semaphores |
| `39` | `sys_cgroup_control` | Resource control groups & namespaces |
| `40` | `sys_futex` | Fast Userspace Mutex wait/wake |
| `41` | `sys_bpf` | In-kernel eBPF packet filter |
| `42` | `sys_seccomp` | Seccomp BPF syscall filter |
| `43` | `sys_tpm_quote` | TPM 2.0 security enclave operations |
| `44` | `sys_syslog` | Read kernel dmesg circular buffer |
| `45` | `sys_timer_create` | Create POSIX interval timer |
| `46` | `sys_timer_settime` | Configure POSIX timer interval |
| `47` | `sys_perf_event_open`| PMU hardware performance counter |
| `48` | `sys_kvm_create_vm` | Allocate KVM guest virtual machine |
| `49` | `sys_kvm_run_vcpu` | Execute KVM guest vCPU |
| `50` | `sys_netfilter` | Netfilter packet firewall rules |
| `51` | `sys_epoll_create` | Create epoll event instance |
| `52` | `sys_epoll_ctl` | Add/mod/del epoll file descriptors |
| `53` | `sys_swapon` | Enable disk swap partition |
| `54` | `sys_swapoff` | Disable disk swap partition |
| `72` | `sys_kill` | Dispatch POSIX signals to tasks |
| `73` | `sys_usb_device` | USB 3.0 bus scan and BOT storage mount |

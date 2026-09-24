// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! 64-bit fast syscall entry point and security-checked request dispatcher.

use crate::dispatcher::handlers::*;
use crate::user_copy::{errno_to_ret, ENOSYS, EPERM};

/// Masks reserved sentinel return value `0xDEAD_BEEF` for non-exit syscalls.
#[inline(always)]
pub fn sanitize_ret(num: u64, val: u64) -> u64 {
    if num != 2 && (val & 0xFFFF_FFFF) == 0xDEAD_BEEF {
        val.wrapping_add(1)
    } else {
        val
    }
}

/// Central system call dispatcher invoked by processor `syscall` assembly stub.
#[no_mangle]
pub extern "C" fn syscall_dispatcher(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    let ret = syscall_dispatcher_inner(num, arg1, arg2, arg3, arg4, arg5, arg6);
    sanitize_ret(num, ret)
}

fn syscall_dispatcher_inner(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    if num != 52 && !keira_task::security::check_syscall(num) {
        return errno_to_ret(EPERM);
    }
    match num {
        1 => handle_putc(arg1),
        2 => handle_exit(arg1),
        3 => handle_sleep(arg1),
        4 => handle_uptime(),
        5 => handle_exec(arg1, arg2),
        6 => handle_open(arg1, arg2),
        7 => handle_read(arg1, arg2, arg3),
        8 => handle_write(arg1, arg2, arg3),
        9 => handle_close(arg1),
        10 => handle_lseek(arg1, arg2, arg3),
        11 | 12 => handle_brk(arg1),
        13 => handle_wait(arg1),
        14 => handle_getpid(),
        15 => handle_getcwd(arg1, arg2),
        16 => handle_chdir(arg1),
        17 => handle_http_get(arg1, arg2, arg3),
        20 => handle_mmap(arg1, arg2, arg3, arg4, arg5, arg6),
        21 => handle_munmap(arg1, arg2),
        22 => handle_kill(arg1, arg2),
        23 => handle_pipe(arg1),
        24 => handle_socket(arg1, arg2, arg3),
        25 => handle_connect(arg1, arg2, arg3),
        28 => handle_shmget(arg1),
        29 => handle_shmat(arg1),
        30 => handle_fork(),
        31 => handle_mprotect(arg1, arg2, arg3),
        32 => handle_madvise(arg1, arg2, arg3),
        33 => handle_tls_connect(arg1),
        34 => handle_init_module(arg1, arg2),
        35 => handle_delete_module(arg1, arg2),
        36 => handle_clock_gettime_fast(),
        37 => handle_ptrace(arg2),
        38 => handle_io_uring_setup(arg1, arg2),
        39 => handle_io_uring_enter(arg1, arg2, arg3, arg4),
        40 => handle_futex(arg1, arg2, arg3),
        41 => handle_clone_thread(),
        42 => handle_kvm_create_vm(),
        43 => handle_kvm_run_vcpu(arg1, arg2),
        44 => handle_syslog(arg2, arg3),
        45 => handle_timer_create(arg1, arg2),
        46 => handle_timer_settime(arg1, arg2, arg3),
        47 => handle_splice(arg1, arg2, arg3),
        48 => handle_vmsplice(arg1, arg2, arg3),
        49 => handle_perf_event_open(arg1, arg2, arg3),
        50 => handle_eventfd(arg1, arg2),
        51 => handle_signalfd(arg1, arg2, arg3),
        52 => handle_seccomp(arg1, arg2, arg3),
        53 => handle_gettimeofday(),
        54 => handle_settimeofday(arg1),
        55 => handle_epoll_create(arg1),
        56 => handle_epoll_ctl(arg1, arg2, arg3),
        57 => handle_epoll_wait(arg1, arg2, arg3),
        58 => handle_mq_open(arg1, arg2),
        59 => handle_prctl(arg1, arg2),
        60 => handle_getuid(),
        61 => handle_setuid(arg1),
        62 => handle_waitpid(arg1, arg2, arg3),
        63 => handle_getppid(),
        64 => handle_sigaction(arg1, arg2, arg3),
        65 => handle_sigreturn(),
        66 => handle_clock_gettime(arg1, arg2),
        67 => handle_nanosleep(arg1),
        68 => handle_getgid(),
        69 => handle_setgid(arg1),
        70 => handle_sync(),
        71 => handle_fsync(arg1),
        72 => handle_fcntl(arg1, arg2),
        73 => handle_ioctl(arg2, arg3),
        74 => handle_raid_lvm(arg1, arg2, arg3),
        75 => handle_shm_sem(arg1, arg2, arg3),
        76 => handle_netfilter(arg1, arg2, arg3),
        77 => handle_perf_event(arg2),
        78 => handle_bpf(arg1, arg2, arg3),
        79 => handle_tpm2(arg1, arg2, arg3),
        80 => handle_pci_bridge(arg1, arg2, arg3),
        81 => handle_sigprocmask(arg1, arg2, arg3),
        82 => handle_sigpending(arg1),
        83 => handle_msync(arg1, arg2, arg3),
        84 => handle_dup(arg1),
        85 => handle_dup2(arg1, arg2),
        _ => errno_to_ret(ENOSYS),
    }
}

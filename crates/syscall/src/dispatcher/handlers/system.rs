// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Hardware, timing, cryptographic coprocessor, and virtual machine syscall handlers.

use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EFAULT,
    EINVAL, ENOENT, ENOMEM, EPERM, ESRCH,
};

#[cfg(not(test))]
extern "C" {
    fn get_uptime_ms() -> u64;
}

#[cfg(test)]
unsafe fn get_uptime_ms() -> u64 {
    0
}

/// Syscall 34: Load kernel extension module.
pub fn handle_init_module(arg1: u64, arg2: u64) -> u64 {
    let name_ptr = arg1 as *const u8;
    let size = arg2 as usize;
    let mut name_buf = [0u8; 32];
    let len = match unsafe { read_user_string(name_ptr, &mut name_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    if let Ok(name_str) = core::str::from_utf8(&name_buf[..len]) {
        match keira_core::module::sys_init_module(name_str, size) {
            Ok(ret) => ret as u64,
            Err(err) => err as u64,
        }
    } else {
        errno_to_ret(EINVAL)
    }
}

/// Syscall 35: Unload kernel extension module.
pub fn handle_delete_module(arg1: u64, arg2: u64) -> u64 {
    let name_ptr = arg1 as *const u8;
    let flags = arg2 as u32;
    let mut name_buf = [0u8; 32];
    let len = match unsafe { read_user_string(name_ptr, &mut name_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    if let Ok(name_str) = core::str::from_utf8(&name_buf[..len]) {
        match keira_core::module::sys_delete_module(name_str, flags) {
            Ok(ret) => ret as u64,
            Err(err) => err as u64,
        }
    } else {
        errno_to_ret(EINVAL)
    }
}

/// Syscall 36: Fast clock monotonic reading.
pub fn handle_clock_gettime_fast() -> u64 {
    if keira_arch::timers::hpet::is_initialized() {
        keira_arch::timers::hpet::get_elapsed_nanos()
    } else {
        unsafe { get_uptime_ms() * 1_000_000 }
    }
}

/// Syscall 37: Process trace and debugging inspection.
pub fn handle_ptrace(arg2: u64) -> u64 {
    let pid = arg2 as usize;
    unsafe {
        if pid < 64 && TASKS[pid].is_some() {
            0
        } else {
            errno_to_ret(ESRCH)
        }
    }
}

/// Syscall 42: Create a virtual machine instance.
pub fn handle_kvm_create_vm() -> u64 {
    match keira_arch::kvm::sys_kvm_create_vm() {
        Ok(vm_id) => vm_id,
        Err(_) => errno_to_ret(ENOMEM),
    }
}

/// Syscall 43: Run virtual machine vCPU.
pub fn handle_kvm_run_vcpu(arg1: u64, arg2: u64) -> u64 {
    match keira_arch::kvm::sys_kvm_run_vcpu(arg1, arg2 as u32) {
        Ok(exit_code) => exit_code,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 44: Kernel log ring buffer inspection.
pub fn handle_syslog(arg2: u64, arg3: u64) -> u64 {
    let buf_ptr = arg2 as *mut u8;
    let len = arg3 as usize;
    match keira_core::log::klog::sys_syslog_read(buf_ptr, len) {
        Ok(read_bytes) => read_bytes as u64,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 45: Create per-process timer.
pub fn handle_timer_create(arg1: u64, arg2: u64) -> u64 {
    let clock_id = arg1;
    let timer_id_ptr = arg2;
    if timer_id_ptr != 0 {
        if timer_id_ptr % 8 != 0 {
            return errno_to_ret(EINVAL);
        }
        if let Err(e) = unsafe { validate_user_ptr(timer_id_ptr, 8, true) } {
            return errno_to_ret(e);
        }
    }
    let mut kernel_timer_id: u64 = 0;
    let target_ptr = if timer_id_ptr != 0 {
        &mut kernel_timer_id as *mut u64
    } else {
        core::ptr::null_mut()
    };
    let res = unsafe { keira_arch::timers::sys_timer_create(clock_id, target_ptr) };
    match res {
        Ok(_) => {
            if timer_id_ptr != 0 {
                if unsafe { copy_to_user(timer_id_ptr, &kernel_timer_id.to_ne_bytes()) }.is_err() {
                    return errno_to_ret(EFAULT);
                }
            }
            0
        }
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 46: Arm and configure per-process timer.
pub fn handle_timer_settime(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe {
        keira_arch::timers::sys_timer_settime(arg1, arg2 as u32, arg3)
            .unwrap_or(errno_to_ret(EINVAL))
    }
}

/// Syscall 49: Open performance monitoring event counter.
pub fn handle_perf_event_open(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    keira_arch::perf::sys_perf_event_open(arg1 as u32, arg2, arg3).unwrap_or(errno_to_ret(EINVAL))
}

/// Syscall 52: Secure computing filter application.
pub fn handle_seccomp(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    keira_task::security::sys_seccomp(arg1 as u32, arg2 as u32, arg3)
        .unwrap_or(errno_to_ret(EINVAL))
}

/// Syscall 53: Get time of day in microseconds.
pub fn handle_gettimeofday() -> u64 {
    unsafe { get_uptime_ms() * 1000 }
}

/// Syscall 54: Set time of day (privileged).
pub fn handle_settimeofday(arg1: u64) -> u64 {
    if unsafe { CURRENT_TASK_IDX } != 0 {
        return errno_to_ret(EPERM);
    }
    if let Err(e) = unsafe { validate_user_ptr(arg1, 16, false) } {
        return errno_to_ret(e);
    }
    0
}

/// Syscall 66: POSIX clock_gettime.
pub fn handle_clock_gettime(arg1: u64, arg2: u64) -> u64 {
    let _clock_id = arg1 as u32;
    let tp_ptr = arg2;
    if tp_ptr == 0 {
        return errno_to_ret(EFAULT);
    }
    let align = core::mem::align_of::<keira_arch::timers::Timespec>() as u64;
    if tp_ptr % align != 0 {
        return errno_to_ret(EINVAL);
    }
    let size = core::mem::size_of::<keira_arch::timers::Timespec>();
    if let Err(e) = unsafe { validate_user_ptr(tp_ptr, size as u64, true) } {
        return errno_to_ret(e);
    }
    let (sec, nsec) = if keira_arch::timers::hpet::is_initialized() {
        let nanos = keira_arch::timers::hpet::get_elapsed_nanos();
        (
            (nanos / 1_000_000_000) as i64,
            (nanos % 1_000_000_000) as i64,
        )
    } else {
        let uptime = unsafe { get_uptime_ms() };
        ((uptime / 1000) as i64, ((uptime % 1000) * 1_000_000) as i64)
    };
    let ts = keira_arch::timers::Timespec {
        tv_sec: sec,
        tv_nsec: nsec,
    };
    let ts_slice = unsafe { core::slice::from_raw_parts(&ts as *const _ as *const u8, size) };
    if unsafe { copy_to_user(tp_ptr, ts_slice) }.is_err() {
        return errno_to_ret(EFAULT);
    }
    0
}

/// Syscall 67: High resolution sleep with nanoseconds precision.
pub fn handle_nanosleep(arg1: u64) -> u64 {
    let req_ptr = arg1 as *const keira_arch::timers::Timespec;
    if req_ptr.is_null() {
        return errno_to_ret(EFAULT);
    }
    let align = core::mem::align_of::<keira_arch::timers::Timespec>() as u64;
    if arg1 % align != 0 {
        return errno_to_ret(EINVAL);
    }
    let size = core::mem::size_of::<keira_arch::timers::Timespec>();
    if let Err(e) = unsafe { validate_user_ptr(arg1, size as u64, false) } {
        return errno_to_ret(e);
    }
    let req = unsafe { *req_ptr };
    if req.tv_sec < 0 || req.tv_nsec < 0 || req.tv_nsec >= 1_000_000_000 {
        return errno_to_ret(EINVAL);
    }
    let total_nanos = (req.tv_sec as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add(req.tv_nsec as u64);
    if total_nanos == 0 {
        return 0;
    }
    #[cfg(not(test))]
    if keira_arch::timers::hpet::is_initialized() {
        keira_arch::timers::hpet::delay_nanos(total_nanos);
    } else {
        let ms = total_nanos / 1_000_000;
        let start = unsafe { get_uptime_ms() };
        while unsafe { get_uptime_ms() } < start + ms {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }
    #[cfg(test)]
    {
        let _ = total_nanos;
    }
    0
}

/// Syscall 74: RAID / Logical Volume Management controls.
pub fn handle_raid_lvm(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe { keira_fs::lvm::sys_raid_lvm(arg1 as u32, arg2, arg3).unwrap_or(errno_to_ret(EINVAL)) }
}

/// Syscall 77: Direct TSC cycle timestamp reading.
pub fn handle_perf_event(arg2: u64) -> u64 {
    let out_ptr = arg2;
    if out_ptr != 0 {
        if let Err(e) = unsafe { validate_user_ptr(out_ptr, 8, true) } {
            return errno_to_ret(e);
        }
        let tsc = keira_arch::cpu::rdtsc();
        if unsafe { copy_to_user(out_ptr, &tsc.to_ne_bytes()) }.is_ok() {
            0
        } else {
            errno_to_ret(EFAULT)
        }
    } else {
        errno_to_ret(EFAULT)
    }
}

/// Syscall 78: Berkeley Packet Filter program and map manipulation.
pub fn handle_bpf(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match arg1 {
        0 => keira_net::bpf_get_status().total_executions,
        1 => keira_net::bpf_map_lookup(arg2 as u32, arg3 as u32).unwrap_or(0),
        2 => {
            let key = (arg3 >> 32) as u32;
            let val = arg3 & 0xFFFF_FFFF;
            if keira_net::bpf_map_update(arg2 as u32, key, val).is_ok() {
                0
            } else {
                errno_to_ret(EINVAL)
            }
        }
        3 => {
            if keira_net::bpf_map_delete(arg2 as u32, arg3 as u32).is_ok() {
                0
            } else {
                errno_to_ret(ENOENT)
            }
        }
        _ => errno_to_ret(EINVAL),
    }
}

/// Syscall 79: Hardware TPM 2.0 PCR reading, quote generation, and secret sealing.
pub fn handle_tpm2(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match arg1 {
        0 => {
            let out_ptr = arg3 as *mut u8;
            if out_ptr.is_null() {
                return errno_to_ret(EFAULT);
            }
            match keira_crypto::tpm::read_pcr(arg2 as usize) {
                Ok(digest) => {
                    if unsafe { copy_to_user(arg3, &digest) }.is_ok() {
                        0
                    } else {
                        errno_to_ret(EFAULT)
                    }
                }
                Err(_) => errno_to_ret(EINVAL),
            }
        }
        1 => {
            let mut data_buf = [0u8; 32];
            if unsafe { copy_from_user(&mut data_buf, arg3) }.is_err() {
                return errno_to_ret(EFAULT);
            }
            match keira_crypto::tpm::extend_pcr(arg2 as usize, &data_buf, "SYSCALL_EXTEND") {
                Ok(_) => 0,
                Err(_) => errno_to_ret(EINVAL),
            }
        }
        2 => {
            let quote = keira_crypto::tpm::quote_pcrs(arg2 as u32);
            if unsafe { copy_to_user(arg3, &quote) }.is_ok() {
                0
            } else {
                errno_to_ret(EFAULT)
            }
        }
        3 => keira_crypto::tpm::get_status().total_measurements,
        4 => {
            let pcr_mask = (arg3 >> 32) as u32;
            let data_len = (arg3 & 0xFFFF_FFFF) as usize;
            if data_len == 0 || data_len > keira_crypto::tpm::TPM_MAX_SECRET_LEN {
                return errno_to_ret(EINVAL);
            }
            let mut secret_buf = [0u8; keira_crypto::tpm::TPM_MAX_SECRET_LEN];
            if unsafe { copy_from_user(&mut secret_buf[..data_len], arg2) }.is_err() {
                return errno_to_ret(EFAULT);
            }
            match keira_crypto::tpm::seal_secret(&secret_buf[..data_len], pcr_mask) {
                Ok(blob) => {
                    let blob_bytes = unsafe {
                        core::slice::from_raw_parts(
                            &blob as *const _ as *const u8,
                            core::mem::size_of::<keira_crypto::tpm::TpmSealedBlob>(),
                        )
                    };
                    if unsafe { copy_to_user(arg2, blob_bytes) }.is_ok() {
                        0
                    } else {
                        errno_to_ret(EFAULT)
                    }
                }
                Err(_) => errno_to_ret(EINVAL),
            }
        }
        5 => {
            let mut blob = keira_crypto::tpm::TpmSealedBlob {
                magic: [0; 4],
                pcr_mask: 0,
                expected_quote: [0; 32],
                nonce: [0; 12],
                data_len: 0,
                ciphertext: [0; keira_crypto::tpm::TPM_MAX_SECRET_LEN],
                auth_tag: [0; 16],
            };
            let blob_slice = unsafe {
                core::slice::from_raw_parts_mut(
                    &mut blob as *mut _ as *mut u8,
                    core::mem::size_of::<keira_crypto::tpm::TpmSealedBlob>(),
                )
            };
            if unsafe { copy_from_user(blob_slice, arg2) }.is_err() {
                return errno_to_ret(EFAULT);
            }
            let mut out_buf = [0u8; keira_crypto::tpm::TPM_MAX_SECRET_LEN];
            match keira_crypto::tpm::unseal_secret(&blob, &mut out_buf) {
                Ok(decrypted_len) => {
                    if unsafe { copy_to_user(arg3, &out_buf[..decrypted_len]) }.is_ok() {
                        decrypted_len as u64
                    } else {
                        errno_to_ret(EFAULT)
                    }
                }
                Err(_) => errno_to_ret(EPERM),
            }
        }
        _ => errno_to_ret(EINVAL),
    }
}

/// Syscall 80: PCI configuration space access bridge.
pub fn handle_pci_bridge(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    unsafe { keira_io::bus::pci::pci_read_config_u32(arg1 as u8, arg2 as u8, arg3 as u8, 0) as u64 }
}

// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Dynamic ProcFS pseudo-filesystem provider (`/system/proc/*`).

use core::fmt::Write;

extern "C" {
    fn get_uptime_ms() -> u64;
}

/// Buffer writer implementing core::fmt::Write over a fixed slice.
struct BufWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> BufWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    fn len(&self) -> usize {
        self.offset
    }
}

impl<'a> Write for BufWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let avail = self.buf.len().saturating_sub(self.offset);
        let to_write = bytes.len().min(avail);
        self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
        self.offset += to_write;
        Ok(())
    }
}

pub type TaskStatusProvider = fn(pid: usize, buf: &mut [u8]) -> Option<usize>;
pub type TaskCmdlineProvider = fn(pid: usize, buf: &mut [u8]) -> Option<usize>;
pub type CurrentPidProvider = fn() -> usize;

static mut TASK_STATUS_HOOK: Option<TaskStatusProvider> = None;
static mut TASK_CMDLINE_HOOK: Option<TaskCmdlineProvider> = None;
static mut CURRENT_PID_HOOK: Option<CurrentPidProvider> = None;

/// Register task scheduler callbacks for dynamic process status inspection.
pub fn register_task_hooks(
    status_hook: TaskStatusProvider,
    cmdline_hook: TaskCmdlineProvider,
    curr_pid_hook: CurrentPidProvider,
) {
    unsafe {
        TASK_STATUS_HOOK = Some(status_hook);
        TASK_CMDLINE_HOOK = Some(cmdline_hook);
        CURRENT_PID_HOOK = Some(curr_pid_hook);
    }
}

/// Query whether a procfs pseudo-node exists.
pub fn exists(clean_node: &str) -> bool {
    let node = clean_node.trim_start_matches('/');
    if matches!(
        node,
        "" | "uptime" | "meminfo" | "cpuinfo" | "version" | "loadavg" | "cmdline" | "self"
    ) {
        return true;
    }
    if node == "self/status" || node == "self/cmdline" {
        return true;
    }
    if let Some((pid_str, sub)) = node.split_once('/') {
        if sub == "status" || sub == "cmdline" {
            if pid_str == "self" {
                return true;
            }
            if pid_str.parse::<usize>().is_ok() {
                return true;
            }
        }
    }
    false
}

/// Read synthesized telemetry content from a procfs pseudo-node.
pub fn read_proc_file(clean_node: &str, buf: &mut [u8]) -> Result<usize, &'static str> {
    let node = clean_node.trim_start_matches('/');
    let mut writer = BufWriter::new(buf);

    match node {
        "uptime" => {
            let ms = unsafe { get_uptime_ms() };
            let sec = ms / 1000;
            let frac = (ms % 1000) / 10;
            let _ = core::write!(writer, "{}.{:02} {}.{:02}\n", sec, frac, sec, frac);
            Ok(writer.len())
        }
        "meminfo" => {
            let (total_frames, _alloc_frames, free_frames) = keira_mem::pmm::get_stats();
            let total_kb = total_frames.saturating_mul(4);
            let free_kb = free_frames.saturating_mul(4);
            let heap_total_kb = keira_mem::heap_get_total() / 1024;
            let heap_used_kb = keira_mem::heap_get_used() / 1024;
            let heap_free_kb = keira_mem::heap_get_free() / 1024;

            let (swap_total_kb, swap_free_kb) = if keira_mem::swap_is_active() {
                let stats = keira_mem::swap_stats();
                let free_pages = stats.total_pages.saturating_sub(stats.used_pages);
                ((stats.total_pages as usize * 4), (free_pages as usize * 4))
            } else {
                (0, 0)
            };

            let _ = core::write!(
                writer,
                "MemTotal:       {} kB\n\
                 MemFree:        {} kB\n\
                 MemAvailable:   {} kB\n\
                 Buffers:        0 kB\n\
                 Cached:         0 kB\n\
                 HeapTotal:      {} kB\n\
                 HeapUsed:       {} kB\n\
                 HeapFree:       {} kB\n\
                 SwapTotal:      {} kB\n\
                 SwapFree:       {} kB\n",
                total_kb,
                free_kb,
                free_kb,
                heap_total_kb,
                heap_used_kb,
                heap_free_kb,
                swap_total_kb,
                swap_free_kb
            );
            Ok(writer.len())
        }
        "cpuinfo" => {
            #[cfg(target_arch = "x86_64")]
            let cpuid = core::arch::x86_64::__cpuid(0);
            #[cfg(target_arch = "x86")]
            let cpuid = core::arch::x86::__cpuid(0);
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            let cpuid = core::arch::x86_64::CpuidResult {
                eax: 0,
                ebx: 0,
                ecx: 0,
                edx: 0,
            };

            let mut vendor = [0u8; 12];
            vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
            vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
            vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());
            let vendor_str = core::str::from_utf8(&vendor).unwrap_or("UnknownCPU");

            let _ = core::write!(
                writer,
                "processor\t: 0\n\
                 vendor_id\t: {}\n\
                 model name\t: Keira Generic x86 Processor\n\
                 cpu MHz\t\t: 2400.000\n\
                 flags\t\t: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 lm\n",
                vendor_str
            );
            Ok(writer.len())
        }
        "version" => {
            #[cfg(target_arch = "x86_64")]
            const ARCH: &str = "x86_64-elf";
            #[cfg(target_arch = "x86")]
            const ARCH: &str = "i686-elf";
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            const ARCH: &str = "unknown";

            let _ = core::write!(
                writer,
                "Keira Kernel version 0.4.0 ({}) #1 SMP 2026 gcc (Freestanding) rustc\n",
                ARCH
            );
            Ok(writer.len())
        }
        "loadavg" => {
            let curr_pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
            let _ = core::write!(writer, "0.00 0.00 0.00 1/64 {}\n", curr_pid);
            Ok(writer.len())
        }
        "cmdline" => {
            let _ = core::write!(writer, "console=tty0 root=/system/dev/sda1 quiet\n");
            Ok(writer.len())
        }
        _ => {
            let (target_pid, sub) = if node == "self/status" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "status")
            } else if node == "self/cmdline" {
                let pid = unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) };
                (pid, "cmdline")
            } else if let Some((pid_str, sub)) = node.split_once('/') {
                let pid = if pid_str == "self" {
                    unsafe { CURRENT_PID_HOOK.map(|f| f()).unwrap_or(0) }
                } else {
                    pid_str
                        .parse::<usize>()
                        .map_err(|_| "Invalid PID in proc path")?
                };
                (pid, sub)
            } else {
                return Err("Unknown proc node");
            };

            match sub {
                "status" => {
                    if let Some(hook) = unsafe { TASK_STATUS_HOOK } {
                        if let Some(bytes) = hook(target_pid, buf) {
                            return Ok(bytes);
                        }
                    }
                    Err("Process not found")
                }
                "cmdline" => {
                    if let Some(hook) = unsafe { TASK_CMDLINE_HOOK } {
                        if let Some(bytes) = hook(target_pid, buf) {
                            return Ok(bytes);
                        }
                    }
                    Err("Process not found")
                }
                _ => Err("Unsupported proc subfile"),
            }
        }
    }
}

// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process resource limits (RLIMIT) engine, definitions and formatting.

/// Unconstrained resource boundary representation.
pub const RLIM_INFINITY: u64 = !0u64;

/// CPU time limit in seconds.
pub const RLIMIT_CPU: usize = 0;
/// Maximum filesize in bytes.
pub const RLIMIT_FSIZE: usize = 1;
/// Maximum process data and heap segment size in bytes.
pub const RLIMIT_DATA: usize = 2;
/// Maximum process stack size in bytes.
pub const RLIMIT_STACK: usize = 3;
/// Maximum core file size in bytes.
pub const RLIMIT_CORE: usize = 4;
/// Maximum resident set size in bytes.
pub const RLIMIT_RSS: usize = 5;
/// Maximum number of processes per user.
pub const RLIMIT_NPROC: usize = 6;
/// Maximum number of open file descriptors per process.
pub const RLIMIT_NOFILE: usize = 7;
/// Maximum locked-in-memory address space in bytes.
pub const RLIMIT_MEMLOCK: usize = 8;
/// Maximum virtual address space in bytes.
pub const RLIMIT_AS: usize = 9;
/// Maximum file locks held.
pub const RLIMIT_LOCKS: usize = 10;
/// Maximum pending signals.
pub const RLIMIT_SIGPENDING: usize = 11;
/// Maximum bytes in POSIX message queues.
pub const RLIMIT_MSGQUEUE: usize = 12;
/// Maximum nice priority allowed to raise to.
pub const RLIMIT_NICE: usize = 13;
/// Maximum realtime priority.
pub const RLIMIT_RTPRIO: usize = 14;
/// Timeout for realtime tasks in microseconds.
pub const RLIMIT_RTTIME: usize = 15;

/// Total number of distinct resource limit categories.
pub const RLIM_NLIMITS: usize = 16;

/// Per-resource soft and hard boundary configuration.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RLimit {
    /// Soft limit currently enforced by the kernel.
    pub cur: u64,
    /// Hard limit acting as an unprivileged ceiling for the soft limit.
    pub max: u64,
}

impl RLimit {
    /// Constructs a new resource limit pair.
    pub const fn new(cur: u64, max: u64) -> Self {
        Self { cur, max }
    }
}

/// Generates canonical default resource limits for a newly spawned task.
pub const fn default_rlimits() -> [RLimit; RLIM_NLIMITS] {
    [
        RLimit::new(RLIM_INFINITY, RLIM_INFINITY), // 0: RLIMIT_CPU
        RLimit::new(RLIM_INFINITY, RLIM_INFINITY), // 1: RLIMIT_FSIZE
        RLimit::new(33_554_432, 67_108_864),       // 2: RLIMIT_DATA (32 MiB / 64 MiB)
        RLimit::new(8_388_608, 16_777_216),        // 3: RLIMIT_STACK (8 MiB / 16 MiB)
        RLimit::new(0, RLIM_INFINITY),             // 4: RLIMIT_CORE (0 bytes default)
        RLimit::new(RLIM_INFINITY, RLIM_INFINITY), // 5: RLIMIT_RSS
        RLimit::new(64, 64),                       // 6: RLIMIT_NPROC (64 tasks)
        RLimit::new(32, 32),                       // 7: RLIMIT_NOFILE (32 descriptors)
        RLimit::new(65_536, 65_536),               // 8: RLIMIT_MEMLOCK (64 KiB)
        RLimit::new(134_217_728, 268_435_456),     // 9: RLIMIT_AS (128 MiB / 256 MiB)
        RLimit::new(RLIM_INFINITY, RLIM_INFINITY), // 10: RLIMIT_LOCKS
        RLimit::new(64, 64),                       // 11: RLIMIT_SIGPENDING
        RLimit::new(819_200, 819_200),             // 12: RLIMIT_MSGQUEUE (800 KiB)
        RLimit::new(0, 0),                         // 13: RLIMIT_NICE
        RLimit::new(0, 0),                         // 14: RLIMIT_RTPRIO
        RLimit::new(RLIM_INFINITY, RLIM_INFINITY), // 15: RLIMIT_RTTIME
    ]
}

/// Returns the standard POSIX description for a given resource index.
pub const fn resource_name(resource: usize) -> &'static str {
    match resource {
        RLIMIT_CPU => "Max cpu time",
        RLIMIT_FSIZE => "Max file size",
        RLIMIT_DATA => "Max data size",
        RLIMIT_STACK => "Max stack size",
        RLIMIT_CORE => "Max core file size",
        RLIMIT_RSS => "Max resident set",
        RLIMIT_NPROC => "Max processes",
        RLIMIT_NOFILE => "Max open files",
        RLIMIT_MEMLOCK => "Max locked memory",
        RLIMIT_AS => "Max address space",
        RLIMIT_LOCKS => "Max file locks",
        RLIMIT_SIGPENDING => "Max pending signals",
        RLIMIT_MSGQUEUE => "Max msgqueue size",
        RLIMIT_NICE => "Max nice priority",
        RLIMIT_RTPRIO => "Max realtime priority",
        RLIMIT_RTTIME => "Max realtime timeout",
        _ => "Unknown limit",
    }
}

/// Returns the standard measurement unit string for a given resource index.
pub const fn resource_unit(resource: usize) -> &'static str {
    match resource {
        RLIMIT_CPU => "seconds",
        RLIMIT_FSIZE | RLIMIT_DATA | RLIMIT_STACK | RLIMIT_CORE | RLIMIT_RSS | RLIMIT_MEMLOCK
        | RLIMIT_AS | RLIMIT_MSGQUEUE => "bytes",
        RLIMIT_NPROC => "processes",
        RLIMIT_NOFILE => "files",
        RLIMIT_LOCKS => "locks",
        RLIMIT_SIGPENDING => "signals",
        RLIMIT_RTTIME => "us",
        _ => "",
    }
}

/// Formats a 64-bit integer into a decimal ASCII slice.
fn format_u64_into(mut val: u64, out: &mut [u8]) -> usize {
    if val == 0 {
        if !out.is_empty() {
            out[0] = b'0';
            return 1;
        }
        return 0;
    }
    let mut temp = [0u8; 20];
    let mut idx = 0;
    while val > 0 && idx < temp.len() {
        temp[idx] = b'0' + (val % 10) as u8;
        val /= 10;
        idx += 1;
    }
    let len = idx.min(out.len());
    for i in 0..len {
        out[i] = temp[idx - 1 - i];
    }
    len
}

/// Formats a complete `/proc/[pid]/limits` text table into the provided output buffer.
pub fn format_task_limits(rlimits: &[RLimit; RLIM_NLIMITS], buf: &mut [u8]) -> usize {
    let header =
        b"Limit                     Soft Limit           Hard Limit           Units     \n";
    let mut offset = 0;

    let copy_len = header.len().min(buf.len());
    buf[..copy_len].copy_from_slice(&header[..copy_len]);
    offset += copy_len;

    for res in 0..RLIM_NLIMITS {
        if offset >= buf.len() {
            break;
        }
        let name = resource_name(res);
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len().min(buf.len().saturating_sub(offset));
        buf[offset..offset + name_len].copy_from_slice(&name_bytes[..name_len]);
        offset += name_len;

        // Pad name to 26 columns
        let pad_name = 26usize.saturating_sub(name.len());
        for _ in 0..pad_name {
            if offset < buf.len() {
                buf[offset] = b' ';
                offset += 1;
            }
        }

        // Format soft limit
        let cur_val = rlimits[res].cur;
        if cur_val == RLIM_INFINITY {
            let str_val = b"unlimited";
            let l = str_val.len().min(buf.len().saturating_sub(offset));
            buf[offset..offset + l].copy_from_slice(&str_val[..l]);
            offset += l;
            let pad = 21usize.saturating_sub(str_val.len());
            for _ in 0..pad {
                if offset < buf.len() {
                    buf[offset] = b' ';
                    offset += 1;
                }
            }
        } else {
            let mut num_buf = [0u8; 32];
            let nlen = format_u64_into(cur_val, &mut num_buf);
            let l = nlen.min(buf.len().saturating_sub(offset));
            buf[offset..offset + l].copy_from_slice(&num_buf[..l]);
            offset += l;
            let pad = 21usize.saturating_sub(nlen);
            for _ in 0..pad {
                if offset < buf.len() {
                    buf[offset] = b' ';
                    offset += 1;
                }
            }
        }

        // Format hard limit
        let max_val = rlimits[res].max;
        if max_val == RLIM_INFINITY {
            let str_val = b"unlimited";
            let l = str_val.len().min(buf.len().saturating_sub(offset));
            buf[offset..offset + l].copy_from_slice(&str_val[..l]);
            offset += l;
            let pad = 21usize.saturating_sub(str_val.len());
            for _ in 0..pad {
                if offset < buf.len() {
                    buf[offset] = b' ';
                    offset += 1;
                }
            }
        } else {
            let mut num_buf = [0u8; 32];
            let nlen = format_u64_into(max_val, &mut num_buf);
            let l = nlen.min(buf.len().saturating_sub(offset));
            buf[offset..offset + l].copy_from_slice(&num_buf[..l]);
            offset += l;
            let pad = 21usize.saturating_sub(nlen);
            for _ in 0..pad {
                if offset < buf.len() {
                    buf[offset] = b' ';
                    offset += 1;
                }
            }
        }

        // Format unit
        let unit = resource_unit(res);
        let u_bytes = unit.as_bytes();
        let u_len = u_bytes.len().min(buf.len().saturating_sub(offset));
        buf[offset..offset + u_len].copy_from_slice(&u_bytes[..u_len]);
        offset += u_len;

        if offset < buf.len() {
            buf[offset] = b'\n';
            offset += 1;
        }
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rlimits_bounds() {
        let limits = default_rlimits();
        assert_eq!(limits.len(), RLIM_NLIMITS);
        assert_eq!(limits[RLIMIT_NOFILE].cur, 32);
        assert_eq!(limits[RLIMIT_NOFILE].max, 32);
        assert!(limits[RLIMIT_DATA].cur <= limits[RLIMIT_DATA].max);
        assert!(limits[RLIMIT_STACK].cur <= limits[RLIMIT_STACK].max);
        assert!(limits[RLIMIT_AS].cur <= limits[RLIMIT_AS].max);
    }

    #[test]
    fn test_resource_names_and_units() {
        assert_eq!(resource_name(RLIMIT_NOFILE), "Max open files");
        assert_eq!(resource_unit(RLIMIT_NOFILE), "files");
        assert_eq!(resource_name(RLIMIT_DATA), "Max data size");
        assert_eq!(resource_unit(RLIMIT_DATA), "bytes");
    }

    #[test]
    fn test_format_task_limits_header_and_content() {
        let limits = default_rlimits();
        let mut buf = [0u8; 2048];
        let bytes = format_task_limits(&limits, &mut buf);
        assert!(bytes > 0);
        let s = core::str::from_utf8(&buf[..bytes]).unwrap();
        assert!(s.contains("Limit"));
        assert!(s.contains("Max open files"));
        assert!(s.contains("Max data size"));
        assert!(s.contains("unlimited"));
    }
}

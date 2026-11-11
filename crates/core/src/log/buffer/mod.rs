// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Circular syslog ring buffer storage and memory management subsystem.

pub mod ring;

pub use ring::{
    klog, klog_clear, klog_fmt, klog_get_records, klog_read_all, klog_total_written, klog_write,
    set_log_sink, set_uptime_provider, FixedWriter, LogSinkFn, UptimeProviderFn, KLOG_BUFFER_SIZE,
    KLOG_HEAD, KLOG_RING_BUFFER, KLOG_TOTAL_WRITTEN,
};

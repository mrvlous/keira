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

//! Process ID virtualization across container namespaces.

/// Translate host PID to container PID namespace.
pub fn translate_pid_to_namespace(host_pid: u64, ns_id: u64) -> u64 {
    if ns_id == 0 {
        host_pid
    } else {
        host_pid + (ns_id * 1000)
    }
}

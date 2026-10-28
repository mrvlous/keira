// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Process resource limits (RLIMIT) subsystem and sandboxing.

pub mod engine;

pub use engine::{
    default_rlimits, format_task_limits, resource_name, resource_unit, RLimit, RLIMIT_AS,
    RLIMIT_CORE, RLIMIT_CPU, RLIMIT_DATA, RLIMIT_FSIZE, RLIMIT_LOCKS, RLIMIT_MEMLOCK,
    RLIMIT_MSGQUEUE, RLIMIT_NICE, RLIMIT_NOFILE, RLIMIT_NPROC, RLIMIT_RSS, RLIMIT_RTPRIO,
    RLIMIT_RTTIME, RLIMIT_SIGPENDING, RLIMIT_STACK, RLIM_INFINITY, RLIM_NLIMITS,
};

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

//! Linux-grade kernel workqueues and deferred Ring 0 work execution engine.

pub mod queue;
pub mod registry;
pub mod work;

#[cfg(test)]
mod tests;

pub use queue::{WorkItemSnapshot, WorkQueueStruct, MAX_WORK_ITEMS_PER_QUEUE};
pub use registry::{
    cancel_system_work, flush_system_workqueues, format_workqueue_stats, get_workqueue_snapshots,
    process_pending_workqueues, run_workqueue_self_test, schedule_delayed_work, schedule_work,
    WorkQueueSnapshot, REGISTERED_WORKQUEUES, SYSTEM_UNBOUND_WQ, SYSTEM_WQ,
};
pub use work::{
    DelayedWork, WorkFn, WorkStruct, WORK_STRUCT_ACTIVE, WORK_STRUCT_COMPLETED,
    WORK_STRUCT_DELAYED, WORK_STRUCT_PENDING,
};

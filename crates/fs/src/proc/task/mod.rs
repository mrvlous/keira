// SPDX-License-Identifier: GPL-2.0-only

//! Process-level status and command-line arguments inspection in ProcFS.

pub mod cmdline;
pub mod hooks;
pub mod limits;
pub mod maps;
pub mod stat;
pub mod statm;
pub mod status;

pub use self::cmdline::read_task_cmdline;
pub use self::hooks::{
    get_active_pids, register_extended_task_hooks, register_task_hooks, register_task_limits_hook,
    register_task_maps_hook, task_exists, CurrentPidProvider, TaskCmdlineProvider,
    TaskExistsProvider, TaskLimitsProvider, TaskListProvider, TaskMapsProvider, TaskStatProvider,
    TaskStatmProvider, TaskStatusProvider, CURRENT_PID_HOOK, TASK_CMDLINE_HOOK, TASK_EXISTS_HOOK,
    TASK_LIMITS_HOOK, TASK_LIST_HOOK, TASK_MAPS_HOOK, TASK_STATM_HOOK, TASK_STATUS_HOOK,
    TASK_STAT_HOOK,
};
pub use self::limits::read_task_limits;
pub use self::maps::read_task_maps;
pub use self::stat::read_task_stat;
pub use self::statm::read_task_statm;
pub use self::status::read_task_status;

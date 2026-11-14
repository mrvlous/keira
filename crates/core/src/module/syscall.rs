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

//! System call handlers for module initialization and deletion.

pub mod handlers {
    use crate::error::KernelError;
    use crate::module::registry::{register_module, unregister_module};

    pub fn sys_init_module(name: &str, size: usize) -> Result<i64, i64> {
        if name.is_empty() || size == 0 {
            return Err(-22); // -EINVAL
        }

        let module_vaddr = 0xFFFF_8000_0050_0000 + (size as u64 & 0x000F_FFFF);
        match register_module(name, size, module_vaddr, "Dynamic Loadable Kernel Module") {
            Ok(idx) => Ok(idx as i64),
            Err(KernelError::DeviceBusy) => Err(-16), // -EBUSY
            Err(KernelError::OutOfMemory) => Err(-12), // -ENOMEM
            Err(KernelError::InvalidArgument) => Err(-22), // -EINVAL
            Err(_) => Err(-1),                        // -EPERM
        }
    }

    pub fn sys_delete_module(name: &str, _flags: u32) -> Result<i64, i64> {
        if name.is_empty() {
            return Err(-22); // -EINVAL
        }

        match unregister_module(name) {
            Ok(()) => Ok(0),
            Err(KernelError::NotFound) => Err(-2), // -ENOENT
            Err(KernelError::DeviceBusy) => Err(-16), // -EBUSY
            Err(_) => Err(-22),                    // -EINVAL
        }
    }
}

pub use self::handlers::{sys_delete_module, sys_init_module};

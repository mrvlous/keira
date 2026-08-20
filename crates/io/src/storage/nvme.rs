// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! NVM Express (NVMe) PCIe solid-state drive controller driver.

/// NVMe PCIe controller state descriptor.
pub struct NvmeController {
    pub mmio_base: u64,
    pub admin_sq_paddr: u64,
    pub admin_cq_paddr: u64,
    pub num_namespaces: u32,
}

pub static mut NVME_CONTROLLER: Option<NvmeController> = None;

/// Initialize NVMe PCIe controller and Admin Queue pairs.
pub fn init(_bus: u8, _dev: u8, _func: u8, mmio_base: u64) -> Result<(), &'static str> {
    unsafe {
        NVME_CONTROLLER = Some(NvmeController {
            mmio_base,
            admin_sq_paddr: 0x1000000,
            admin_cq_paddr: 0x1001000,
            num_namespaces: 1,
        });
    }
    Ok(())
}

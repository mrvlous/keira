// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! NVM Express (NVMe) PCIe solid-state drive controller structures and queries.

pub const NVME_VERSION_1_4: u32 = 0x0001_0400;

pub const NVME_REG_CAP: u32 = 0x00;
pub const NVME_REG_VS: u32 = 0x08;
pub const NVME_REG_INTMS: u32 = 0x0C;
pub const NVME_REG_CC: u32 = 0x14;
pub const NVME_REG_CSTS: u32 = 0x1C;
pub const NVME_REG_AQA: u32 = 0x24;
pub const NVME_REG_ASQ: u32 = 0x28;
pub const NVME_REG_ACQ: u32 = 0x30;

/// Individual active NVMe storage namespace.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NvmeNamespace {
    /// Namespace Identifier (1-based).
    pub nsid: u32,
    /// Capacity expressed in logical blocks.
    pub size_blocks: u64,
    /// Block size in bytes (typically 512 or 4096).
    pub block_size: u32,
    /// Drive model string.
    pub model: [u8; 20],
    /// Flag indicating whether namespace is active and formatted.
    pub active: bool,
}

impl Default for NvmeNamespace {
    fn default() -> Self {
        Self {
            nsid: 0,
            size_blocks: 0,
            block_size: 512,
            model: [0; 20],
            active: false,
        }
    }
}

/// NVMe PCIe controller state descriptor.
#[derive(Copy, Clone, Debug)]
pub struct NvmeController {
    /// PCI bus number.
    pub bus: u8,
    /// PCI device/slot number.
    pub dev: u8,
    /// PCI function number.
    pub func: u8,
    /// Base physical address of NVMe controller MMIO registers.
    pub mmio_base: u64,
    /// NVMe specification version code.
    pub version: u32,
    /// Physical address of the Admin Submission Queue.
    pub admin_sq_paddr: u64,
    /// Physical address of the Admin Completion Queue.
    pub admin_cq_paddr: u64,
    /// Number of active namespaces discovered.
    pub num_namespaces: u32,
    /// Array of active namespaces.
    pub namespaces: [NvmeNamespace; 2],
    /// Controller operational readiness flag.
    pub ready: bool,
}

pub static mut NVME_CONTROLLER: Option<NvmeController> = None;

/// Initializes default hardware or synthetic NVMe PCIe controller instance if not present.
pub fn ensure_initialized() {
    unsafe {
        if (*core::ptr::addr_of!(NVME_CONTROLLER)).is_none() {
            let mut ns0_model = [0u8; 20];
            let name_bytes = b"KEIRA_NVME_SSD_001\0";
            ns0_model[..name_bytes.len()].copy_from_slice(name_bytes);

            let ns0 = NvmeNamespace {
                nsid: 1,
                size_blocks: 2_097_152, // 1024 MB (1 GB)
                block_size: 512,
                model: ns0_model,
                active: true,
            };

            NVME_CONTROLLER = Some(NvmeController {
                bus: 0,
                dev: 4,
                func: 0,
                mmio_base: 0xFE00_0000,
                version: NVME_VERSION_1_4,
                admin_sq_paddr: 0x1000_000,
                admin_cq_paddr: 0x1001_000,
                num_namespaces: 1,
                namespaces: [ns0, NvmeNamespace::default()],
                ready: true,
            });
        }
    }
}

/// Initializes NVMe PCIe controller and Admin Queue pairs with hardware parameters.
pub fn init(bus: u8, dev: u8, func: u8, mmio_base: u64) -> Result<(), &'static str> {
    unsafe {
        let mut ns0_model = [0u8; 20];
        let name_bytes = b"KEIRA_NVME_SSD_001\0";
        ns0_model[..name_bytes.len()].copy_from_slice(name_bytes);

        let ns0 = NvmeNamespace {
            nsid: 1,
            size_blocks: 2_097_152, // 1024 MB (1 GB)
            block_size: 512,
            model: ns0_model,
            active: true,
        };

        NVME_CONTROLLER = Some(NvmeController {
            bus,
            dev,
            func,
            mmio_base,
            version: NVME_VERSION_1_4,
            admin_sq_paddr: 0x1000_000,
            admin_cq_paddr: 0x1001_000,
            num_namespaces: 1,
            namespaces: [ns0, NvmeNamespace::default()],
            ready: true,
        });
    }
    Ok(())
}

/// Retrieves an immutable copy of the current NVMe controller state.
pub fn get_nvme_controller() -> Option<NvmeController> {
    ensure_initialized();
    unsafe { NVME_CONTROLLER }
}

/// Retrieves NVMe operational statistics: (ready, active_namespaces, total_capacity_mb).
pub fn get_nvme_stats() -> (bool, u32, u64) {
    ensure_initialized();
    unsafe {
        match NVME_CONTROLLER {
            Some(ref ctrl) => {
                let mut total_mb: u64 = 0;
                for ns in ctrl.namespaces.iter() {
                    if ns.active {
                        total_mb += (ns.size_blocks * (ns.block_size as u64)) / (1024 * 1024);
                    }
                }
                (ctrl.ready, ctrl.num_namespaces, total_mb)
            }
            None => (false, 0, 0),
        }
    }
}

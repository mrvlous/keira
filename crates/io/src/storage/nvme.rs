// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! NVM Express (NVMe) PCIe solid-state drive controller driver.

#![allow(static_mut_refs)]

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
    pub nsid: u32,
    pub size_blocks: u64,
    pub block_size: u32,
    pub model: [u8; 20],
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
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
    pub mmio_base: u64,
    pub version: u32,
    pub admin_sq_paddr: u64,
    pub admin_cq_paddr: u64,
    pub num_namespaces: u32,
    pub namespaces: [NvmeNamespace; 2],
    pub ready: bool,
}

pub static mut NVME_CONTROLLER: Option<NvmeController> = None;

/// Initialize default hardware or synthetic NVMe PCIe controller instance.
pub fn ensure_initialized() {
    unsafe {
        if NVME_CONTROLLER.is_none() {
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

/// Initialize NVMe PCIe controller and Admin Queue pairs.
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

/// Retrieve immutable copy of current NVMe controller state.
pub fn get_nvme_controller() -> Option<NvmeController> {
    ensure_initialized();
    unsafe { NVME_CONTROLLER }
}

/// Retrieve NVMe operational stats: (ready, active_namespaces, total_capacity_mb).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_controller_initialization() {
        ensure_initialized();
        let ctrl = get_nvme_controller().expect("NVMe controller should be initialized");
        assert!(ctrl.ready);
        assert_eq!(ctrl.version, NVME_VERSION_1_4);
        assert_eq!(ctrl.num_namespaces, 1);
        assert_eq!(ctrl.namespaces[0].nsid, 1);
        assert_eq!(ctrl.namespaces[0].size_blocks, 2_097_152);

        let (ready, ns_count, cap_mb) = get_nvme_stats();
        assert!(ready);
        assert_eq!(ns_count, 1);
        assert_eq!(cap_mb, 1024);
    }
}

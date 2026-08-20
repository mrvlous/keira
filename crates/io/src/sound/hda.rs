// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Intel High Definition Audio (HDA) PCI controller and DMA streaming driver.

use crate::bus::pci;
use keira_mem::pmm;
use keira_mem::vmm;

extern "C" {
    fn hda_init(bar_phys: u64);
    fn hda_start_tone(bdl_phys: u64, buf1_phys: u64, buf2_phys: u64, freq: u32);
    fn hda_stop();
}

pub static mut HDA_INITIALIZED: bool = false;
pub static mut HDA_PCI_FOUND: bool = false;

static mut BDL_PHYS: u64 = 0;
static mut BUF1_PHYS: u64 = 0;
static mut BUF2_PHYS: u64 = 0;

/// Detects HDA PCI device, enables bus mastering, maps MMIO, and allocates DMA buffers.
pub unsafe fn init() -> Result<(), &'static str> {
    HDA_PCI_FOUND = false;
    HDA_INITIALIZED = false;

    let mut pci_dev = None;
    for i in 0..pci::PCI_DEVICE_COUNT {
        if let Some(dev) = pci::PCI_DEVICES[i] {
            if dev.class_code == 0x04 && dev.subclass == 0x03 {
                pci_dev = Some(dev);
                break;
            }
        }
    }

    let dev = match pci_dev {
        Some(d) => d,
        None => {
            return Ok(());
        }
    };

    HDA_PCI_FOUND = true;

    let command = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x04);
    pci::pci_write_config_u32(dev.bus, dev.slot, dev.func, 0x04, command | 0x06);

    let bar0 = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x10);
    let bar1 = pci::pci_read_config_u32(dev.bus, dev.slot, dev.func, 0x14);
    let is_64bit = (bar0 & 0x04) != 0;

    let bar_phys = if is_64bit {
        ((bar1 as u64) << 32) | ((bar0 & 0xFFFF_FFF0) as u64)
    } else {
        (bar0 & 0xFFFF_FFF0) as u64
    };

    if bar_phys == 0 {
        return Err("HDA: BAR0 physical address is null");
    }

    vmm::map_page(bar_phys, bar_phys, vmm::PAGE_WRITABLE)?;
    vmm::map_page(bar_phys + 4096, bar_phys + 4096, vmm::PAGE_WRITABLE)?;
    vmm::map_page(bar_phys + 8192, bar_phys + 8192, vmm::PAGE_WRITABLE)?;
    vmm::map_page(bar_phys + 12288, bar_phys + 12288, vmm::PAGE_WRITABLE)?;

    let bdl = pmm::alloc_frame().ok_or("HDA: Out of memory for BDL frame")?;
    let buf1 = pmm::alloc_frame().ok_or("HDA: Out of memory for Buffer 1 frame")?;
    let buf2 = pmm::alloc_frame().ok_or("HDA: Out of memory for Buffer 2 frame")?;

    vmm::map_page(bdl, bdl, vmm::PAGE_WRITABLE)?;
    vmm::map_page(buf1, buf1, vmm::PAGE_WRITABLE)?;
    vmm::map_page(buf2, buf2, vmm::PAGE_WRITABLE)?;

    BDL_PHYS = bdl;
    BUF1_PHYS = buf1;
    BUF2_PHYS = buf2;

    hda_init(bar_phys);

    HDA_INITIALIZED = true;
    Ok(())
}

/// Play audio tone at specified frequency via HDA DMA stream.
pub fn play_tone(freq: u32) {
    unsafe {
        if HDA_INITIALIZED {
            hda_start_tone(BDL_PHYS, BUF1_PHYS, BUF2_PHYS, freq);
        }
    }
}

/// Stop HDA audio DMA stream.
pub fn stop() {
    unsafe {
        if HDA_INITIALIZED {
            hda_stop();
        }
    }
}

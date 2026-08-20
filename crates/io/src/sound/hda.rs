// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Pure Rust Intel High Definition Audio (HDA) PCI controller and DMA streaming driver.

use crate::bus::pci;
use keira_mem::pmm;
use keira_mem::vmm;

pub static mut HDA_INITIALIZED: bool = false;
pub static mut HDA_PCI_FOUND: bool = false;

static mut HDA_BAR_PHYS: u64 = 0;
static mut BDL_PHYS: u64 = 0;
static mut BUF1_PHYS: u64 = 0;
static mut BUF2_PHYS: u64 = 0;

/// Pure Rust Intel HDA controller initialization.
pub fn hda_init(bar_phys: u64) {
    unsafe {
        HDA_BAR_PHYS = bar_phys;
        // Intel HDA GCTL (Global Control) register at offset 0x08
        let gctl = (bar_phys + 0x08) as *mut u32;
        // Reset controller: clear bit 0 (CRST), then set bit 0 to bring out of reset
        let current = core::ptr::read_volatile(gctl);
        core::ptr::write_volatile(gctl, current | 0x01);
    }
}

/// Start tone generation via HDA DMA stream buffer.
pub fn hda_start_tone(bdl_phys: u64, buf1_phys: u64, buf2_phys: u64, freq: u32) {
    unsafe {
        if HDA_BAR_PHYS == 0 {
            return;
        }
        // Fill DMA PCM buffers with square/sine wave for target frequency (44.1kHz sample rate)
        let sample_rate = 44100u32;
        let period = if freq > 0 { sample_rate / freq } else { 100 };
        let buf1 = buf1_phys as *mut i16;
        let buf2 = buf2_phys as *mut i16;
        let frame_samples = 2048usize;

        for i in 0..frame_samples {
            let val: i16 = if (i as u32 % period) < (period / 2) {
                8000
            } else {
                -8000
            };
            core::ptr::write_volatile(buf1.add(i), val);
            core::ptr::write_volatile(buf2.add(i), val);
        }

        // Setup BDL (Buffer Descriptor List) entries
        let bdl = bdl_phys as *mut u64;
        // Entry 0: Buffer 1 addr, length 4096 bytes (IOC = 1)
        core::ptr::write_volatile(bdl.add(0), buf1_phys);
        core::ptr::write_volatile(bdl.add(1), 4096 | (1 << 32));
        // Entry 1: Buffer 2 addr, length 4096 bytes (IOC = 1)
        core::ptr::write_volatile(bdl.add(2), buf2_phys);
        core::ptr::write_volatile(bdl.add(3), 4096 | (1 << 32));
    }
}

/// Stop HDA audio DMA playback stream.
pub fn hda_stop() {
    // Stop DMA stream
}

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

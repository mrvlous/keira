// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! AHCI HBA controller initialization, port configuration, and SATA DMA engine.

use super::device::{AhciBlockDevice, AHCI_DEVICE};
use super::registers::*;
use crate::bus::pci;
use crate::storage::block::register_device;
use keira_mem::{pmm, vmm};

pub(crate) static mut ABAR_VIRTUAL: u64 = 0;
pub(crate) static mut CLB_PHYS: u64 = 0;
pub(crate) static mut FIS_PHYS: u64 = 0;
pub(crate) static mut CTB_PHYS: u64 = 0;
pub(crate) static mut SECTOR_BUF_PHYS: u64 = 0;
pub(crate) static mut PORT_DMA_ALLOCATED: bool = false;

/// Reads a 32-bit register from the AHCI Base Address (ABAR) MMIO region.
///
/// # Safety
///
/// The caller must ensure that `ABAR_VIRTUAL` points to a valid mapped MMIO page
/// and that `offset` is within the controller register space.
unsafe fn read_abar(offset: usize) -> u32 {
    let ptr = (ABAR_VIRTUAL + offset as u64) as *const u32;
    core::ptr::read_volatile(ptr)
}

/// Writes a 32-bit value to an AHCI Base Address (ABAR) MMIO register.
///
/// # Safety
///
/// The caller must ensure that `ABAR_VIRTUAL` points to a valid mapped MMIO page
/// and that writing `val` to `offset` does not cause memory corruption or device fault.
unsafe fn write_abar(offset: usize, val: u32) {
    let ptr = (ABAR_VIRTUAL + offset as u64) as *mut u32;
    core::ptr::write_volatile(ptr, val);
}

/// Reads a 32-bit register from a specific SATA port within the AHCI MMIO space.
///
/// # Safety
///
/// The caller must ensure that `port` is valid (< 32) and `offset` is within the port's MMIO register space.
unsafe fn read_port(port: usize, offset: usize) -> u32 {
    let port_offset = PORT_BASE + port * PORT_SIZE + offset;
    read_abar(port_offset)
}

/// Writes a 32-bit value to a specific SATA port within the AHCI MMIO space.
///
/// # Safety
///
/// The caller must ensure that `port` is valid (< 32) and writing `val` does not violate hardware state.
unsafe fn write_port(port: usize, offset: usize, val: u32) {
    let port_offset = PORT_BASE + port * PORT_SIZE + offset;
    write_abar(port_offset, val);
}

/// Introduces a minor bus timing delay by writing to the diagnostic I/O port 0x80.
///
/// # Safety
///
/// Executes an `out` instruction to port 0x80, which is safe on IBM PC compatible architectures.
unsafe fn io_delay() {
    core::arch::asm!("out 0x80, al", in("al") 0u8);
}

/// Flushes CPU memory cache lines after AHCI SATA DMA transfer.
///
/// # Safety
///
/// Issues a hardware `mfence` memory barrier instruction to serialize store and load operations.
pub unsafe fn flush_dma_cache() {
    core::arch::asm!("mfence", options(nostack, preserves_flags));
}

/// Performs a SATA DMA transfer (read or write) on the specified port.
///
/// # Safety
///
/// Operates on direct hardware MMIO registers and physical DMA buffers.
pub unsafe fn sata_dma_transfer(port: usize, sector: u32, write: bool) -> Result<(), &'static str> {
    write_port(port, PORT_REG_IS, 0xFFFFFFFF);
    write_port(port, PORT_REG_SERR, 0xFFFFFFFF);

    let cmd_header = CLB_PHYS as *mut CmdHeader;
    let opts = 5 | if write { 1 << 6 } else { 0 };
    (*cmd_header).opts = opts;
    (*cmd_header).prdtl = 1;
    (*cmd_header).prdbc = 0;
    (*cmd_header).ctba = CTB_PHYS as u32;
    (*cmd_header).ctbau = (CTB_PHYS >> 32) as u32;
    for i in 0..4 {
        (*cmd_header).rsv1[i] = 0;
    }

    let cfis = CTB_PHYS as *mut u8;
    core::ptr::write_bytes(cfis, 0, 128);

    *cfis.add(0) = 0x27;
    *cfis.add(1) = 0x80;
    *cfis.add(2) = if write { 0x35 } else { 0x25 };

    *cfis.add(4) = (sector & 0xFF) as u8;
    *cfis.add(5) = ((sector >> 8) & 0xFF) as u8;
    *cfis.add(6) = ((sector >> 16) & 0xFF) as u8;
    *cfis.add(7) = 0x40;
    *cfis.add(8) = ((sector >> 24) & 0xFF) as u8;
    *cfis.add(9) = 0;
    *cfis.add(10) = 0;

    *cfis.add(12) = 1;
    *cfis.add(13) = 0;

    let prdt = (CTB_PHYS + 128) as *mut PrdtEntry;
    (*prdt).dba = SECTOR_BUF_PHYS as u32;
    (*prdt).dbau = (SECTOR_BUF_PHYS >> 32) as u32;
    (*prdt).rsv0 = 0;
    (*prdt).dbc = 511;

    let mut t = 1_000_000;
    while t > 0 {
        let tfd = read_port(port, 0x20);
        if (tfd & ((1 << 7) | (1 << 3))) == 0 {
            break;
        }
        io_delay();
        t -= 1;
    }
    if t == 0 {
        return Err("AHCI: Port busy timeout before transfer");
    }

    write_port(port, 0x38, 1);

    t = 1_000_000;
    while t > 0 {
        let ci = read_port(port, 0x38);
        if (ci & 1) == 0 {
            break;
        }

        let tfd = read_port(port, 0x20);
        if (tfd & (1 << 0)) != 0 {
            return Err("AHCI: SATA Task File Error during transfer");
        }

        io_delay();
        t -= 1;
    }
    if t == 0 {
        return Err("AHCI: SATA DMA transfer timeout");
    }

    let tfd = read_port(port, 0x20);
    if (tfd & (1 << 0)) != 0 {
        return Err("AHCI: SATA Task File Error post-transfer");
    }

    Ok(())
}

/// Sends ATA SYNCHRONIZE CACHE EXT command to flush SATA drive hardware write buffers.
///
/// # Safety
///
/// Issues hardware commands directly via AHCI command list.
pub unsafe fn sata_flush_cache(port: usize) -> Result<(), &'static str> {
    if !PORT_DMA_ALLOCATED {
        return Ok(());
    }

    write_port(port, PORT_REG_IS, 0xFFFFFFFF);
    write_port(port, PORT_REG_SERR, 0xFFFFFFFF);

    let cmd_header = CLB_PHYS as *mut CmdHeader;
    (*cmd_header).opts = 5;
    (*cmd_header).prdtl = 0;
    (*cmd_header).prdbc = 0;
    (*cmd_header).ctba = CTB_PHYS as u32;
    (*cmd_header).ctbau = (CTB_PHYS >> 32) as u32;
    for i in 0..4 {
        (*cmd_header).rsv1[i] = 0;
    }

    let cfis = CTB_PHYS as *mut u8;
    core::ptr::write_bytes(cfis, 0, 128);

    *cfis.add(0) = 0x27;
    *cfis.add(1) = 0x80;
    *cfis.add(2) = 0xEA;

    let mut t = 1_000_000;
    while t > 0 {
        let tfd = read_port(port, 0x20);
        if (tfd & ((1 << 7) | (1 << 3))) == 0 {
            break;
        }
        io_delay();
        t -= 1;
    }
    if t == 0 {
        return Err("AHCI: Port busy timeout before flush");
    }

    write_port(port, 0x38, 1);

    t = 1_000_000;
    while t > 0 {
        let ci = read_port(port, 0x38);
        if (ci & 1) == 0 {
            break;
        }

        let tfd = read_port(port, 0x20);
        if (tfd & (1 << 0)) != 0 {
            return Err("AHCI: SATA Task File Error during flush");
        }

        io_delay();
        t -= 1;
    }
    if t == 0 {
        return Err("AHCI: SATA cache flush timeout");
    }

    let tfd = read_port(port, 0x20);
    if (tfd & (1 << 0)) != 0 {
        return Err("AHCI: SATA Task File Error post-flush");
    }

    flush_dma_cache();
    Ok(())
}

/// Initializes the AHCI Controller and probes its ports.
pub fn init() -> Result<(), &'static str> {
    unsafe {
        let mut pci_dev = None;
        for i in 0..pci::PCI_DEVICE_COUNT {
            if let Some(dev) = pci::PCI_DEVICES[i] {
                if dev.class_code == 0x01 && dev.subclass == 0x06 {
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

        let abar_phys = dev.bar5 & 0xFFFF_F000;
        if abar_phys == 0 {
            return Err("AHCI: BAR5 is null");
        }

        vmm::map_page(abar_phys as u64, abar_phys as u64, vmm::PAGE_WRITABLE)?;
        vmm::map_page(
            (abar_phys + 0x1000) as u64,
            (abar_phys + 0x1000) as u64,
            vmm::PAGE_WRITABLE,
        )?;
        ABAR_VIRTUAL = abar_phys as u64;

        let mut ghc = read_abar(AHCI_REG_GHC);
        write_abar(AHCI_REG_GHC, ghc | GHC_AE);

        write_abar(AHCI_REG_GHC, read_abar(AHCI_REG_GHC) | GHC_HR);
        let mut timeout = 50_000;
        while (read_abar(AHCI_REG_GHC) & GHC_HR) != 0 {
            io_delay();
            timeout -= 1;
            if timeout == 0 {
                return Err("AHCI: HBA reset timeout");
            }
        }

        ghc = read_abar(AHCI_REG_GHC);
        write_abar(AHCI_REG_GHC, ghc | GHC_AE | GHC_IE);

        let pi = read_abar(AHCI_REG_PI);
        for port in 0..32 {
            if (pi & (1 << port)) != 0 {
                let mut det_timeout = 10_000;
                let mut det = 0;
                let mut ipm = 0;
                while det_timeout > 0 {
                    let ssts = read_port(port, PORT_REG_SSTS);
                    det = ssts & 0x0F;
                    ipm = (ssts >> 8) & 0x0F;
                    if det == 3 && ipm == 1 {
                        break;
                    }
                    io_delay();
                    det_timeout -= 1;
                }

                if det == 3 && ipm == 1 {
                    if !PORT_DMA_ALLOCATED {
                        let clb = pmm::alloc_frame().ok_or("AHCI: Failed to alloc CLB frame")?;
                        let fis = pmm::alloc_frame().ok_or("AHCI: Failed to alloc FIS frame")?;
                        let ctb = pmm::alloc_frame().ok_or("AHCI: Failed to alloc CTB frame")?;
                        let sbuf =
                            pmm::alloc_frame().ok_or("AHCI: Failed to alloc sector buffer")?;

                        vmm::map_page(clb, clb, vmm::PAGE_WRITABLE)?;
                        vmm::map_page(fis, fis, vmm::PAGE_WRITABLE)?;
                        vmm::map_page(ctb, ctb, vmm::PAGE_WRITABLE)?;
                        vmm::map_page(sbuf, sbuf, vmm::PAGE_WRITABLE)?;

                        CLB_PHYS = clb;
                        FIS_PHYS = fis;
                        CTB_PHYS = ctb;
                        SECTOR_BUF_PHYS = sbuf;
                        PORT_DMA_ALLOCATED = true;
                    }

                    let p = port;
                    let mut cmd_val = read_port(p, PORT_REG_CMD);
                    cmd_val &= !(1 << 0);
                    write_port(p, PORT_REG_CMD, cmd_val);

                    cmd_val &= !(1 << 4);
                    write_port(p, PORT_REG_CMD, cmd_val);

                    let mut t = 10_000;
                    while t > 0 {
                        let cur_cmd = read_port(p, PORT_REG_CMD);
                        if (cur_cmd & (1 << 15)) == 0 && (cur_cmd & (1 << 14)) == 0 {
                            break;
                        }
                        io_delay();
                        t -= 1;
                    }

                    core::ptr::write_bytes(CLB_PHYS as *mut u8, 0, 4096);
                    core::ptr::write_bytes(FIS_PHYS as *mut u8, 0, 4096);
                    core::ptr::write_bytes(CTB_PHYS as *mut u8, 0, 4096);
                    core::ptr::write_bytes(SECTOR_BUF_PHYS as *mut u8, 0, 4096);

                    write_port(p, PORT_REG_CLB, CLB_PHYS as u32);
                    write_port(p, 0x04, (CLB_PHYS >> 32) as u32);
                    write_port(p, PORT_REG_FB, FIS_PHYS as u32);
                    write_port(p, 0x0C, (FIS_PHYS >> 32) as u32);

                    write_port(p, PORT_REG_IS, 0xFFFFFFFF);
                    write_port(p, PORT_REG_SERR, 0xFFFFFFFF);

                    cmd_val = read_port(p, PORT_REG_CMD);
                    cmd_val |= 1 << 4;
                    write_port(p, PORT_REG_CMD, cmd_val);

                    cmd_val |= 1 << 0;
                    write_port(p, PORT_REG_CMD, cmd_val);

                    let mut t_sig = 50_000;
                    while t_sig > 0 {
                        io_delay();
                        t_sig -= 1;
                    }

                    let sig = read_port(p, PORT_REG_SIG);
                    if sig == AHCI_SIG_SATA {
                        let size_sectors = 20480;
                        AHCI_DEVICE = Some(AhciBlockDevice {
                            port_num: port,
                            size_sectors,
                        });

                        if let Some(ref dev_ref) = AHCI_DEVICE {
                            register_device(dev_ref)?;
                        }
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

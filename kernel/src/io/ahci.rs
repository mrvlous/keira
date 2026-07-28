// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: AHCI (SATA) Storage Driver
//!
//! Exposes and initializes the AHCI SATA storage controller,
//! maps the MMIO space (ABAR), probes ports, and registers SATA disks.

#![allow(dead_code)]

use crate::io::block::BlockDevice;
use crate::io::pci;
use crate::mem::vmm;

// AHCI Register Offsets
const AHCI_REG_CAP: usize = 0x00;
const AHCI_REG_GHC: usize = 0x04;
const AHCI_REG_IS: usize = 0x08;
const AHCI_REG_PI: usize = 0x0C;
const AHCI_REG_VS: usize = 0x10;

// AHCI GHC bits
// HBA Reset
const GHC_HR: u32 = 1 << 0;
// Interrupt Enable
const GHC_IE: u32 = 1 << 1;
// AHCI Enable
const GHC_AE: u32 = 1 << 31;

// Port Registers offset
const PORT_BASE: usize = 0x100;
const PORT_SIZE: usize = 0x80;

// Port Register Offsets inside each port
const PORT_REG_CLB: usize = 0x00;
const PORT_REG_FB: usize = 0x08;
const PORT_REG_IS: usize = 0x10;
const PORT_REG_IE: usize = 0x14;
const PORT_REG_CMD: usize = 0x18;
const PORT_REG_SIG: usize = 0x24;
const PORT_REG_SSTS: usize = 0x28;
const PORT_REG_SCTL: usize = 0x2C;
const PORT_REG_SERR: usize = 0x30;

// Signatures
const AHCI_SIG_SATA: u32 = 0x00000101;
const AHCI_SIG_SATAPI: u32 = 0xEB140101;

#[repr(C, packed)]
struct CmdHeader {
    opts: u16,
    prdtl: u16,
    prdbc: u32,
    ctba: u32,
    ctbau: u32,
    rsv1: [u32; 4],
}

#[repr(C, packed)]
struct PrdtEntry {
    dba: u32,
    dbau: u32,
    rsv0: u32,
    dbc: u32,
}

pub struct AhciBlockDevice {
    pub port_num: usize,
    pub size_sectors: u32,
}

impl BlockDevice for AhciBlockDevice {
    fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
        unsafe {
            sata_dma_transfer(self.port_num, sector, false)?;
            let src = SECTOR_BUF_PHYS as *const u8;
            core::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), 512);
            Ok(())
        }
    }

    fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
        unsafe {
            let dst = SECTOR_BUF_PHYS as *mut u8;
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), dst, 512);
            sata_dma_transfer(self.port_num, sector, true)?;
            Ok(())
        }
    }

    fn get_size_sectors(&self) -> u32 {
        self.size_sectors
    }

    fn get_name(&self) -> &'static str {
        "ahci0"
    }
}

pub static mut AHCI_DEVICE: Option<AhciBlockDevice> = None;
static mut ABAR_VIRTUAL: u64 = 0;

static mut CLB_PHYS: u64 = 0;
static mut FIS_PHYS: u64 = 0;
static mut CTB_PHYS: u64 = 0;
static mut SECTOR_BUF_PHYS: u64 = 0;
static mut PORT_DMA_ALLOCATED: bool = false;

unsafe fn read_abar(offset: usize) -> u32 {
    let ptr = (ABAR_VIRTUAL + offset as u64) as *const u32;
    core::ptr::read_volatile(ptr)
}

unsafe fn write_abar(offset: usize, val: u32) {
    let ptr = (ABAR_VIRTUAL + offset as u64) as *mut u32;
    core::ptr::write_volatile(ptr, val);
}

unsafe fn read_port(port: usize, offset: usize) -> u32 {
    let port_offset = PORT_BASE + port * PORT_SIZE + offset;
    read_abar(port_offset)
}

unsafe fn write_port(port: usize, offset: usize, val: u32) {
    let port_offset = PORT_BASE + port * PORT_SIZE + offset;
    write_abar(port_offset, val);
}

unsafe fn io_delay() {
    core::arch::asm!("out 0x80, al", in("al") 0u8);
}

unsafe fn sata_dma_transfer(port: usize, sector: u32, write: bool) -> Result<(), &'static str> {
    // 1. Clear interrupt status and error registers
    write_port(port, PORT_REG_IS, 0xFFFFFFFF);
    write_port(port, PORT_REG_SERR, 0xFFFFFFFF);

    // 2. Set up command header (slot 0)
    let cmd_header = CLB_PHYS as *mut CmdHeader;
    // CFIS size = 5 (20 bytes), Write bit
    let opts = 5 | if write { 1 << 6 } else { 0 };
    (*cmd_header).opts = opts;
    // 1 PRDT entry
    (*cmd_header).prdtl = 1;
    (*cmd_header).prdbc = 0;
    (*cmd_header).ctba = CTB_PHYS as u32;
    (*cmd_header).ctbau = (CTB_PHYS >> 32) as u32;
    for i in 0..4 {
        (*cmd_header).rsv1[i] = 0;
    }

    // 3. Set up command table
    let cfis = CTB_PHYS as *mut u8;
    // Clear CFIS area
    core::ptr::write_bytes(cfis, 0, 128);

    // Register H2D FIS
    *cfis.add(0) = 0x27;
    // Command bit set
    *cfis.add(1) = 0x80;
    // WRITE DMA EXT / READ DMA EXT
    *cfis.add(2) = if write { 0x35 } else { 0x25 };

    // Set LBA (LBA48: 6 bytes)
    *cfis.add(4) = (sector & 0xFF) as u8;
    *cfis.add(5) = ((sector >> 8) & 0xFF) as u8;
    *cfis.add(6) = ((sector >> 16) & 0xFF) as u8;
    // Device register (LBA mode)
    *cfis.add(7) = 0x40;
    *cfis.add(8) = ((sector >> 24) & 0xFF) as u8;
    *cfis.add(9) = 0;
    *cfis.add(10) = 0;

    // Sector count (1 sector)
    *cfis.add(12) = 1;
    *cfis.add(13) = 0;

    // 4. Set up PRDT (starts at offset 128 in CTB)
    let prdt = (CTB_PHYS + 128) as *mut PrdtEntry;
    (*prdt).dba = SECTOR_BUF_PHYS as u32;
    (*prdt).dbau = (SECTOR_BUF_PHYS >> 32) as u32;
    (*prdt).rsv0 = 0;
    // 512 bytes (dbc is 0-indexed byte count)
    (*prdt).dbc = 511;

    // 5. Wait for port to be ready (BSY and DRQ must be clear)
    let mut t = 1_000_000;
    while t > 0 {
        // PORT_REG_TFD
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

    // 6. Issue command
    // PORT_REG_CI (slot 0)
    write_port(port, 0x38, 1);

    // 7. Wait for command completion
    t = 1_000_000;
    while t > 0 {
        // PORT_REG_CI
        let ci = read_port(port, 0x38);
        if (ci & 1) == 0 {
            break;
        }

        // PORT_REG_TFD
        let tfd = read_port(port, 0x20);
        if (tfd & (1 << 0)) != 0 {
            // Error bit (bit 0)
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

/// Initialize the AHCI Controller and probe its ports
pub fn init() -> Result<(), &'static str> {
    unsafe {
        // 1. Locate SATA controller in PCI device list
        let mut pci_dev = None;
        for i in 0..pci::PCI_DEVICE_COUNT {
            if let Some(dev) = pci::PCI_DEVICES[i] {
                // Class 0x01 = Storage, Subclass 0x06 = SATA
                if dev.class_code == 0x01 && dev.subclass == 0x06 {
                    pci_dev = Some(dev);
                    break;
                }
            }
        }

        let dev = match pci_dev {
            Some(d) => d,
            None => {
                crate::io::serial::print_str("AHCI: No SATA controller found in PCI bus\n");
                return Ok(());
            }
        };

        // 2. Extract BAR5 and map it (BAR5 contains ABAR)
        let abar_phys = dev.bar5 & 0xFFFF_F000;
        if abar_phys == 0 {
            return Err("AHCI: BAR5 is null");
        }

        // Map two 4KB pages to cover HBA registers and ports
        vmm::map_page(abar_phys as u64, abar_phys as u64, vmm::PAGE_WRITABLE)?;
        vmm::map_page(
            (abar_phys + 0x1000) as u64,
            (abar_phys + 0x1000) as u64,
            vmm::PAGE_WRITABLE,
        )?;
        ABAR_VIRTUAL = abar_phys as u64;

        crate::io::serial::print_str("   - AHCI: Mapped ABAR controller memory space at physical ");
        crate::io::serial::print_hex(abar_phys as u64);
        crate::io::serial::print_str("\n");

        // 3. Initialize Controller
        // Enable AHCI mode
        let mut ghc = read_abar(AHCI_REG_GHC);
        write_abar(AHCI_REG_GHC, ghc | GHC_AE);

        // Perform HBA Reset with safe bare-metal delay
        write_abar(AHCI_REG_GHC, read_abar(AHCI_REG_GHC) | GHC_HR);
        let mut timeout = 50_000;
        while (read_abar(AHCI_REG_GHC) & GHC_HR) != 0 {
            io_delay();
            timeout -= 1;
            if timeout == 0 {
                return Err("AHCI: HBA reset timeout");
            }
        }

        // Enable AHCI & global interrupts
        ghc = read_abar(AHCI_REG_GHC);
        write_abar(AHCI_REG_GHC, ghc | GHC_AE | GHC_IE);

        // 4. Probe Ports
        let pi = read_abar(AHCI_REG_PI);
        for port in 0..32 {
            if (pi & (1 << port)) != 0 {
                // Wait for device detection (bare-metal link negotiation)
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
                    // Allocate physical frames for DMA command list & FIS area
                    if !PORT_DMA_ALLOCATED {
                        let clb = crate::mem::pmm::alloc_frame()
                            .ok_or("AHCI: Failed to alloc CLB frame")?;
                        let fis = crate::mem::pmm::alloc_frame()
                            .ok_or("AHCI: Failed to alloc FIS frame")?;
                        let ctb = crate::mem::pmm::alloc_frame()
                            .ok_or("AHCI: Failed to alloc CTB frame")?;
                        let sbuf = crate::mem::pmm::alloc_frame()
                            .ok_or("AHCI: Failed to alloc sector buffer")?;

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

                    // Stop port command engine
                    let p = port;
                    let mut cmd_val = read_port(p, PORT_REG_CMD);
                    // ST
                    cmd_val &= !(1 << 0);
                    // FRE
                    cmd_val &= !(1 << 4);
                    write_port(p, PORT_REG_CMD, cmd_val);

                    // Wait for port ST and FRE engines to stop running
                    let mut t = 10_000;
                    while t > 0 {
                        let cur_cmd = read_port(p, PORT_REG_CMD);
                        if (cur_cmd & (1 << 15)) == 0 && (cur_cmd & (1 << 14)) == 0 {
                            break;
                        }
                        io_delay();
                        t -= 1;
                    }

                    // Clear memory descriptors
                    core::ptr::write_bytes(CLB_PHYS as *mut u8, 0, 4096);
                    core::ptr::write_bytes(FIS_PHYS as *mut u8, 0, 4096);
                    core::ptr::write_bytes(CTB_PHYS as *mut u8, 0, 4096);
                    core::ptr::write_bytes(SECTOR_BUF_PHYS as *mut u8, 0, 4096);

                    // Set physical addresses
                    write_port(p, PORT_REG_CLB, CLB_PHYS as u32);
                    // PORT_REG_CLBU
                    write_port(p, 0x04, (CLB_PHYS >> 32) as u32);
                    write_port(p, PORT_REG_FB, FIS_PHYS as u32);
                    // PORT_REG_FBU
                    write_port(p, 0x0C, (FIS_PHYS >> 32) as u32);

                    // Clear port interrupts & errors
                    write_port(p, PORT_REG_IS, 0xFFFFFFFF);
                    write_port(p, PORT_REG_SERR, 0xFFFFFFFF);

                    // Start port command engine
                    cmd_val = read_port(p, PORT_REG_CMD);
                    // FRE
                    cmd_val |= 1 << 4;
                    write_port(p, PORT_REG_CMD, cmd_val);

                    // ST
                    cmd_val |= 1 << 0;
                    write_port(p, PORT_REG_CMD, cmd_val);

                    // Wait a bit for the signature to stabilize
                    let mut t_sig = 50_000;
                    while t_sig > 0 {
                        io_delay();
                        t_sig -= 1;
                    }

                    let sig = read_port(p, PORT_REG_SIG);
                    if sig == AHCI_SIG_SATA {
                        crate::io::serial::print_str(
                            "   - AHCI: Detected active SATA Hard Disk drive on Port ",
                        );
                        crate::io::serial::print_u64(port as u64);
                        crate::io::serial::print_str("\n");

                        // Register SATA Block Device (size = 10MB as safe default / 20480 sectors)
                        let size_sectors = 20480;
                        AHCI_DEVICE = Some(AhciBlockDevice {
                            port_num: port,
                            size_sectors,
                        });

                        if let Some(ref dev_ref) = AHCI_DEVICE {
                            crate::io::block::register_device(dev_ref)?;
                        }
                        // Register first detected disk
                        break;
                    } else if sig == AHCI_SIG_SATAPI {
                        crate::io::serial::print_str("AHCI: CD-ROM (SATAPI) detected on Port ");
                        crate::io::serial::print_u64(port as u64);
                        crate::io::serial::print_str("\n");
                    }
                }
            }
        }
    }

    Ok(())
}

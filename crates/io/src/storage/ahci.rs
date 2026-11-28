// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Advanced Host Controller Interface (AHCI) Serial ATA (SATA) disk driver.

pub mod device {
    use super::driver::{flush_dma_cache, sata_dma_transfer_slot, sata_flush_cache_slot, PORT_DMA};
    use crate::storage::block::BlockDevice;

    pub struct AhciBlockDevice {
        /// Driver slot index (`0` for `ahci0`, `1` for `ahci1`).
        pub dev_index: usize,
        /// Controller port index attached to this drive.
        pub port_num: usize,
        /// Total capacity in 512-byte sectors.
        pub size_sectors: u32,
    }

    impl BlockDevice for AhciBlockDevice {
        fn read_sector(&self, sector: u32, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
            unsafe {
                sata_dma_transfer_slot(self.dev_index, self.port_num, sector, false)?;
                let src = PORT_DMA[self.dev_index].sbuf as *const u8;
                core::ptr::copy_nonoverlapping(src, buffer.as_mut_ptr(), 512);
                Ok(())
            }
        }

        fn write_sector(&self, sector: u32, buffer: &[u8; 512]) -> Result<(), &'static str> {
            unsafe {
                let dst = PORT_DMA[self.dev_index].sbuf as *mut u8;
                core::ptr::copy_nonoverlapping(buffer.as_ptr(), dst, 512);
                sata_dma_transfer_slot(self.dev_index, self.port_num, sector, true)?;
                flush_dma_cache();
                Ok(())
            }
        }

        fn get_size_sectors(&self) -> u32 {
            self.size_sectors
        }

        fn get_name(&self) -> &'static str {
            if self.dev_index == 0 {
                "ahci0"
            } else {
                "ahci1"
            }
        }

        fn flush(&self) -> Result<(), &'static str> {
            unsafe { sata_flush_cache_slot(self.dev_index, self.port_num) }
        }
    }

    pub static mut AHCI_DEVICE: Option<AhciBlockDevice> = None;
    pub static mut AHCI_SECONDARY_DEVICE: Option<AhciBlockDevice> = None;
}

pub mod driver {
    use super::device::{AhciBlockDevice, AHCI_DEVICE, AHCI_SECONDARY_DEVICE};
    use super::registers::*;
    use crate::bus::pci;
    use crate::storage::block::register_device;
    use keira_mem::{pmm, vmm};

    #[derive(Copy, Clone)]
    pub struct AhciPortDma {
        pub allocated: bool,
        pub clb: u64,
        pub fis: u64,
        pub ctb: u64,
        pub sbuf: u64,
    }

    impl AhciPortDma {
        /// Create an unallocated AHCI DMA frame descriptor tuple.
        pub const fn empty() -> Self {
            Self {
                allocated: false,
                clb: 0,
                fis: 0,
                ctb: 0,
                sbuf: 0,
            }
        }
    }

    pub(crate) static mut ABAR_VIRTUAL: u64 = 0;
    pub(crate) static mut PORT_DMA: [AhciPortDma; 2] = [AhciPortDma::empty(), AhciPortDma::empty()];

    /// Reads a 32-bit register from the AHCI Host Bus Adapter MMIO space at `offset`.
    ///
    /// # Safety
    /// `ABAR_VIRTUAL` must hold a valid mapped AHCI MMIO base address and
    /// `ABAR_VIRTUAL + offset` must be 4-byte aligned within the mapped HBA register region.
    unsafe fn read_abar(offset: usize) -> u32 {
        let ptr = (ABAR_VIRTUAL + offset as u64) as *const u32;
        core::ptr::read_volatile(ptr)
    }

    /// Writes a 32-bit value to the AHCI Host Bus Adapter MMIO space at `offset`.
    ///
    /// # Safety
    /// `ABAR_VIRTUAL` must hold a valid writable mapped AHCI MMIO base address and
    /// `ABAR_VIRTUAL + offset` must be 4-byte aligned within the mapped HBA register region.
    unsafe fn write_abar(offset: usize, val: u32) {
        let ptr = (ABAR_VIRTUAL + offset as u64) as *mut u32;
        core::ptr::write_volatile(ptr, val);
    }

    /// Reads a 32-bit register for AHCI `port` at byte `offset`.
    ///
    /// # Safety
    /// `port` must be an implemented port index (< 32) and the computed port register offset
    /// must satisfy the alignment and mapping preconditions of [`read_abar`].
    unsafe fn read_port(port: usize, offset: usize) -> u32 {
        let port_offset = PORT_BASE + port * PORT_SIZE + offset;
        read_abar(port_offset)
    }

    /// Writes a 32-bit value to a register for AHCI `port` at byte `offset`.
    ///
    /// # Safety
    /// `port` must be an implemented port index (< 32) and the computed port register offset
    /// must satisfy the alignment and mapping preconditions of [`write_abar`].
    unsafe fn write_port(port: usize, offset: usize, val: u32) {
        let port_offset = PORT_BASE + port * PORT_SIZE + offset;
        write_abar(port_offset, val);
    }

    /// Performs a brief bus delay by writing to diagnostic port `0x80`.
    ///
    /// # Safety
    /// Caller must execute in Ring 0 (CPL 0) on an x86 platform where port `0x80` is an
    /// unused POST diagnostic port.
    unsafe fn io_delay() {
        core::arch::asm!("out 0x80, al", in("al") 0u8);
    }

    /// Executes an `mfence` memory barrier to order DMA buffer writes and MMIO accesses.
    ///
    /// # Safety
    /// Caller must execute on an x86 CPU supporting SSE2 (`mfence`).
    pub unsafe fn flush_dma_cache() {
        core::arch::asm!("mfence", options(nostack, preserves_flags));
    }

    /// Executes a single-sector SATA DMA read or write command on the specified AHCI slot and port.
    ///
    /// # Safety
    /// `PORT_DMA[slot]` must be allocated and mapped and `port` must be an active initialized SATA
    /// port (< 32). Callers must serialize concurrent transfers on the same `slot`.
    pub unsafe fn sata_dma_transfer_slot(
        slot: usize,
        port: usize,
        sector: u32,
        write: bool,
    ) -> Result<(), &'static str> {
        let dma = (*core::ptr::addr_of!(PORT_DMA))
            .get(slot)
            .copied()
            .ok_or("AHCI: Invalid slot index")?;
        if !dma.allocated {
            return Err("AHCI: Port DMA not allocated");
        }

        write_port(port, PORT_REG_IS, 0xFFFFFFFF);
        write_port(port, PORT_REG_SERR, 0xFFFFFFFF);

        let cmd_header = dma.clb as *mut CmdHeader;
        let opts = 5 | if write { 1 << 6 } else { 0 };
        (*cmd_header).opts = opts;
        (*cmd_header).prdtl = 1;
        (*cmd_header).prdbc = 0;
        (*cmd_header).ctba = dma.ctb as u32;
        (*cmd_header).ctbau = (dma.ctb >> 32) as u32;
        for i in 0..4 {
            (*cmd_header).rsv1[i] = 0;
        }

        let cfis = dma.ctb as *mut u8;
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

        let prdt = (dma.ctb + 128) as *mut PrdtEntry;
        (*prdt).dba = dma.sbuf as u32;
        (*prdt).dbau = (dma.sbuf >> 32) as u32;
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

    /// Executes a single-sector SATA DMA transfer on primary slot 0.
    ///
    /// # Safety
    /// Satisfies all safety preconditions of [`sata_dma_transfer_slot`].
    pub unsafe fn sata_dma_transfer(
        port: usize,
        sector: u32,
        write: bool,
    ) -> Result<(), &'static str> {
        sata_dma_transfer_slot(0, port, sector, write)
    }

    /// Issues an ATA `FLUSH CACHE` command on the specified AHCI slot and port.
    ///
    /// # Safety
    /// `PORT_DMA[slot]` must be allocated and mapped and `port` must be an active SATA port (< 32).
    pub unsafe fn sata_flush_cache_slot(slot: usize, port: usize) -> Result<(), &'static str> {
        let dma = match (*core::ptr::addr_of!(PORT_DMA)).get(slot) {
            Some(d) if d.allocated => *d,
            _ => return Ok(()),
        };

        write_port(port, PORT_REG_IS, 0xFFFFFFFF);
        write_port(port, PORT_REG_SERR, 0xFFFFFFFF);

        let cmd_header = dma.clb as *mut CmdHeader;
        (*cmd_header).opts = 5;
        (*cmd_header).prdtl = 0;
        (*cmd_header).prdbc = 0;
        (*cmd_header).ctba = dma.ctb as u32;
        (*cmd_header).ctbau = (dma.ctb >> 32) as u32;
        for i in 0..4 {
            (*cmd_header).rsv1[i] = 0;
        }

        let cfis = dma.ctb as *mut u8;
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

    /// Issues an ATA `FLUSH CACHE` command on primary slot 0.
    ///
    /// # Safety
    /// Satisfies all safety preconditions of [`sata_flush_cache_slot`].
    pub unsafe fn sata_flush_cache(port: usize) -> Result<(), &'static str> {
        sata_flush_cache_slot(0, port)
    }

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
            let mut next_slot = 0usize;
            for port in 0..32 {
                if next_slot >= (&*core::ptr::addr_of!(PORT_DMA)).len() {
                    break;
                }
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
                        if !PORT_DMA[next_slot].allocated {
                            let clb =
                                pmm::alloc_frame().ok_or("AHCI: Failed to alloc CLB frame")?;
                            let fis =
                                pmm::alloc_frame().ok_or("AHCI: Failed to alloc FIS frame")?;
                            let ctb =
                                pmm::alloc_frame().ok_or("AHCI: Failed to alloc CTB frame")?;
                            let sbuf =
                                pmm::alloc_frame().ok_or("AHCI: Failed to alloc sector buffer")?;

                            vmm::map_page(clb, clb, vmm::PAGE_WRITABLE)?;
                            vmm::map_page(fis, fis, vmm::PAGE_WRITABLE)?;
                            vmm::map_page(ctb, ctb, vmm::PAGE_WRITABLE)?;
                            vmm::map_page(sbuf, sbuf, vmm::PAGE_WRITABLE)?;

                            PORT_DMA[next_slot] = AhciPortDma {
                                allocated: true,
                                clb,
                                fis,
                                ctb,
                                sbuf,
                            };
                        }

                        let dma = PORT_DMA[next_slot];
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

                        core::ptr::write_bytes(dma.clb as *mut u8, 0, 4096);
                        core::ptr::write_bytes(dma.fis as *mut u8, 0, 4096);
                        core::ptr::write_bytes(dma.ctb as *mut u8, 0, 4096);
                        core::ptr::write_bytes(dma.sbuf as *mut u8, 0, 4096);

                        write_port(p, PORT_REG_CLB, dma.clb as u32);
                        write_port(p, 0x04, (dma.clb >> 32) as u32);
                        write_port(p, PORT_REG_FB, dma.fis as u32);
                        write_port(p, 0x0C, (dma.fis >> 32) as u32);

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
                            let size_sectors = 65536;
                            if next_slot == 0 {
                                AHCI_DEVICE = Some(AhciBlockDevice {
                                    dev_index: 0,
                                    port_num: port,
                                    size_sectors,
                                });
                                if let Some(ref dev_ref) = AHCI_DEVICE {
                                    register_device(dev_ref)?;
                                }
                            } else {
                                AHCI_SECONDARY_DEVICE = Some(AhciBlockDevice {
                                    dev_index: 1,
                                    port_num: port,
                                    size_sectors,
                                });
                                if let Some(ref dev_ref) = AHCI_SECONDARY_DEVICE {
                                    register_device(dev_ref)?;
                                }
                            }
                            next_slot += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub mod registers {
    pub const AHCI_REG_GHC: usize = 0x04;
    pub const AHCI_REG_PI: usize = 0x0C;

    pub const GHC_HR: u32 = 1 << 0;
    pub const GHC_IE: u32 = 1 << 1;
    pub const GHC_AE: u32 = 1 << 31;

    pub const PORT_BASE: usize = 0x100;
    pub const PORT_SIZE: usize = 0x80;

    pub const PORT_REG_CLB: usize = 0x00;
    pub const PORT_REG_FB: usize = 0x08;
    pub const PORT_REG_IS: usize = 0x10;
    pub const PORT_REG_CMD: usize = 0x18;
    pub const PORT_REG_SIG: usize = 0x24;
    pub const PORT_REG_SSTS: usize = 0x28;
    pub const PORT_REG_SERR: usize = 0x30;

    pub const AHCI_SIG_SATA: u32 = 0x00000101;
    pub const AHCI_SIG_SATAPI: u32 = 0xEB140101;

    #[repr(C, packed)]
    pub struct CmdHeader {
        pub opts: u16,
        pub prdtl: u16,
        pub prdbc: u32,
        pub ctba: u32,
        pub ctbau: u32,
        pub rsv1: [u32; 4],
    }

    #[repr(C, packed)]
    pub struct PrdtEntry {
        pub dba: u32,
        pub dbau: u32,
        pub rsv0: u32,
        pub dbc: u32,
    }
}

pub use self::device::*;
pub use self::driver::*;
pub use self::registers::*;

#[cfg(test)]
mod tests {
    use super::registers::{CmdHeader, PrdtEntry, AHCI_SIG_SATA, AHCI_SIG_SATAPI};

    #[test]
    fn test_ahci_signatures() {
        assert_eq!(AHCI_SIG_SATA, 0x0000_0101);
        assert_eq!(AHCI_SIG_SATAPI, 0xEB14_0101);
    }

    #[test]
    fn test_ahci_struct_sizes() {
        assert_eq!(core::mem::size_of::<CmdHeader>(), 32);
        assert_eq!(core::mem::size_of::<PrdtEntry>(), 16);
    }
}

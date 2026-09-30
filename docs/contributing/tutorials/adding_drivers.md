<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tutorial: Developing Hardware Device Drivers

This tutorial guides developers through writing, initializing, and registering hardware device drivers in Keira Kernel, covering PCI bus discovery, Memory-Mapped I/O (MMIO), DMA allocation, and interrupt routing.

---

## 1. Hardware Abstraction Layers

Keira structures drivers into specialized crates and abstraction layers:

```mermaid
graph TD
    PCI["PCI Bus Scan (keira_io::bus::pci)"] --> Discovery["Vendor & Device ID Match"]
    Discovery --> Init["Driver Initialization Routine"]
    Init --> MMIO["VMM Identity / Virtual Page Mapping"]
    Init --> DMA["PMM Contiguous Physical DMA Pool"]
    Init --> IRQ["IDT Interrupt Gate & APIC/PIC Routing"]
    Init --> Subsystem["Subsystem Trait Registration (Block, Net, Char)"]
```

---

## 2. Step 1: PCI Device Probing

PCI devices report standard configuration headers. Identify your hardware:
* **Vendor ID**: 16-bit identifier (e.g. `0x8086` for Intel, `0x10EC` for Realtek).
* **Device ID**: 16-bit hardware identifier.
* **Base Address Registers (BARs)**: Indicate port I/O ranges or physical MMIO addresses.

In `crates/io/src/bus/pci/scanner.rs`:
```rust
for dev in pci::scan_bus() {
    if dev.vendor_id == 0x8086 && dev.device_id == 0x100E {
        // Intel 82540EM Gigabit Ethernet NIC detected
        e1000::init(dev);
    }
}
```

---

## 3. Step 2: Mapping MMIO & Allocating DMA

Device registers are mapped into kernel virtual address space:

```rust
use keira_mem::dma::alloc_dma_buffer;
use keira_mem::vmm::mapping::map_page;

pub struct MyDriver {
    mmio_base: u64,
    dma_buffer: u64,
    dma_phys: u64,
}

impl MyDriver {
    pub fn new(bar0_phys: u64) -> Result<Self, &'static str> {
        // 1. Map physical MMIO frame with Cache-Disable flags
        let mmio_virt = bar0_phys;
        map_page(mmio_virt, bar0_phys, true)?;

        // 2. Allocate physically contiguous 32-bit DMA ring buffer
        let dma = alloc_dma_buffer(8192)?;

        Ok(Self {
            mmio_base: mmio_virt,
            dma_buffer: dma.vaddr,
            dma_phys: dma.paddr,
        })
    }
}
```

---

## 4. Step 3: Registering with Subsystem

If writing a block storage driver (e.g. NVMe, AHCI, Ramdisk), implement `BlockDevice`:

```rust
use keira_io::storage::block::{BlockDevice, register_block_device};

impl BlockDevice for MyDriver {
    fn read_sector(&mut self, lba: u64, buf: &mut [u8; 512]) -> Result<(), &'static str> {
        // Device-specific DMA command dispatch
        Ok(())
    }

    fn write_sector(&mut self, lba: u64, buf: &[u8; 512]) -> Result<(), &'static str> {
        Ok(())
    }
}
```

Register the driver instance:
```rust
register_block_device("hd0", Box::new(driver));
```

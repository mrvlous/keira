<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Tutorial: Developing Hardware Drivers

Step-by-step guide for implementing new hardware drivers in `keira-io`.

## Step 1: Identify PCI IDs
Locate the PCI Vendor ID, Device ID, Class Code, and Subclass for your hardware device.

## Step 2: Implement Driver Module
Create a new driver under `crates/io/src/storage/`, `crates/io/src/sound/`, or `crates/io/src/bus/`.
- Setup MMIO register bases.
- Allocate physically contiguous DMA buffers via `keira_mem::dma::alloc_dma_buffer()`.
- Register the interrupt vector with the IDT/APIC.

## Step 3: Register with Block / Character Layer
If it is a block storage device, implement the `BlockDevice` trait and register it via `keira_io::storage::block::register_device()`.

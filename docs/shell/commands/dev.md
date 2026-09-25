<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Device & Hardware Driver Commands

The `dev` command suite provides bus enumeration, driver registry inspection, and storage controller diagnostics.

---

## Command Reference

| Command | Syntax | Description | Flags / Options |
| :--- | :--- | :--- | :--- |
| `devices` | `devices` | Enumerate PCI bus hierarchy, vendor IDs, and device classes | `-h, --help` |
| `drivers` | `drivers` | Display active kernel driver registry and hardware interfaces | `-h, --help` |
| `framebuffer` | `framebuffer` | Query VESA framebuffer resolution, pitch, and pixel format | `-h, --help` |
| `kvm` | `kvm` | Inspect hardware virtualization extensions and guest VM states | `-h, --help` |
| `lkm` | `lkm` | Display loaded kernel modules (LKM), symbol table, and state | `-h, --help` |
| `nvme` | `nvme` | Inspect NVMe PCIe controller queues and active namespaces | `-h, --help` |
| `tpm` | `tpm` | Query TPM 2.0 hardware security enclave and PCR bank states | `-h, --help` |
| `usb` | `usb` | Scan xHCI USB controller, root hubs, and attached endpoints | `-h, --help` |

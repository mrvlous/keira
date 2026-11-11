// SPDX-License-Identifier: GPL-2.0-only

//! System buses, PCI device scanner and PCIe ECAM / MSI configurations.

pub mod pci;
pub mod pcie;

pub use self::pci::*;
pub use self::pcie as pci_express;

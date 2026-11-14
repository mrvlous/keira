// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Hardware devices and peripheral driver shell commands.

pub mod hardware {
    pub mod devices {
        use keira_io::vga;

        fn print_hex_u16(val: u16) {
            let chars = b"0123456789ABCDEF";
            let mut buf = [0u8; 4];
            buf[0] = chars[((val >> 12) & 0xF) as usize];
            buf[1] = chars[((val >> 8) & 0xF) as usize];
            buf[2] = chars[((val >> 4) & 0xF) as usize];
            buf[3] = chars[(val & 0xF) as usize];
            if let Ok(s) = core::str::from_utf8(&buf) {
                vga::print_str(s);
            }
        }

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            unsafe {
                if let Some("-h") | Some("--help") = parts.next() {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: devices [-h]\n\n");
                    vga::print_str("Description:\n  Scan PCI bus slots and enumerate detected hardware devices, vendor IDs, device IDs and class types.\n\n");
                    vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }

                // Initialize/rescan the PCI bus
                keira_io::pci::init();

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("BUS   SLOT  FUNC  VENDOR  DEVICE  CLASS TYPE\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                for i in 0..keira_io::pci::PCI_DEVICE_COUNT {
                    if let Some(dev) = keira_io::pci::PCI_DEVICES[i] {
                        // Print bus
                        vga::print_u64(dev.bus as u64);
                        vga::print_str("     ");

                        // Print slot
                        vga::print_u64(dev.slot as u64);
                        if dev.slot < 10 {
                            vga::print_str("     ");
                        } else {
                            vga::print_str("    ");
                        }

                        // Print func
                        vga::print_u64(dev.func as u64);
                        vga::print_str("     ");

                        // Print Vendor ID
                        print_hex_u16(dev.vendor_id);
                        vga::print_str("    ");

                        // Print Device ID
                        print_hex_u16(dev.device_id);
                        vga::print_str("    ");

                        // Print Class Description
                        let class_str =
                            keira_io::pci::pci_class_to_str(dev.class_code, dev.subclass);
                        vga::print_str(class_str);
                        vga::print_str("\n");
                    }
                }
            }
        }
    }
    pub use self::devices::*;
}

pub mod virt {
    pub mod drivers {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: drivers [-h]\n\n");
                vga::print_str(
                    "Description:\n  Query active C and Rust hardware device drivers.\n\n",
                );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            unsafe {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Active Kernel Hardware Drivers:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  [C Driver] Serial UART COM1 (115200 8N1)\n");
                vga::print_str("  [C Driver] VGA Text Mode (80x25 / 80x50)\n");

                vga::print_str("  [C Driver] Intel e1000 Gigabit Ethernet NIC\n");
                vga::print_str("  [C Driver] PS/2 Keyboard & Mouse Controller\n");
                vga::print_str("  [C Driver] CMOS Real Time Clock (RTC)\n");
                vga::print_str("  [C Driver] Intel High Definition Audio (HDA) DSP\n");
                vga::print_str("  [Rust Driver] NVMe PCIe Controller & Storage Host\n");
                vga::print_str("  [Rust Driver] USB 3.0 xHCI Host Controller & Mass Storage\n");
                vga::print_str("  [PCI Bus] Devices Detected: ");
                vga::print_u64(keira_io::pci::PCI_DEVICE_COUNT as u64);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
    pub use self::drivers::*;
}

pub use self::hardware::devices;
pub use self::virt::drivers;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_commands_help_invocation() {
        let mut args = "devices --help".split_whitespace();
        args.next();
        devices::run(&mut args);

        let mut args = "drivers --help".split_whitespace();
        args.next();
        drivers::run(&mut args);
    }
}

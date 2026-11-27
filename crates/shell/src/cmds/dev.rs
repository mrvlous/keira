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

pub mod modules {
    pub mod lsmod {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: lsmod [-h]\n\n");
                vga::print_str(
                    "Description:\n  Show status of loaded kernel modules in memory.\n\n",
                );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let (mods, count) = keira_core::module::get_modules_snapshot();
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Module                  Size  Used by  State     Address\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            for m_opt in mods.iter().flatten() {
                let name = m_opt.name_str();
                vga::print_str(name);
                for _ in 0..24usize.saturating_sub(name.len()) {
                    vga::print_str(" ");
                }

                vga::print_u64(m_opt.size as u64);
                let sz_len = if m_opt.size >= 100000 {
                    6
                } else if m_opt.size >= 10000 {
                    5
                } else if m_opt.size >= 1000 {
                    4
                } else {
                    3
                };
                for _ in 0..10usize.saturating_sub(sz_len) {
                    vga::print_str(" ");
                }

                vga::print_u64(m_opt.ref_count as u64);
                vga::print_str("        ");

                vga::print_str(m_opt.state.as_str());
                for _ in 0..10usize.saturating_sub(m_opt.state.as_str().len()) {
                    vga::print_str(" ");
                }

                vga::print_hex(m_opt.load_address);
                vga::print_str("\n");
            }
            vga::print_str("\nTotal loaded modules: ");
            vga::print_u64(count as u64);
            vga::print_str("\n");
        }
    }

    pub mod insmod {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let arg1 = parts.next();
            if arg1.is_none() || matches!(arg1, Some("-h") | Some("--help")) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: insmod <file.ko | name> [size]\n\n");
                vga::print_str(
                    "Description:\n  Insert an ELF relocatable module or dynamic driver into the running kernel.\n\n",
                );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let target = arg1.unwrap();

            // Attempt reading and dynamically linking as an ELF .ko object if file exists or ends with .ko
            if target.ends_with(".ko") || keira_fs::vfs::exists(target) {
                let mut file_buf = [0u8; 16384];
                if let Ok(bytes_read) = keira_fs::vfs::read_file_offset(target, 0, &mut file_buf) {
                    if bytes_read >= 16 && file_buf[0..4] == [0x7F, b'E', b'L', b'F'] {
                        let mod_name = target
                            .rsplit('/')
                            .next()
                            .unwrap_or(target)
                            .trim_end_matches(".ko");
                        match keira_core::module::load_elf_module(
                            mod_name,
                            &file_buf[..bytes_read],
                            true,
                        ) {
                            Ok(loaded) => {
                                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                vga::print_str("[kernel] ELF module '");
                                vga::print_str(loaded.name_str());
                                vga::print_str("' linked successfully (addr: ");
                                vga::print_hex(loaded.load_address);
                                vga::print_str(", size: ");
                                vga::print_u64(loaded.size as u64);
                                vga::print_str(" bytes)\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                return;
                            }
                            Err(e) => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("insmod: failed to link ELF module '");
                                vga::print_str(target);
                                vga::print_str("': ");
                                vga::print_str(e.as_str());
                                vga::print_str("\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                                return;
                            }
                        }
                    }
                }
            }

            // Fallback to name-based module registration
            let mod_name = target;
            let mod_size = parts
                .next()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(32768);
            let load_addr = 0xFFFF_8000_0050_0000u64 + ((mod_size as u64) & 0x000F_FFFF);

            match keira_core::module::register_module(
                mod_name,
                mod_size,
                load_addr,
                "Dynamic Loadable Kernel Module",
            ) {
                Ok(idx) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[kernel] Module '");
                    vga::print_str(mod_name);
                    vga::print_str("' loaded successfully at slot ");
                    vga::print_u64(idx as u64);
                    vga::print_str(" (addr: ");
                    vga::print_hex(load_addr);
                    vga::print_str(")\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error loading module '");
                    vga::print_str(mod_name);
                    vga::print_str("': ");
                    vga::print_str(e.as_str());
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }
    }

    pub mod rmmod {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let arg1 = parts.next();
            if arg1.is_none() || matches!(arg1, Some("-h") | Some("--help")) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: rmmod <name>\n\n");
                vga::print_str("Description:\n  Unload a module from the running kernel.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let mod_name = arg1.unwrap();

            // Try dynamic ELF unloading first, fallback to registry unregister
            let unload_res = if keira_core::module::unload_elf_module(mod_name).is_ok() {
                Ok(())
            } else {
                keira_core::module::unregister_module(mod_name)
            };

            match unload_res {
                Ok(()) => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[kernel] Module '");
                    vga::print_str(mod_name);
                    vga::print_str("' unloaded successfully\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error unloading module '");
                    vga::print_str(mod_name);
                    vga::print_str("': ");
                    vga::print_str(e.as_str());
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
            }
        }
    }

    pub mod modinfo {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let arg1 = parts.next();
            if arg1.is_none() || matches!(arg1, Some("-h") | Some("--help")) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: modinfo <name>\n\n");
                vga::print_str(
                    "Description:\n  Show information about a Linux-grade kernel module.\n\n",
                );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let mod_name = arg1.unwrap();
            if let Some(m) = keira_core::module::get_module(mod_name) {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("filename:       /lib/modules/");
                vga::print_str(m.name_str());
                vga::print_str(".ko\n");
                vga::print_str("description:    ");
                vga::print_str(m.description);
                vga::print_str(
                    "\nauthor:         Moh. Ananda Firmansyah Putra <mrvlous@proton.me>\n",
                );
                vga::print_str("license:        GPL-2.0-only\n");
                vga::print_str("size:           ");
                vga::print_u64(m.size as u64);
                vga::print_str(" bytes\n");
                vga::print_str("refcount:       ");
                vga::print_u64(m.ref_count as u64);
                vga::print_str("\nstate:          ");
                vga::print_str(m.state.as_str());
                vga::print_str("\nload_address:   ");
                vga::print_hex(m.load_address);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            } else {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("modinfo: ERROR: Module ");
                vga::print_str(mod_name);
                vga::print_str(" not found.\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            }
        }
    }
}

pub use self::hardware::devices;
pub use self::modules::{insmod, lsmod, modinfo, rmmod};
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

        let mut args = "lsmod --help".split_whitespace();
        args.next();
        lsmod::run(&mut args);

        let mut args = "insmod --help".split_whitespace();
        args.next();
        insmod::run(&mut args);

        let mut args = "rmmod --help".split_whitespace();
        args.next();
        rmmod::run(&mut args);

        let mut args = "modinfo --help".split_whitespace();
        args.next();
        modinfo::run(&mut args);
    }

    #[test]
    fn test_modules_commands_execution() {
        let mut args = "lsmod".split_whitespace();
        args.next();
        lsmod::run(&mut args);

        let mut args = "modinfo ext4_fs".split_whitespace();
        args.next();
        modinfo::run(&mut args);

        let mut args = "insmod test_mod 16384".split_whitespace();
        args.next();
        insmod::run(&mut args);

        let mut args = "modinfo test_mod".split_whitespace();
        args.next();
        modinfo::run(&mut args);

        let mut args = "rmmod test_mod".split_whitespace();
        args.next();
        rmmod::run(&mut args);
    }
}

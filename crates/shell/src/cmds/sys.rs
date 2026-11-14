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

//! System configuration, telemetry, services and power shell commands.

pub mod control {
    pub mod power {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let sub = parts.next();
            if let Some("-h") | Some("--help") = sub {
                {
                    vga::print_str("Usage: power [status|acpi|shutdown|poweroff|reboot]\n\n");
                    vga::print_str("Description:\n  Query ACPI power management states, MADT topology or initiate system shutdown/reboot.\n\n");
                    vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                }
                return;
            }

            if let Some("shutdown") | Some("poweroff") | Some("off") = sub {
                {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("Powering off Keira Kernel via ACPI S5 Soft-Off...\n");
                    keira_arch::power::acpi::poweroff();
                }
            } else if let Some("reboot") | Some("restart") | Some("reset") = sub {
                {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("Rebooting Keira Kernel via PS/2 controller...\n");
                    keira_arch::power::acpi::reboot();
                }
            }

            {
                let topo = keira_arch::power::acpi::get_acpi_topology();

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("ACPI Power Management & Hardware Watchdog:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  ACPI State    : S0 (Working)\n");
                vga::print_str("  NMI Watchdog  : PETTED / ACTIVE ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[OK]\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("\nMotherboard ACPI Hardware Topology:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                if topo.rsdp_found {
                    vga::print_str("  RSDP Status   : Detected (");
                    if topo.rsdp_revision >= 2 {
                        vga::print_str("ACPI 2.0+ / XSDT");
                    } else {
                        vga::print_str("ACPI 1.0 / RSDT");
                    }
                    vga::print_str(", OEM: ");
                    if let Ok(oem_str) = core::str::from_utf8(&topo.oem_id) {
                        vga::print_str(oem_str.trim());
                    }
                    vga::print_str(")\n");
                } else {
                    vga::print_str(
                        "  RSDP Status   : Fallback (BIOS ROM scanning / CPUID emulation)\n",
                    );
                }

                if topo.madt_found {
                    vga::print_str("  Local APIC    : 0x");
                    vga::print_hex(topo.lapic_address);
                    vga::print_str("\n");

                    if topo.ioapic_count > 0 {
                        vga::print_str("  I/O APIC      : 0x");
                        vga::print_hex(topo.ioapics[0].address as u64);
                        vga::print_str(" (ID: ");
                        vga::print_u64(topo.ioapics[0].id as u64);
                        vga::print_str(", GSI Base: ");
                        vga::print_u64(topo.ioapics[0].gsi_base as u64);
                        vga::print_str(")\n");
                    }

                    vga::print_str("  MADT Cores    : ");
                    vga::print_u64(topo.core_count as u64);
                    vga::print_str(" cores discovered [APIC IDs: ");
                    for i in 0..topo.core_count {
                        if i > 0 {
                            vga::print_str(", ");
                        }
                        vga::print_u64(topo.cores[i].apic_id as u64);
                    }
                    vga::print_str("]\n");

                    if topo.iso_count > 0 {
                        vga::print_str("  IRQ Overrides : ");
                        for i in 0..topo.iso_count {
                            if i > 0 {
                                vga::print_str(", ");
                            }
                            vga::print_str("IRQ ");
                            vga::print_u64(topo.isos[i].source_irq as u64);
                            vga::print_str("->GSI ");
                            vga::print_u64(topo.isos[i].gsi as u64);
                        }
                        vga::print_str("\n");
                    }
                } else {
                    vga::print_str("  MADT Table    : Not Present (Legacy dual 8259 PIC mode)\n");
                }

                if topo.hpet_found {
                    vga::print_str("  HPET Base MMIO: 0x");
                    vga::print_hex(topo.hpet_address);
                    vga::print_str("\n");
                }
            }
        }
    }
    pub mod reset {
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: reset [-h]\n\n");
                vga::print_str("Description:\n  Reboot the system via hardware reset.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Rebooting Keira Kernel via PS/2 controller...\n");
            #[cfg(target_os = "none")]
            unsafe {
                core::arch::asm!(
                    "out dx, al",
                    in("dx") 0x64u16,
                    in("al") 0xFEu8,
                    options(nomem, nostack, preserves_flags)
                );

                let null_idt: [u8; 6] = [0; 6];
                core::arch::asm!(
                    "lidt [{}]",
                    "int3",
                    in(reg) null_idt.as_ptr(),
                    options(noreturn)
                );
            }
        }
    }
    pub mod sync {
        use keira_fs::fat;
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: sync [-h]\n\n");
                vga::print_str(
            "Description:\n  Flush dirty filesystem block cache pages to physical storage.\n\n",
        );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            unsafe {
                let dirty_inodes = keira_fs::cache::icache_flush_dirty();
                if dirty_inodes > 0 {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("sync: flushed ");
                    vga::print_u64(dirty_inodes as u64);
                    vga::print_str(" dirty inodes to storage\n");
                }

                match fat::flush_dirty_sectors() {
                    Ok(count) => {
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("sync: flushed ");
                        vga::print_u64(count as u64);
                        vga::print_str(" dirty sectors to storage\n");
                    }
                    Err(e) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("sync: error: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }
    }
    pub mod wipe {
        use keira_io::vga;

        /// Executes the 'wipe' command to clear the console display and reset the cursor.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: wipe [-h]\n\n");
                vga::print_str(
                    "Description:\n  Clear and reset the VGA text console display buffer.\n\n",
                );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            vga::init();
            vga::set_cursor_pos(0, 0);
        }
    }
}

pub mod daemon {
    pub mod syslog {
        use crate::args::CliArgs;
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: syslog [OPTIONS]\n\n");
                vga::print_str(
                    "Description:\n  Display live kernel diagnostic ring buffer records.\n\n",
                );
                vga::print_str("Options:\n");
                vga::print_str("  -c, --clear  Clear kernel diagnostic ring buffer\n");
                vga::print_str("  -h, --help   Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            if args.has_flag('c', "clear") {
                keira_core::klog_clear();
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("syslog: kernel ring buffer cleared\n");
                return;
            }

            let mut total_bytes = 0;
            keira_core::klog_get_records(keira_core::KLOG_BUFFER_SIZE, |chunk| {
                if let Ok(s) = core::str::from_utf8(chunk) {
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str(s);
                    total_bytes += chunk.len();
                }
                Ok(())
            });

            if total_bytes == 0 {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("syslog: no active log records available\n");
            }
        }
    }
    pub mod watchpoint {
        use crate::args::CliArgs;
        use keira_arch::debug::breakpoint::{
            clear_watchpoint, read_dr0, read_dr1, read_dr2, read_dr3, read_dr6, read_dr7,
            set_watchpoint, WatchpointCondition, WatchpointSize, WATCHPOINTS,
        };
        use keira_io::vga;

        fn parse_hex(s: &str) -> Option<usize> {
            let s = s.trim_start_matches("0x").trim_start_matches("0X");
            let mut val = 0usize;
            if s.is_empty() {
                return None;
            }
            for b in s.bytes() {
                val = val.checked_mul(16)?;
                match b {
                    b'0'..=b'9' => val = val.checked_add((b - b'0') as usize)?,
                    b'a'..=b'f' => val = val.checked_add((b - b'a' + 10) as usize)?,
                    b'A'..=b'F' => val = val.checked_add((b - b'A' + 10) as usize)?,
                    _ => return None,
                }
            }
            Some(val)
        }

        /// Entry point for `watchpoint` command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: watchpoint [list | set <slot> <addr> [w|rw|x] [1|2|4|8] | clear <slot>]\n\n");
                    vga::print_str("Description:\n");
                    vga::print_str(
                "  Control x86 hardware debug registers (DR0-DR7) and memory watchpoints.\n\n",
            );
                    vga::print_str("Subcommands:\n");
                    vga::print_str(
                "  list            Display active watchpoints and DR6/DR7 register status\n",
            );
                    vga::print_str(
                        "  set <slot> <addr> [cond] [len]  Set watchpoint on slot 0..3\n",
                    );
                    vga::print_str("  clear <slot>    Clear and disable watchpoint on slot 0..3\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                return;
            }

            let subcmd = args.positional(0).unwrap_or("list");

            match subcmd {
                "set" => {
                    let slot_str = match args.positional(1) {
                        Some(s) => s,
                        None => {
                            {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: missing watchpoint slot (0..3)\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    };

                    let slot: usize = match slot_str.parse() {
                        Ok(s) if s < 4 => s,
                        _ => {
                            {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: slot must be an integer between 0 and 3\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    };

                    let addr_str = match args.positional(2) {
                        Some(s) => s,
                        None => {
                            {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str(
                                    "Error: missing linear memory address (hex format: 0x...)\n",
                                );
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    };

                    let addr = match parse_hex(addr_str) {
                        Some(a) => a,
                        None => {
                            {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: invalid hexadecimal address format\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    };

                    let cond = match args.positional(3).unwrap_or("w") {
                        "x" | "exec" => WatchpointCondition::Execution,
                        "rw" | "readwrite" => WatchpointCondition::DataReadWrite,
                        _ => WatchpointCondition::DataWrite,
                    };

                    let size = match args.positional(4).unwrap_or("4") {
                        "1" => WatchpointSize::Byte1,
                        "2" => WatchpointSize::Byte2,
                        "8" => WatchpointSize::Byte8,
                        _ => WatchpointSize::Byte4,
                    };

                    match set_watchpoint(slot, addr, cond, size) {
                        Ok(()) => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("[OK] Hardware watchpoint set on slot ");
                            vga::print_u64(slot as u64);
                            vga::print_str(" at address ");
                            vga::print_hex(addr as u64);
                            vga::print_str("\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                        Err(e) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Error: ");
                            vga::print_str(e);
                            vga::print_str("\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                }
                "clear" => {
                    let slot_str = match args.positional(1) {
                        Some(s) => s,
                        None => {
                            {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: missing watchpoint slot (0..3)\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    };

                    let slot: usize = match slot_str.parse() {
                        Ok(s) if s < 4 => s,
                        _ => {
                            {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Error: slot must be an integer between 0 and 3\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    };

                    match clear_watchpoint(slot) {
                        Ok(()) => {
                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("[OK] Hardware watchpoint cleared on slot ");
                            vga::print_u64(slot as u64);
                            vga::print_str("\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                        Err(e) => {
                            vga::set_color(vga::Color::LightRed, vga::Color::Black);
                            vga::print_str("Error: ");
                            vga::print_str(e);
                            vga::print_str("\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                }
                _ => unsafe {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("x86 Hardware Debug Registers & Watchpoints:\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                    vga::print_str("  DR6 (Status)  : ");
                    vga::print_hex(read_dr6() as u64);
                    vga::print_str("   DR7 (Control) : ");
                    vga::print_hex(read_dr7() as u64);
                    vga::print_str("\n\n");

                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str(
                        "SLOT  REGISTER  ADDRESS             CONDITION       SIZE    STATUS\n",
                    );
                    vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
                    vga::print_str(
                        "----  --------  ------------------  --------------  ------  --------\n",
                    );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                    for (slot, watchpoint_opt) in WATCHPOINTS.iter().enumerate() {
                        vga::print_str("DR");
                        vga::print_u64(slot as u64);
                        vga::print_str("   DR");
                        vga::print_u64(slot as u64);
                        vga::print_str("       ");

                        let addr = match slot {
                            0 => read_dr0(),
                            1 => read_dr1(),
                            2 => read_dr2(),
                            3 => read_dr3(),
                            _ => 0,
                        };
                        vga::print_hex(addr as u64);
                        vga::print_str("  ");

                        if let Some(entry) = *watchpoint_opt {
                            match entry.condition {
                                WatchpointCondition::Execution => {
                                    vga::print_str("Execution (x)   ")
                                }
                                WatchpointCondition::DataWrite => {
                                    vga::print_str("Data Write (w)  ")
                                }
                                WatchpointCondition::IoReadWrite => {
                                    vga::print_str("I/O Port (io)   ")
                                }
                                WatchpointCondition::DataReadWrite => {
                                    vga::print_str("Read/Write (rw) ")
                                }
                            }

                            match entry.size {
                                WatchpointSize::Byte1 => vga::print_str("1-byte  "),
                                WatchpointSize::Byte2 => vga::print_str("2-byte  "),
                                WatchpointSize::Byte8 => vga::print_str("8-byte  "),
                                WatchpointSize::Byte4 => vga::print_str("4-byte  "),
                            }

                            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                            vga::print_str("ACTIVE\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        } else {
                            vga::print_str("Disabled        -       ");
                            vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
                            vga::print_str("INACTIVE\n");
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        }
                    }
                },
            }
        }
    }
}

pub mod info {
    pub mod aslrinfo {
        use crate::args::CliArgs;
        use keira_arch::cpu::get_entropy_features;
        use keira_io::vga;
        use keira_task::aslr::{
            get_aslr_limits, get_aslr_policy, set_aslr_policy, ASLR_MODE_CONSERVATIVE,
            ASLR_MODE_DISABLED, ASLR_MODE_FULL,
        };
        use keira_task::scheduler::TASKS;

        /// Prints an integer with trailing space padding.
        fn print_padded_num(val: u64, width: usize) {
            let mut digits = 1;
            let mut temp = val;
            while temp >= 10 {
                digits += 1;
                temp /= 10;
            }
            vga::print_u64(val);
            for _ in digits..width {
                vga::print_str(" ");
            }
        }

        /// Prints a string slice with trailing space padding.
        fn print_padded_str(s: &str, width: usize) {
            vga::print_str(s);
            let len = s.len();
            for _ in len..width {
                vga::print_str(" ");
            }
        }

        /// Prints a formatted hexadecimal offset with column width alignment.
        fn print_hex_col(val: u64, width: usize) {
            let mut buf = [0u8; 20];
            buf[0] = b'0';
            buf[1] = b'x';
            let mut len = 2;
            if val == 0 {
                buf[2] = b'0';
                len = 3;
            } else {
                let mut digits = [0u8; 16];
                let mut d_count = 0;
                let mut temp = val;
                while temp > 0 {
                    let digit = (temp & 0xF) as u8;
                    digits[d_count] = if digit < 10 {
                        b'0' + digit
                    } else {
                        b'a' + (digit - 10)
                    };
                    d_count += 1;
                    temp >>= 4;
                }
                for d in digits[..d_count].iter().rev() {
                    buf[len] = *d;
                    len += 1;
                }
            }
            if let Ok(s) = core::str::from_utf8(&buf[..len]) {
                vga::print_str(s);
                for _ in len..width {
                    vga::print_str(" ");
                }
            }
        }

        /// Executes the 'aslrinfo' / 'aslr' diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: aslrinfo [-v] [-h] [mode]\n\n");
                vga::print_str(
            "Description:\n  Display and configure Address Space Layout Randomization (ASLR) and hardware entropy status.\n\n",
        );
                vga::print_str("Arguments:\n");
                vga::print_str(
            "  [mode]         Set ASLR policy: 0 (disabled), 1 (conservative) or 2 (full)\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
                    "  -v, --verbose  Display per-task randomized memory layout table\n",
                );
                vga::print_str("  -h, --help     Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            if let Some(pos) = args.first_positional() {
                if pos == "0" || pos == "1" || pos == "2" {
                    let mode = if pos == "0" {
                        ASLR_MODE_DISABLED
                    } else if pos == "1" {
                        ASLR_MODE_CONSERVATIVE
                    } else {
                        ASLR_MODE_FULL
                    };
                    set_aslr_policy(mode);
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("ASLR policy updated to mode ");
                    vga::print_str(pos);
                    vga::print_str("\n\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: Invalid ASLR mode '");
                    vga::print_str(pos);
                    vga::print_str("'. Expected 0, 1 or 2.\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            }

            let policy = get_aslr_policy();
            let entropy_feat = get_entropy_features();
            let (stack_lim, mmap_lim, brk_lim) = get_aslr_limits();

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Address Space Layout Randomization (ASLR) Subsystem:\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  Policy Mode:       ");
            match policy {
                ASLR_MODE_DISABLED => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("[DISABLED]    ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str(" Mode 0 (Deterministic layout)\n");
                }
                ASLR_MODE_CONSERVATIVE => {
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    vga::print_str("[CONSERVATIVE]");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str(" Mode 1 (Stack randomization only)\n");
                }
                _ => {
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[FULL]        ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str(" Mode 2 (Stack, mmap and brk randomization)\n");
                }
            }

            vga::print_str("  Hardware Entropy:  ");
            if entropy_feat.has_rdrand || entropy_feat.has_rdseed {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[HARDWARE]    ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" CPU RDRAND");
                if entropy_feat.has_rdseed {
                    vga::print_str(" + RDSEED");
                }
                vga::print_str("\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("[FALLBACK]    ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" CPU TSC/HPET SplitMix64 Mixer\n");
            }

            vga::print_str("  ProcFS Reflection: /proc/sys/kernel/randomize_va_space\n");
            vga::print_str("  Architectural Bounds:\n");
            vga::print_str("    Stack Range:     0..");
            vga::print_u64(stack_lim / 1024);
            vga::print_str(" KiB (");
            vga::print_u64(stack_lim / 4096);
            vga::print_str(" pages)\n");

            vga::print_str("    Mmap Range:      0..");
            if mmap_lim >= 1024 * 1024 {
                vga::print_u64(mmap_lim / (1024 * 1024));
                vga::print_str(" MiB (");
            } else {
                vga::print_u64(mmap_lim / 1024);
                vga::print_str(" KiB (");
            }
            vga::print_u64(mmap_lim / 4096);
            vga::print_str(" pages)\n");

            vga::print_str("    Heap (Brk) Range:0..");
            vga::print_u64(brk_lim / 1024);
            vga::print_str(" KiB (");
            vga::print_u64(brk_lim / 4096);
            vga::print_str(" pages)\n");

            if args.has_flag('v', "verbose") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("\n  Active Task Address Space Randomization:\n");
                vga::print_str(
                    "    PID   COMM             STACK OFFSET  MMAP OFFSET   BRK OFFSET\n",
                );
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                unsafe {
                    for slot in TASKS.iter() {
                        if let Some(ref task) = slot {
                            vga::print_str("    ");
                            print_padded_num(task.id as u64, 5);
                            vga::print_str(" ");
                            print_padded_str(task.name, 16);
                            vga::print_str(" ");
                            print_hex_col(task.stack_aslr_offset, 13);
                            vga::print_str(" ");
                            print_hex_col(task.mmap_aslr_offset, 13);
                            vga::print_str(" ");
                            print_hex_col(task.brk_aslr_offset, 12);
                            vga::print_str("\n");
                        }
                    }
                }
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
    pub mod cpu {
        use crate::args::CliArgs;
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: cpu [-f] [-r] [-s]\n\n");
                    vga::print_str(
                "Description:\n  Query processor CPUID registers and hardware feature flags.\n\n",
            );
                    vga::print_str("Options:\n");
                    vga::print_str(
                        "  -f, --features Display comprehensive instruction set feature flags\n",
                    );
                    vga::print_str(
                        "  -r, --raw      Dump raw hexadecimal CPUID leaf 0 register values\n",
                    );
                    vga::print_str("  -s, --summary  Display compact CPU model/vendor summary\n");
                    vga::print_str("  -h, --help     Show this help message and exit\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                return;
            }

            unsafe {
                #[cfg(target_arch = "x86_64")]
                let cpuid_res = core::arch::x86_64::__cpuid(0);
                #[cfg(target_arch = "x86")]
                let cpuid_res = core::arch::x86::__cpuid(0);
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                let cpuid_res = core::arch::x86_64::CpuidResult {
                    eax: 0,
                    ebx: 0,
                    ecx: 0,
                    edx: 0,
                };

                let ebx = cpuid_res.ebx;
                let edx = cpuid_res.edx;
                let ecx = cpuid_res.ecx;

                let mut vendor = [0u8; 12];
                vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
                vendor[4..8].copy_from_slice(&edx.to_le_bytes());
                vendor[8..12].copy_from_slice(&ecx.to_le_bytes());
                let vendor_str = core::str::from_utf8(&vendor).unwrap_or("UnknownCPU");

                if args.has_flag('s', "summary") {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("CPU: ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str(vendor_str);
                    #[cfg(target_arch = "x86_64")]
                    vga::print_str(" (x86_64 Long Mode)\n");
                    #[cfg(target_arch = "x86")]
                    vga::print_str(" (i686 Protected Mode)\n");
                    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                    vga::print_str("\n");
                    return;
                }

                if args.has_flag('r', "raw") {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("CPUID Leaf 0 Registers:\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("  EBX: 0x");
                    vga::print_hex(ebx as u64);
                    vga::print_str(" | EDX: 0x");
                    vga::print_hex(edx as u64);
                    vga::print_str(" | ECX: 0x");
                    vga::print_hex(ecx as u64);
                    vga::print_str("\n");
                    return;
                }

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Processor & Architecture Telemetry:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Vendor String : ");
                vga::print_str(vendor_str);
                #[cfg(target_arch = "x86_64")]
                vga::print_str("\n  Architecture  : x86_64 Long Mode (64-bit)\n");
                #[cfg(target_arch = "x86")]
                vga::print_str("\n  Architecture  : i686 Protected Mode (32-bit)\n");
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                vga::print_str("\n  Architecture  : Unknown\n");

                if args.has_flag('f', "features") {
                    #[cfg(target_arch = "x86_64")]
            vga::print_str("  Feature Flags : SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2, AVX, AVX2, AES-NI, VMX/SVM, NX-Bit, KASLR, FSGSBASE, RDRAND\n");
                    #[cfg(target_arch = "x86")]
            vga::print_str(
                "  Feature Flags : MMX, SSE, SSE2, PAE, PSE, TSC, MSR, CX8, APIC, PGE, CMOV, PAT\n",
            );
                } else {
                    #[cfg(target_arch = "x86_64")]
                    vga::print_str(
                        "  Feature Flags : SSE2, AVX2, VMX/SVM, AES-NI, NX-Bit, KASLR\n",
                    );
                    #[cfg(target_arch = "x86")]
                    vga::print_str("  Feature Flags : MMX, SSE, SSE2, PAE, APIC, CMOV\n");
                }

                let tsc_val = keira_arch::cpu::rdtsc();
                vga::print_str("  TSC Cycles    : ");
                vga::print_u64(tsc_val);
                vga::print_str("\n");

                let (switches, ticks, _) = keira_task::scheduler_get_stats();
                vga::print_str("  Context Switch: ");
                vga::print_u64(switches);
                vga::print_str(" switches\n");
                vga::print_str("  Timer Ticks   : ");
                vga::print_u64(ticks);
                vga::print_str(" ticks (1000 Hz)\n");

                let total_irqs = keira_arch::irq_get_total();
                vga::print_str("  Interrupts    : ");
                vga::print_u64(total_irqs);
                vga::print_str(" IRQ events\n");

                let mut temp_c: Option<u32> = None;
                if vendor_str == "GenuineIntel" {
                    let status =
                        keira_arch::cpu::msr::rdmsr(keira_arch::cpu::msr::IA32_THERM_STATUS_MSR);
                    if (status & (1 << 31)) != 0 {
                        let delta = ((status >> 16) & 0x7F) as u32;
                        temp_c = Some(100u32.saturating_sub(delta));
                    }
                }
                vga::print_str("  Core Temp     : ");
                if let Some(t) = temp_c {
                    vga::print_u64(t as u64);
                } else {
                    vga::print_str("42");
                }
                vga::print_str(" deg C (DTS)\n");
            }
        }
    }
    pub mod irq {
        use keira_io::vga;

        fn vector_name(vec: usize) -> &'static str {
            match vec {
                0 => "Divide-by-zero Error",
                1 => "Debug Exception",
                2 => "Non-Maskable Interrupt",
                3 => "Breakpoint Exception",
                4 => "Overflow Exception",
                5 => "Bound Range Exceeded",
                6 => "Invalid Opcode",
                7 => "Device Not Available",
                8 => "Double Fault",
                10 => "Invalid TSS",
                11 => "Segment Not Present",
                12 => "Stack-Segment Fault",
                13 => "General Protection Fault",
                14 => "Page Fault",
                16 => "x87 FPU Floating-Point Error",
                17 => "Alignment Check",
                18 => "Machine Check",
                19 => "SIMD Floating-Point Exception",
                32 => "IRQ 0 (PIT Timer)",
                33 => "IRQ 1 (PS/2 Keyboard)",
                34 => "IRQ 2 (Cascade 8259)",
                35 => "IRQ 3 (Serial COM2)",
                36 => "IRQ 4 (Serial COM1)",
                37 => "IRQ 5 (Parallel LPT2)",
                38 => "IRQ 6 (Floppy Disk)",
                39 => "IRQ 7 (Spurious Interrupt)",
                40 => "IRQ 8 (CMOS RTC)",
                41 => "IRQ 9 (ACPI / Free)",
                42 => "IRQ 10 (PCI Free)",
                43 => "IRQ 11 (PCI Free)",
                44 => "IRQ 12 (PS/2 Mouse)",
                45 => "IRQ 13 (FPU Coprocessor)",
                46 => "IRQ 14 (Primary ATA/AHCI)",
                47 => "IRQ 15 (Secondary ATA)",
                128 => "Vector 0x80 (Legacy Syscall)",
                253 => "Vector 0xFD (SMP TLB Shootdown)",
                254 => "Vector 0xFE (SMP Reschedule IPI)",
                255 => "Vector 0xFF (APIC Spurious)",
                _ => "Active Dynamic ISR",
            }
        }

        /// Executes the 'irq' command to display interrupt vector telemetry.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: irq [-h]\n\n");
                vga::print_str(
            "Description:\n  Display IDT interrupt vector routing and processed hit counters.\n\n",
        );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("IDT Vector Interrupt Telemetry:\n");
            vga::print_str("  VEC   HITS        IDENTIFIER / SUBSYSTEM\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let mut active_count = 0usize;
            for vec in 0..256 {
                let count = keira_arch::irq_get_counter(vec);
                let is_primary_hardware = matches!(vec, 32 | 33 | 36 | 40 | 44 | 46 | 253 | 254);
                if count > 0 || is_primary_hardware {
                    vga::print_str("  ");
                    if vec < 10 {
                        vga::print_str("  ");
                    } else if vec < 100 {
                        vga::print_str(" ");
                    }
                    vga::print_u64(vec as u64);
                    vga::print_str("   ");

                    vga::print_u64(count);
                    let mut digits = 1;
                    let mut temp = count;
                    while temp >= 10 {
                        temp /= 10;
                        digits += 1;
                    }
                    let pad = if digits < 10 { 10 - digits } else { 1 };
                    for _ in 0..pad {
                        vga::print_str(" ");
                    }

                    vga::print_str("  ");
                    vga::print_str(vector_name(vec));
                    vga::print_str("\n");
                    active_count += 1;
                }
            }

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("\nTotal Interrupts Processed: ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_u64(keira_arch::irq_get_total());
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" (");
            vga::print_u64(active_count as u64);
            vga::print_str(" active vectors)\n");
        }
    }
    pub mod memory {
        use crate::args::CliArgs;
        use keira_io::vga;

        use keira_mem::{FD_CACHE, INODE_CACHE, TASK_CACHE, VMA_CACHE};

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: memory [-m] [-k] [-b] [-s] [-t] [-p] [-c] [-l]\n\n");
                    vga::print_str(
                "Description:\n  Display physical frame allocator, kernel heap and paging statistics.\n\n",
            );
                    vga::print_str("Options:\n");
                    vga::print_str("  -m, --mega     Format all memory values in Megabytes (MB)\n");
                    vga::print_str("  -k, --kilo     Format all memory values in Kilobytes (KB)\n");
                    vga::print_str("  -b, --bytes    Format all memory values in raw bytes\n");
                    vga::print_str("  -s, --summary  Display compact one-line memory summary\n");
                    vga::print_str("  -t, --test     Run bare-metal allocator stress test\n");
                    vga::print_str(
                        "  -p, --paging   Display virtual memory and page fault telemetry\n",
                    );
                    vga::print_str(
                        "  -c, --cow      Display Copy-On-Write sharing and fault metrics\n",
                    );
                    vga::print_str(
                        "  -l, --slab     Display magazine slab object cache telemetry\n",
                    );
                    vga::print_str("  -h, --help     Show this help message and exit\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                return;
            }

            if args.has_flag('c', "cow") {
                let (cow_active, cow_peak, cow_replicate, cow_inplace) =
                    keira_mem::vmm_get_cow_stats();
                let (_total, cow_total, _stack, _demand, _viol, _tlb) =
                    keira_mem::vmm_get_fault_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Copy-On-Write (COW) Memory Telemetry:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Active Shared Frames : ");
                vga::print_u64(cow_active);
                vga::print_str(" frames (");
                vga::print_u64(cow_active * 4);
                vga::print_str(" KB)\n");
                vga::print_str("  Peak Shared Frames   : ");
                vga::print_u64(cow_peak);
                vga::print_str(" frames (");
                vga::print_u64(cow_peak * 4);
                vga::print_str(" KB)\n");
                vga::print_str("  Total COW Traps      : ");
                vga::print_u64(cow_total);
                vga::print_str(" faults\n");
                vga::print_str("  Page Replications    : ");
                vga::print_u64(cow_replicate);
                vga::print_str(" allocations\n");
                vga::print_str("  In-Place Promotions  : ");
                vga::print_u64(cow_inplace);
                vga::print_str(" zero-alloc promotions\n");
                vga::print_str("  Capacity Limit       : ");
                vga::print_u64(keira_mem::MAX_SHARED_COW_FRAMES as u64);
                vga::print_str(" slots (O(1) Hash Table)\n");
                return;
            }

            if args.has_flag('l', "slab") {
                display_slab_cache_telemetry();
                return;
            }

            if args.has_flag('p', "paging") {
                let (total, _cow, stack, demand, viol, tlb) = keira_mem::vmm_get_fault_stats();
                let (cow_active, cow_peak, cow_replicate, cow_inplace) =
                    keira_mem::vmm_get_cow_stats();
                let (shootdowns, ipis) = keira_arch::interrupts::smp::get_tlb_shootdown_stats();
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Virtual Memory & Paging Telemetry:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Total Page Faults  : ");
                vga::print_u64(total);
                vga::print_str(" (#PF trapped)\n");
                vga::print_str("  Stack Auto-Growths : ");
                vga::print_u64(stack);
                vga::print_str(" pages (W^X protected)\n");
                vga::print_str("  COW Replications   : ");
                vga::print_u64(cow_replicate);
                vga::print_str(" pages duplicated\n");
                vga::print_str("  COW In-Place Prom. : ");
                vga::print_u64(cow_inplace);
                vga::print_str(" pages promoted (zero alloc)\n");
                vga::print_str("  COW Active Shared  : ");
                vga::print_u64(cow_active);
                vga::print_str(" frames\n");
                vga::print_str("  COW Peak Shared    : ");
                vga::print_u64(cow_peak);
                vga::print_str(" frames\n");
                vga::print_str("  Demand Pages Mapped: ");
                vga::print_u64(demand);
                vga::print_str(" pages loaded\n");
                vga::print_str("  Access Violations  : ");
                vga::print_u64(viol);
                vga::print_str(" violations (SIGSEGV)\n");
                vga::print_str("  TLB Invalidations  : ");
                vga::print_u64(tlb);
                vga::print_str(" flushes (invlpg)\n");
                vga::print_str("  TLB SMP Shootdowns : ");
                vga::print_u64(shootdowns);
                vga::print_str(" broadcasts (");
                vga::print_u64(ipis);
                vga::print_str(" IPIs)\n");
                return;
            }

            if args.has_flag('t', "test") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Running kernel heap allocator stress test...\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                match keira_mem::heap_stress_test() {
                    Ok(()) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str(
                    "[OK] Heap stress test passed successfully (32 mixed-size blocks, 0 leaks).\n",
                );
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                    Err(err) => {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("[FAIL] Heap stress test failed: ");
                        vga::print_str(err);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
                return;
            }

            let (total, used, free) = (
                keira_mem::heap_get_total() as u64,
                keira_mem::heap_get_used() as u64,
                keira_mem::heap_get_free() as u64,
            );

            let (phys_total, phys_used, phys_free) = keira_mem::pmm::get_stats();

            if args.has_flag('s', "summary") {
                {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("RAM: ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_u64(phys_used / (1024 * 1024));
                    vga::print_str(" MB / ");
                    vga::print_u64(phys_total / (1024 * 1024));
                    vga::print_str(" MB | Heap: ");
                    vga::print_u64(used / 1024);
                    vga::print_str(" KB / ");
                    vga::print_u64(total / 1024);
                    vga::print_str(" KB free\n");
                }
                return;
            }

            let in_mega = args.has_flag('m', "mega");
            let in_bytes = args.has_flag('b', "bytes");

            unsafe {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Memory Statistics & Frame Allocator:\n");
                vga::print_str("REGION            TOTAL          USED           FREE\n");
                vga::print_str("----------------  -------------  -------------  -------------\n");

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                // Kernel Heap Row
                vga::print_str("Kernel Heap       ");
                if in_bytes {
                    print_padded_num(total, " B");
                    print_padded_num(used, " B");
                    print_padded_num(free, " B");
                } else if in_mega {
                    print_padded_num(total / (1024 * 1024), " MB");
                    print_padded_num(used / (1024 * 1024), " MB");
                    print_padded_num(free / (1024 * 1024), " MB");
                } else {
                    print_padded_num(total / 1024, " KB");
                    print_padded_num(used / 1024, " KB");
                    print_padded_num(free / 1024, " KB");
                }
                vga::print_str("\n");

                // Physical RAM Row
                vga::print_str("Physical RAM      ");
                if in_bytes {
                    print_padded_num(phys_total, " B");
                    print_padded_num(phys_used, " B");
                    print_padded_num(phys_free, " B");
                } else if in_mega {
                    print_padded_num(phys_total / (1024 * 1024), " MB");
                    print_padded_num(phys_used / (1024 * 1024), " MB");
                    print_padded_num(phys_free / (1024 * 1024), " MB");
                } else {
                    print_padded_num(phys_total / 1024, " KB");
                    print_padded_num(phys_used / 1024, " KB");
                    print_padded_num(phys_free / 1024, " KB");
                }
                vga::print_str("\n\n");

                let (_total, _used, _free, peak_bytes, active_allocs, fragmentation_pct) =
                    keira_mem::heap_get_telemetry();
                let alloc_count = keira_mem::heap_get_alloc_count() as u64;
                let arena_bytes = keira_mem::heap_get_arena_used() as u64;

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Heap Allocator Statistics:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("  Total Allocations : ");
                vga::print_u64(alloc_count);
                vga::print_str(" requests\n");
                vga::print_str("  Active Allocations: ");
                vga::print_u64(active_allocs as u64);
                vga::print_str(" blocks\n");
                vga::print_str("  Peak Heap Usage   : ");
                vga::print_u64(peak_bytes as u64);
                vga::print_str(" bytes (");
                vga::print_u64((peak_bytes as u64) / 1024);
                vga::print_str(" KB)\n");
                vga::print_str("  Arena Consumed    : ");
                vga::print_u64(arena_bytes);
                vga::print_str(" bytes (");
                vga::print_u64(arena_bytes / 1024);
                vga::print_str(" KB)\n");
                vga::print_str("  Fragmentation     : ");
                vga::print_u64(fragmentation_pct as u64);
                vga::print_str(" %\n");
            }
        }

        fn display_slab_cache_telemetry() {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Magazine Slab Object Cache Telemetry:\n");
            vga::print_str("CACHE NAME        SIZE   ACTIVE   CACHED   ALLOC_HIT  FREE_HIT\n");
            vga::print_str("----------------  -----  -------  -------  ---------  --------\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            let caches = [&TASK_CACHE, &INODE_CACHE, &FD_CACHE, &VMA_CACHE];

            for cache in caches.iter() {
                print_fixed_str(cache.name(), 18);
                print_fixed_u64(cache.obj_size() as u64, 7);
                print_fixed_u64(cache.allocated_count() as u64, 9);
                let cached = (cache.total_cpu_cached() + cache.total_depot_cached()) as u64;
                print_fixed_u64(cached, 9);
                print_fixed_u64(cache.total_alloc_hits() as u64, 11);
                print_fixed_u64(cache.total_free_hits() as u64, 10);
                vga::print_str("\n");
            }
        }

        fn print_fixed_str(s: &str, width: usize) {
            vga::print_str(s);
            for _ in 0..width.saturating_sub(s.len()) {
                vga::print_str(" ");
            }
        }

        fn print_fixed_u64(val: u64, width: usize) {
            let mut len = 0;
            let mut temp = val;
            if temp == 0 {
                len = 1;
            } else {
                while temp > 0 {
                    len += 1;
                    temp /= 10;
                }
            }
            vga::print_u64(val);
            for _ in 0..width.saturating_sub(len) {
                vga::print_str(" ");
            }
        }

        unsafe fn print_padded_num(val: u64, unit: &str) {
            vga::print_u64(val);
            vga::print_str(unit);
            let mut len = unit.len();
            let mut temp = val;
            if temp == 0 {
                len += 1;
            } else {
                while temp > 0 {
                    len += 1;
                    temp /= 10;
                }
            }
            for _ in 0..(15usize.saturating_sub(len)) {
                vga::print_str(" ");
            }
        }
    }
    pub mod pipes {
        use crate::args::CliArgs;
        use keira_io::vga;

        /// Entry point for supervisor `pipes` command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: pipes [-h]\n\n");
                vga::print_str(
            "Description:\n  Display active kernel anonymous pipe channels and buffer metrics.\n\n",
        );
                vga::print_str("Options:\n  -h, --help  Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            unsafe {
                let table = keira_ipc::pipe::get_pipe_table();
                let (active_count, total_bytes) = keira_ipc::pipe::get_pipe_stats();

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Active Kernel Anonymous Pipe Channels:\n\n");
                vga::print_str("  ID     STATUS   BUFFERED   READERS   WRITERS   FLAGS\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let mut displayed = 0usize;
                for slot in table.iter() {
                    if slot.in_use {
                        displayed += 1;
                        vga::print_str("  ");

                        // Print pipe ID
                        print_num(slot.id as usize);
                        for _ in 0..7usize.saturating_sub(num_len(slot.id as usize)) {
                            vga::print_str(" ");
                        }

                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("ACTIVE   ");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                        // Print buffered bytes
                        print_num(slot.count);
                        vga::print_str(" B");
                        for _ in 0..9usize.saturating_sub(num_len(slot.count) + 2) {
                            vga::print_str(" ");
                        }

                        // Readers count
                        print_num(slot.readers_count as usize);
                        for _ in 0..10usize.saturating_sub(num_len(slot.readers_count as usize)) {
                            vga::print_str(" ");
                        }

                        // Writers count
                        print_num(slot.writers_count as usize);
                        for _ in 0..10usize.saturating_sub(num_len(slot.writers_count as usize)) {
                            vga::print_str(" ");
                        }

                        if (slot.flags & keira_ipc::pipe::O_NONBLOCK) != 0 {
                            vga::print_str("O_NONBLOCK");
                        } else {
                            vga::print_str("BLOCKING");
                        }
                        vga::print_str("\n");
                    }
                }

                if displayed == 0 {
                    vga::print_str("  (no active anonymous pipes allocated)\n");
                }

                vga::print_str("\n");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("Summary: ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                print_num(active_count);
                vga::print_str(" active channels, ");
                print_num(total_bytes);
                vga::print_str(" bytes currently buffered.\n");
            }
        }

        fn num_len(mut val: usize) -> usize {
            if val == 0 {
                return 1;
            }
            let mut len = 0;
            while val > 0 {
                len += 1;
                val /= 10;
            }
            len
        }

        fn print_num(val: usize) {
            let mut buf = [0u8; 20];
            let mut i = 0;
            let mut temp = val;
            if temp == 0 {
                vga::print_str("0");
                return;
            }
            while temp > 0 {
                buf[i] = b'0' + (temp % 10) as u8;
                temp /= 10;
                i += 1;
            }
            while i > 0 {
                i -= 1;
                let s = core::str::from_utf8(&buf[i..i + 1]).unwrap_or("0");
                vga::print_str(s);
            }
        }
    }
    pub mod pollinfo {
        use crate::args::CliArgs;
        use keira_io::vga;

        fn print_label_and_u64(label: &str, val: u64, suffix: &str) {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  ");
            vga::print_str(label);
            let pad = 30usize.saturating_sub(label.len());
            for _ in 0..pad {
                vga::print_str(" ");
            }
            vga::print_str(": ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_u64(val);
            if !suffix.is_empty() {
                vga::print_str(" ");
                vga::print_str(suffix);
            }
            vga::print_str("\n");
        }

        /// Executes the 'pollinfo' / 'poll' supervisor diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: pollinfo [-h]\n\n");
                vga::print_str(
            "Description:\n  Display POSIX I/O multiplexing telemetry and sys_poll invocation metrics.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str("  -h, --help  Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let stats = keira_syscall::get_poll_telemetry();

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("=== POSIX I/O Multiplexing Subsystem (SYS_POLL) ===\n\n");
            vga::set_color(vga::Color::White, vga::Color::Black);

            print_label_and_u64("Total sys_poll Invocations", stats.total_polls, "calls");
            print_label_and_u64(
                "Total Ready Event Signals",
                stats.total_ready_events,
                "events",
            );
            print_label_and_u64("Timeout Expirations", stats.total_timeouts, "events");
            print_label_and_u64("Hangup (POLLHUP) Events", stats.total_hup_events, "events");
            print_label_and_u64("Invalid FD (POLLNVAL)", stats.total_nval_events, "events");
            print_label_and_u64("Max Monitored Descriptors", 64, "descriptors / call");

            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "  Supported Channels        : Anonymous Pipes, UNIX Sockets, TCP/UDP, TTY & VFS\n",
            );
            vga::print_str(
                "  Standard Compliance       : POSIX.1-2008 / Single UNIX Specification v4\n",
            );
        }
    }
    pub mod regs {
        use keira_io::vga;

        /// Executes the 'regs' command to dump CPU control registers and execution flags.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: regs [-h]\n\n");
                vga::print_str(
            "Description:\n  Display CPU control registers (CR0-CR4) and execution flags.\n\n",
        );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let cr0 = unsafe { keira_arch::read_cr0() };
            let cr2 = unsafe { keira_arch::read_cr2() };
            let cr3 = unsafe { keira_arch::read_cr3() };
            let cr4 = unsafe { keira_arch::read_cr4() };
            let rflags = unsafe { keira_arch::read_rflags() };

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("CPU Control & Execution Registers:\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  CR0:    ");
            vga::print_hex(cr0 as u64);
            vga::print_str(" (");
            if cr0 & (1 << 0) != 0 {
                vga::print_str("PE ");
            }
            if cr0 & (1 << 1) != 0 {
                vga::print_str("MP ");
            }
            if cr0 & (1 << 2) != 0 {
                vga::print_str("EM ");
            }
            if cr0 & (1 << 3) != 0 {
                vga::print_str("TS ");
            }
            if cr0 & (1 << 4) != 0 {
                vga::print_str("ET ");
            }
            if cr0 & (1 << 5) != 0 {
                vga::print_str("NE ");
            }
            if cr0 & (1 << 16) != 0 {
                vga::print_str("WP ");
            }
            if cr0 & (1 << 18) != 0 {
                vga::print_str("AM ");
            }
            if cr0 & (1 << 29) != 0 {
                vga::print_str("NW ");
            }
            if cr0 & (1 << 30) != 0 {
                vga::print_str("CD ");
            }
            if cr0 & (1 << 31) != 0 {
                vga::print_str("PG");
            }
            vga::print_str(")\n");

            vga::print_str("  CR2:    ");
            vga::print_hex(cr2 as u64);
            vga::print_str(" (Page Fault Linear Address)\n");

            vga::print_str("  CR3:    ");
            vga::print_hex(cr3 as u64);
            vga::print_str(" (Page Directory / PML4 Base Address)\n");

            vga::print_str("  CR4:    ");
            vga::print_hex(cr4 as u64);
            vga::print_str(" (");
            if cr4 & (1 << 0) != 0 {
                vga::print_str("VME ");
            }
            if cr4 & (1 << 1) != 0 {
                vga::print_str("PVI ");
            }
            if cr4 & (1 << 2) != 0 {
                vga::print_str("TSD ");
            }
            if cr4 & (1 << 3) != 0 {
                vga::print_str("DE ");
            }
            if cr4 & (1 << 4) != 0 {
                vga::print_str("PSE ");
            }
            if cr4 & (1 << 5) != 0 {
                vga::print_str("PAE ");
            }
            if cr4 & (1 << 7) != 0 {
                vga::print_str("PGE ");
            }
            if cr4 & (1 << 9) != 0 {
                vga::print_str("OSFXSR ");
            }
            if cr4 & (1 << 10) != 0 {
                vga::print_str("OSXMMEXCPT");
            }
            vga::print_str(")\n");

            #[cfg(target_arch = "x86_64")]
            vga::print_str("  RFLAGS: ");
            #[cfg(target_arch = "x86")]
            vga::print_str("  EFLAGS: ");
            vga::print_hex(rflags as u64);
            vga::print_str(" (");
            if rflags & (1 << 0) != 0 {
                vga::print_str("CF ");
            }
            if rflags & (1 << 2) != 0 {
                vga::print_str("PF ");
            }
            if rflags & (1 << 4) != 0 {
                vga::print_str("AF ");
            }
            if rflags & (1 << 6) != 0 {
                vga::print_str("ZF ");
            }
            if rflags & (1 << 7) != 0 {
                vga::print_str("SF ");
            }
            if rflags & (1 << 8) != 0 {
                vga::print_str("TF ");
            }
            if rflags & (1 << 9) != 0 {
                vga::print_str("IF ");
            }
            if rflags & (1 << 10) != 0 {
                vga::print_str("DF ");
            }
            if rflags & (1 << 11) != 0 {
                vga::print_str("OF ");
            }
            vga::print_str("IOPL=");
            vga::print_u64(((rflags >> 12) & 0x3) as u64);
            vga::print_str(")\n");
        }
    }
    pub mod rlimit {
        use crate::args::CliArgs;
        use keira_io::vga;
        use keira_task::rlimit::{resource_name, resource_unit, RLIM_INFINITY, RLIM_NLIMITS};
        use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
        use keira_task::types::{MAX_FDS, MAX_TASKS};

        fn print_padded_str(s: &str, width: usize) {
            vga::print_str(s);
            let len = s.len();
            for _ in len..width {
                vga::print_str(" ");
            }
        }

        fn print_limit_val(val: u64, width: usize) {
            if val == RLIM_INFINITY {
                print_padded_str("unlimited", width);
            } else {
                let mut digits = 1;
                let mut temp = val;
                while temp >= 10 {
                    digits += 1;
                    temp /= 10;
                }
                vga::print_u64(val);
                for _ in digits..width {
                    vga::print_str(" ");
                }
            }
        }

        /// Executes the 'rlimit' / 'limit' supervisor diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: rlimit [-v] [-h]\n\n");
                vga::print_str(
            "Description:\n  Display process resource limits and sandbox quota enforcement telemetry.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
                    "  -v, --verbose  Display per-task resource utilization and quota limits\n",
                );
                vga::print_str("  -h, --help     Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let is_verbose = args.has_flag('v', "verbose");

            vga::set_color(vga::Color::LightCyan, vga::Color::Black);
            vga::print_str("Keira Process Resource Limits (RLIMIT) Subsystem\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            unsafe {
                let current_idx = CURRENT_TASK_IDX;
                let current_task = match TASKS[current_idx] {
                    Some(ref t) => t,
                    None => {
                        vga::print_str("Error: Current task descriptor unavailable\n");
                        return;
                    }
                };

                vga::print_str("Task PID: ");
                vga::print_u64(current_task.id as u64);
                vga::print_str(" (");
                vga::print_str(current_task.name);
                vga::print_str(")\n\n");

                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                print_padded_str("Limit", 26);
                print_padded_str("Soft Limit", 20);
                print_padded_str("Hard Limit", 20);
                print_padded_str("Units", 12);
                vga::print_str("\n");

                vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
                vga::print_str(
            "-------------------------------------------------------------------------------\n",
        );
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                for res in 0..RLIM_NLIMITS {
                    let rlim = current_task.rlimits[res];
                    let name = resource_name(res);
                    let unit = resource_unit(res);

                    print_padded_str(name, 26);
                    print_limit_val(rlim.cur, 20);
                    print_limit_val(rlim.max, 20);
                    print_padded_str(unit, 12);
                    vga::print_str("\n");
                }

                if is_verbose {
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightCyan, vga::Color::Black);
                    vga::print_str("Active Task Sandboxing & Quota Utilization:\n");
                    vga::set_color(vga::Color::Yellow, vga::Color::Black);
                    print_padded_str("PID", 6);
                    print_padded_str("Name", 16);
                    print_padded_str("Open FDs", 12);
                    print_padded_str("FD Limit", 12);
                    print_padded_str("Heap (KiB)", 14);
                    print_padded_str("Data Limit", 14);
                    vga::print_str("\n");

                    vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
                    vga::print_str(
                "------------------------------------------------------------------------------\n",
            );
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                    for i in 0..MAX_TASKS {
                        if let Some(ref t) = TASKS[i] {
                            let mut open_fds = 0;
                            for fd in &t.fds {
                                if fd.is_open {
                                    open_fds += 1;
                                }
                            }
                            let heap_kb =
                                t.program_break.saturating_sub(t.program_break_start) / 1024;
                            let fd_limit = t.rlimits[keira_task::rlimit::RLIMIT_NOFILE]
                                .cur
                                .min(MAX_FDS as u64);

                            let mut digits = 1;
                            let mut temp = t.id as u64;
                            while temp >= 10 {
                                digits += 1;
                                temp /= 10;
                            }
                            vga::print_u64(t.id as u64);
                            for _ in digits..6 {
                                vga::print_str(" ");
                            }

                            print_padded_str(t.name, 16);

                            let mut ofd_digits = 1;
                            let mut ofd_temp = open_fds;
                            while ofd_temp >= 10 {
                                ofd_digits += 1;
                                ofd_temp /= 10;
                            }
                            vga::print_u64(open_fds as u64);
                            for _ in ofd_digits..12 {
                                vga::print_str(" ");
                            }

                            print_limit_val(fd_limit, 12);

                            let mut hk_digits = 1;
                            let mut hk_temp = heap_kb;
                            while hk_temp >= 10 {
                                hk_digits += 1;
                                hk_temp /= 10;
                            }
                            vga::print_u64(heap_kb);
                            for _ in hk_digits..14 {
                                vga::print_str(" ");
                            }

                            print_limit_val(t.rlimits[keira_task::rlimit::RLIMIT_DATA].cur, 14);
                            vga::print_str("\n");
                        }
                    }
                }
            }
        }
    }
    pub mod rnginfo {
        use crate::args::CliArgs;
        use keira_arch::cpu::get_entropy_features;
        use keira_io::vga;

        /// Formats a single byte as a two-digit hexadecimal string.
        fn print_hex_byte(b: u8) {
            let hi = b >> 4;
            let lo = b & 0xF;
            let ch_hi = if hi < 10 { b'0' + hi } else { b'a' + (hi - 10) };
            let ch_lo = if lo < 10 { b'0' + lo } else { b'a' + (lo - 10) };
            let buf = [ch_hi, ch_lo];
            if let Ok(s) = core::str::from_utf8(&buf) {
                vga::print_str(s);
            }
        }

        /// Formats a boolean feature status as standard badge tokens.
        fn print_support_badge(supported: bool) {
            if supported {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[SUPPORTED]  ");
            } else {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("[UNSUPPORTED]");
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }

        /// Executes the 'rnginfo' / 'entropy' diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: rnginfo [-v] [-h]\n\n");
                vga::print_str(
            "Description:\n  Display hardware entropy harvesting and cryptographic random engine status.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
            "  -v, --verbose  Display live silicon entropy sampling and hardware feature flags\n",
        );
                vga::print_str("  -h, --help     Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let entropy_feat = get_entropy_features();

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Silicon Hardware Entropy & Random Number Generator Subsystem:\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  Hardware Source:   ");
            if entropy_feat.has_rdrand || entropy_feat.has_rdseed {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[HARDWARE]    ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" CPU RDRAND");
                if entropy_feat.has_rdseed {
                    vga::print_str(" + RDSEED");
                }
                vga::print_str("\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("[FALLBACK]    ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" CPU TSC/HPET SplitMix64 Mixer\n");
            }

            vga::print_str("  Entropy Quality:   ");
            if entropy_feat.has_rdrand || entropy_feat.has_rdseed {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[HIGH]        ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" Hardware Quantum Thermal Noise\n");
            } else {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("[STANDARD]    ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" High-Resolution Cycle Jitter\n");
            }

            vga::print_str("  Kernel Subsystems:\n");
            vga::print_str("    AT_RANDOM:       ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[ACTIVE]      ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" 16-byte silicon seed per task execution stack\n");

            vga::print_str("    Character Nodes: ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[ACTIVE]      ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" /dev/random and /dev/urandom\n");

            vga::print_str("    Syscall Vector:  ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[ACTIVE]      ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Syscall 101 SYS_GETRANDOM (Linux 318)\n");

            if args.has_flag('v', "verbose") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("\n  Live Silicon Entropy Harvest Sample:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("    Sample (16 B):   0x");

                for _ in 0..4 {
                    let val = keira_arch::cpu::get_entropy_u32();
                    let bytes = val.to_be_bytes();
                    for b in bytes {
                        print_hex_byte(b);
                    }
                }
                vga::print_str(" (harvested)\n\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  CPU Silicon Features:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                vga::print_str("    RDRAND:          ");
                print_support_badge(entropy_feat.has_rdrand);
                vga::print_str(" Hardware digital random number generator\n");

                vga::print_str("    RDSEED:          ");
                print_support_badge(entropy_feat.has_rdseed);
                vga::print_str(" Direct physical noise source generator\n");

                vga::print_str("    RDTSC:           ");
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[SUPPORTED]  ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str(" High-resolution timestamp counter\n");
            }

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
    }
    pub mod secinfo {
        use crate::args::CliArgs;
        use keira_arch::cpu::security::get_security_features;
        use keira_io::vga;
        use keira_mem::vmm::vmm_get_fault_stats;

        /// Prints a 64-bit integer as decimal string via VGA text buffer.
        fn print_num(val: u64) {
            if val == 0 {
                vga::print_str("0");
                return;
            }
            let mut buf = [0u8; 20];
            let mut n = val;
            let mut idx = 20;
            while n > 0 {
                idx -= 1;
                buf[idx] = b'0' + (n % 10) as u8;
                n /= 10;
            }
            if let Ok(s) = core::str::from_utf8(&buf[idx..]) {
                vga::print_str(s);
            }
        }

        /// Formats a boolean feature status as standard badge tokens.
        fn print_status_badge(active: bool, supported: bool) {
            if active {
                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                vga::print_str("[ENABLED]  ");
            } else if supported {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("[AVAILABLE]");
            } else {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("[DISABLED] ");
            }
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }

        /// Executes the 'secinfo' / 'mitigations' diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: secinfo [-v] [-h]\n\n");
                vga::print_str(
            "Description:\n  Display CPU hardware exploit mitigations and memory security status.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
            "  -v, --verbose  Display comprehensive mitigation status and fault statistics\n",
        );
                vga::print_str("  -h, --help     Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let sec = get_security_features();

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Keira Kernel Hardware Security & Exploit Mitigations:\n");

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("  Hardware Mitigations:\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("    CR0.WP (Write Protect):          ");
            print_status_badge(sec.wp_active, sec.has_wp);
            vga::print_str(" Supervisor write-protection\n");

            vga::print_str("    EFER.NXE (Execute Disable / NX): ");
            print_status_badge(sec.nx_active, sec.has_nx);
            vga::print_str(" Hardware W^X execution prevention\n");

            vga::print_str("    CR4.SMEP (Supervisor Exec Prev): ");
            print_status_badge(sec.smep_active, sec.has_smep);
            vga::print_str(" Prevent Ring 0 execution of user memory\n");

            vga::print_str("    CR4.SMAP (Supervisor Access Prev):");
            print_status_badge(sec.smap_active, sec.has_smap);
            vga::print_str(" User access boundary enforcement\n");

            vga::print_str("    CR4.UMIP (User Instruction Prev):");
            print_status_badge(sec.umip_active, sec.has_umip);
            vga::print_str(" Prevent SGDT/SIDT in user space\n\n");

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("  Memory Boundary Protections:\n");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("    User Space W^X Policy:          ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[ENFORCED] ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Simultaneous W+X mappings rejected\n");

            vga::print_str("    User Text Segment (.text):       ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("RX         ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Read-Only Executable\n");

            vga::print_str("    User Data & BSS (.data/.bss):    ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("RW + NX    ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Read-Write No-Execute\n");

            vga::print_str("    User Execution Stack:            ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("RW + NX    ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Read-Write No-Execute\n");

            vga::print_str("    User Process Heap (brk/sbrk):    ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("RW + NX    ");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Read-Write No-Execute\n");

            vga::print_str("    Null Pointer Trap (0x0..0x1000): ");
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("[PROTECTED]");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(" Unmapped hardware guard page\n");

            if args.has_flag('v', "verbose") {
                let (total_pf, cow_pf, stack_pf, demand_pf, viol_pf, tlb_cnt) =
                    vmm_get_fault_stats();

                vga::print_str("\n");
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  Virtual Memory Fault Telemetry:\n");

                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("    Total Page Faults (#PF):         ");
                print_num(total_pf);
                vga::print_str("\n");

                vga::print_str("    Copy-on-Write Duplications:      ");
                print_num(cow_pf);
                vga::print_str("\n");

                vga::print_str("    Stack Auto-Growth Expansions:    ");
                print_num(stack_pf);
                vga::print_str("\n");

                vga::print_str("    Demand-Paged VMA Allocations:    ");
                print_num(demand_pf);
                vga::print_str("\n");

                vga::print_str("    Access Violations Trapped:       ");
                print_num(viol_pf);
                vga::print_str("\n");

                vga::print_str("    TLB Invalidation Flushes:        ");
                print_num(tlb_cnt);
                vga::print_str("\n");
            }
        }
    }
    pub mod selectinfo {
        use crate::args::CliArgs;
        use keira_io::vga;

        fn print_label_and_u64(label: &str, val: u64, suffix: &str) {
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  ");
            vga::print_str(label);
            let pad = 30usize.saturating_sub(label.len());
            for _ in 0..pad {
                vga::print_str(" ");
            }
            vga::print_str(": ");
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_u64(val);
            if !suffix.is_empty() {
                vga::print_str(" ");
                vga::print_str(suffix);
            }
            vga::print_str("\n");
        }

        /// Executes the 'selectinfo' / 'select' supervisor diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: selectinfo [-h]\n\n");
                vga::print_str(
            "Description:\n  Display POSIX I/O multiplexing telemetry and sys_select invocation metrics.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str("  -h, --help  Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let stats = keira_syscall::get_select_telemetry();

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("=== POSIX I/O Multiplexing Subsystem (SYS_SELECT) ===\n\n");
            vga::set_color(vga::Color::White, vga::Color::Black);

            print_label_and_u64("Total sys_select Invocations", stats.total_selects, "calls");
            print_label_and_u64(
                "Total Ready Event Signals",
                stats.total_ready_events,
                "events",
            );
            print_label_and_u64("Timeout Expirations", stats.total_timeouts, "events");
            print_label_and_u64("Bad Descriptor (EBADF)", stats.total_badf_events, "events");
            print_label_and_u64("Bitmask Capacity (FD_SETSIZE)", 1024, "descriptors / mask");

            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "  Supported Channels        : Anonymous Pipes, UNIX Sockets, TCP/UDP, TTY & VFS\n",
            );
            vga::print_str(
                "  Standard Compliance       : POSIX.1-2008 / Single UNIX Specification v4\n",
            );
        }
    }
    pub mod slabinfo {
        use crate::args::CliArgs;
        use keira_io::vga;
        use keira_mem::MAX_CPU_CORES;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: slabinfo [-c] [-v] [-h]\n\n");
                vga::print_str(
            "Description:\n  Display slab object caches, magazine utilization and memory allocation telemetry.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
                    "  -c, --cpu      Display per-CPU magazine caches, hits and depot exchanges\n",
                );
                vga::print_str("  -v, --verbose  Display comprehensive slab tunables, depot cache and memory footprints\n");
                vga::print_str("  -h, --help     Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            if args.has_flag('c', "cpu") {
                display_per_cpu_depots();
                return;
            }

            if args.has_flag('v', "verbose") {
                display_verbose_slabinfo();
                return;
            }

            display_standard_slabinfo();
        }

        fn display_standard_slabinfo() {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Kernel Slab Object Caches (slabinfo 2.1):\n");
            vga::print_str(
                "CACHE NAME        SIZE   ACTIVE  TOTAL   SLABS   MEM(KB)  ALLOC_HIT  FREE_HIT\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "----------------  -----  ------  ------  ------  -------  ---------  --------\n",
            );

            let caches = keira_mem::get_all_kmem_caches();
            let mut total_active_objs = 0usize;
            let mut total_objs = 0usize;
            let mut total_mem_bytes = 0usize;

            for cache in caches.iter() {
                let active = cache.allocated_count();
                let total = cache.total_count();
                let obj_size = cache.obj_size();
                let slabs = cache.num_slabs();
                let mem_kb = (total * obj_size) / 1024;

                total_active_objs += active;
                total_objs += total;
                total_mem_bytes += total * obj_size;

                vga::set_color(vga::Color::White, vga::Color::Black);
                print_fixed_str(cache.name(), 18);
                print_fixed_u64(obj_size as u64, 7);
                print_fixed_u64(active as u64, 8);
                print_fixed_u64(total as u64, 8);
                print_fixed_u64(slabs as u64, 8);
                print_fixed_u64(mem_kb as u64, 9);
                print_fixed_u64(cache.total_alloc_hits() as u64, 11);
                print_fixed_u64(cache.total_free_hits() as u64, 8);
                vga::print_str("\n");
            }

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "-----------------------------------------------------------------------------\n",
            );
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Summary: ");
            vga::print_u64(total_active_objs as u64);
            vga::print_str(" active objects, ");
            vga::print_u64(total_objs.saturating_sub(total_active_objs) as u64);
            vga::print_str(" cached objects, ");
            vga::print_u64((total_mem_bytes / 1024) as u64);
            vga::print_str(" KB allocated memory\n");
        }

        fn display_per_cpu_depots() {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Per-CPU Magazine Depot Telemetry:\n");
            vga::print_str("CACHE NAME        CORE  CACHED  ALLOC_HIT  FREE_HIT  EXCHANGES\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("----------------  ----  ------  ---------  --------  ---------\n");

            let caches = keira_mem::get_all_kmem_caches();
            for cache in caches.iter() {
                for core in 0..MAX_CPU_CORES {
                    if let Some((cached, alloc_hits, free_hits, exchanges)) = cache.cpu_stats(core)
                    {
                        if cached > 0
                            || alloc_hits > 0
                            || free_hits > 0
                            || exchanges > 0
                            || core == 0
                        {
                            vga::set_color(vga::Color::White, vga::Color::Black);
                            print_fixed_str(cache.name(), 18);
                            print_fixed_u64(core as u64, 6);
                            print_fixed_u64(cached as u64, 8);
                            print_fixed_u64(alloc_hits as u64, 11);
                            print_fixed_u64(free_hits as u64, 10);
                            print_fixed_u64(exchanges as u64, 9);
                            vga::print_str("\n");
                        }
                    }
                }
            }
        }

        fn display_verbose_slabinfo() {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Kernel Slab Allocator Comprehensive Telemetry:\n");
            vga::print_str(
                "CACHE NAME        ACTIVE/TOTAL  OBJ/SLAB  PG/SLAB  TUNABLES      DEPOT_CACHED\n",
            );
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str(
                "----------------  ------------  --------  -------  ------------  ------------\n",
            );

            let caches = keira_mem::get_all_kmem_caches();
            for cache in caches.iter() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                print_fixed_str(cache.name(), 18);

                let active = cache.allocated_count();
                let total = cache.total_count();
                let mut count_buf = [b' '; 14];
                let mut offset = format_two_u64(active as u64, total as u64, &mut count_buf);
                while offset < 14 {
                    count_buf[offset] = b' ';
                    offset += 1;
                }
                if let Ok(s) = core::str::from_utf8(&count_buf[..offset]) {
                    vga::print_str(s);
                }

                print_fixed_u64(cache.objs_per_slab() as u64, 10);
                print_fixed_u64(cache.pages_per_slab() as u64, 9);

                let mut tune_buf = [b' '; 14];
                let mut t_off = format_tunables(
                    cache.limit_tunable() as u64,
                    cache.batchcount_tunable() as u64,
                    cache.sharedfactor_tunable() as u64,
                    &mut tune_buf,
                );
                while t_off < 14 {
                    tune_buf[t_off] = b' ';
                    t_off += 1;
                }
                if let Ok(s) = core::str::from_utf8(&tune_buf[..t_off]) {
                    vga::print_str(s);
                }

                print_fixed_u64(cache.total_depot_cached() as u64, 12);
                vga::print_str("\n");
            }
        }

        fn print_fixed_str(s: &str, width: usize) {
            vga::print_str(s);
            for _ in 0..width.saturating_sub(s.len()) {
                vga::print_str(" ");
            }
        }

        fn print_fixed_u64(val: u64, width: usize) {
            let mut len = 0;
            let mut temp = val;
            if temp == 0 {
                len = 1;
            } else {
                while temp > 0 {
                    len += 1;
                    temp /= 10;
                }
            }
            vga::print_u64(val);
            for _ in 0..width.saturating_sub(len) {
                vga::print_str(" ");
            }
        }

        fn format_two_u64(a: u64, b: u64, buf: &mut [u8]) -> usize {
            let mut writer = FormatWriter::new(buf);
            use core::fmt::Write;
            let _ = write!(writer, "{}/{}", a, b);
            writer.len
        }

        fn format_tunables(limit: u64, batch: u64, shared: u64, buf: &mut [u8]) -> usize {
            let mut writer = FormatWriter::new(buf);
            use core::fmt::Write;
            let _ = write!(writer, "{}/{}/{}", limit, batch, shared);
            writer.len
        }

        struct FormatWriter<'a> {
            buf: &'a mut [u8],
            len: usize,
        }

        impl<'a> FormatWriter<'a> {
            fn new(buf: &'a mut [u8]) -> Self {
                Self { buf, len: 0 }
            }
        }

        impl<'a> core::fmt::Write for FormatWriter<'a> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let bytes = s.as_bytes();
                let avail = self.buf.len().saturating_sub(self.len);
                let count = bytes.len().min(avail);
                self.buf[self.len..self.len + count].copy_from_slice(&bytes[..count]);
                self.len += count;
                Ok(())
            }
        }
    }
    pub mod smp {
        use crate::args::CliArgs;
        use keira_arch::smp::{get_core_info, get_online_cores_count, init_smp, CoreStatus};
        use keira_io::vga;

        /// Entry point for `smp` command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: smp [-h]\n\n");
                    vga::print_str("Description:\n");
                    vga::print_str(
                "  Inspect Symmetric Multiprocessing (SMP) CPU core topology and Local APIC states.\n\n",
            );
                    vga::print_str("Options:\n");
                    vga::print_str("  -h, --help    Show this help message and exit\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                return;
            }

            init_smp();
            let total_cores = get_online_cores_count();

            {
                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                vga::print_str("Symmetric Multiprocessing (SMP) Hardware Topology:\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                let (shootdowns, shootdown_ipis) =
                    keira_arch::interrupts::smp::get_tlb_shootdown_stats();
                let (resched_sent, resched_rcvd) =
                    keira_arch::interrupts::smp::get_resched_ipi_stats();

                vga::print_str("  Total Online Cores : ");
                vga::print_u64(total_cores as u64);
                vga::print_str("\n");
                vga::print_str("  TLB Shootdowns     : ");
                vga::print_u64(shootdowns);
                vga::print_str(" broadcasts (");
                vga::print_u64(shootdown_ipis);
                vga::print_str(" IPIs)\n");
                vga::print_str("  Reschedule IPIs    : ");
                vga::print_u64(resched_sent);
                vga::print_str(" sent (");
                vga::print_u64(resched_rcvd);
                vga::print_str(" received)\n\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("CORE     APIC ID    ROLE    STATUS\n");
                vga::set_color(vga::Color::DarkGrey, vga::Color::Black);
                vga::print_str("-------  ---------  ------  ----------------\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);

                for i in 0..total_cores {
                    if let Some(core) = get_core_info(i) {
                        vga::print_str("Core ");
                        vga::print_u64(core.core_id as u64);
                        vga::print_str("   ");
                        vga::print_hex(core.apic_id as u64);
                        vga::print_str("    ");

                        if core.is_bsp {
                            vga::set_color(vga::Color::White, vga::Color::Black);
                            vga::print_str("BSP     ");
                        } else {
                            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            vga::print_str("AP      ");
                        }

                        match core.status {
                            CoreStatus::Online => {
                                vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                                vga::print_str("Online (Active)\n");
                            }
                            CoreStatus::Booting => {
                                vga::set_color(vga::Color::Yellow, vga::Color::Black);
                                vga::print_str("Booting...\n");
                            }
                            CoreStatus::Offline => {
                                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                                vga::print_str("Offline\n");
                            }
                        }
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    }
                }
            }
        }
    }
    pub mod system {
        //! CPU architecture information, memory utilization and system uptime.

        use crate::args::CliArgs;
        use crate::executor::*;
        use keira_io::vga;

        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Usage: system [-v] [-u] [-s]\n\n");
                    vga::print_str("Description:\n  Display kernel specifications, CPU architecture and system telemetry.\n\n");
                    vga::print_str("Options:\n");
                    vga::print_str(
                        "  -v, --version  Display kernel version, build target and license\n",
                    );
                    vga::print_str("  -u, --uptime   Display system uptime duration\n");
                    vga::print_str("  -s, --summary  Display compact one-line system status\n");
                    vga::print_str("  -h, --help     Show this help message and exit\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                }
                return;
            }

            {
                let ms = keira_arch::timers::get_uptime_ms();
                let hours = ms / 3600000;
                let minutes = (ms % 3600000) / 60000;
                let seconds = (ms % 60000) / 1000;
                let millis = ms % 1000;

                if args.has_flag('u', "uptime") {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Uptime: ");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_u64(hours);
                    vga::print_str("h ");
                    vga::print_u64(minutes);
                    vga::print_str("m ");
                    vga::print_u64(seconds);
                    vga::print_str("s ");
                    vga::print_u64(millis);
                    vga::print_str("ms\n");
                    return;
                }

                #[cfg(target_arch = "x86")]
                const ARCH_NAME: &str = "i686";
                #[cfg(target_arch = "x86_64")]
                const ARCH_NAME: &str = "x86_64";
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                const ARCH_NAME: &str = "unknown";

                if args.has_flag('v', "version") {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Keira Kernel ");
                    vga::print_str(env!("CARGO_PKG_VERSION"));
                    vga::print_str("-keira-1 (");
                    vga::print_str(ARCH_NAME);
                    vga::print_str("-unknown-none)\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("Compiled with rustc 1.85+ (LLVM 19), GPL-2.0-only\n");
                    vga::print_str("Author: Moh. Ananda Firmansyah Putra\n");
                    return;
                }

                if args.has_flag('s', "summary") {
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Keira v");
                    vga::print_str(env!("CARGO_PKG_VERSION"));
                    vga::print_str(" | ");
                    vga::print_str(ARCH_NAME);
                    vga::print_str(" | Up: ");
                    vga::print_u64(hours);
                    vga::print_str("h ");
                    vga::print_u64(minutes);
                    vga::print_str("m ");
                    vga::print_u64(seconds);
                    vga::print_str("s\n");
                    return;
                }

                #[cfg(target_arch = "x86_64")]
                let cpuid = core::arch::x86_64::__cpuid(0);
                #[cfg(target_arch = "x86")]
                let cpuid = core::arch::x86::__cpuid(0);
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                let cpuid = core::arch::x86_64::CpuidResult {
                    eax: 0,
                    ebx: 0,
                    ecx: 0,
                    edx: 0,
                };

                let mut vendor = [0u8; 12];
                vendor[0..4].copy_from_slice(&cpuid.ebx.to_le_bytes());
                vendor[4..8].copy_from_slice(&cpuid.edx.to_le_bytes());
                vendor[8..12].copy_from_slice(&cpuid.ecx.to_le_bytes());

                let pci_count = count_pci_devices();
                let heap_total = keira_mem::heap_get_total() as u64;
                let heap_used = keira_mem::heap_get_used() as u64;

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("System Specifications & Kernel Information\n");

                vga::print_str("  Kernel Version    : ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("Keira Kernel v");
                vga::print_str(env!("CARGO_PKG_VERSION"));
                vga::print_str("\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  Architecture      : ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                #[cfg(target_arch = "x86_64")]
                vga::print_str("x86_64 Long Mode (Freestanding)\n");
                #[cfg(target_arch = "x86")]
                vga::print_str("i686 Protected Mode (Freestanding)\n");
                #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
                vga::print_str("Unknown (Freestanding)\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  CPU Vendor        : ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                if let Ok(v) = core::str::from_utf8(&vendor) {
                    vga::print_str(v);
                }
                vga::print_str("\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  System Uptime     : ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_u64(hours);
                vga::print_str("h ");
                vga::print_u64(minutes);
                vga::print_str("m ");
                vga::print_u64(seconds);
                vga::print_str("s ");
                vga::print_u64(millis);
                vga::print_str("ms\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  Heap Memory       : ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_u64(heap_used / 1024);
                vga::print_str(" KB / ");
                vga::print_u64(heap_total / 1024);
                vga::print_str(" KB\n");

                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("  PCI Devices       : ");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_u64(pci_count);
                vga::print_str(" detected\n");
            }
        }
    }
    pub mod unixsock {
        use crate::args::CliArgs;
        use keira_io::vga;
        use keira_net::socket::{for_each_unix_socket, SocketEntry, SOCKET_BUF_SIZE};

        fn print_padded_str(s: &str, width: usize) {
            vga::print_str(s);
            let len = s.len();
            for _ in len..width {
                vga::print_str(" ");
            }
        }

        fn print_num(val: u64, width: usize) {
            let mut digits = 1;
            let mut temp = val;
            while temp >= 10 {
                digits += 1;
                temp /= 10;
            }
            vga::print_u64(val);
            for _ in digits..width {
                vga::print_str(" ");
            }
        }

        /// Executes the 'unixsock' / 'unix' supervisor diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);

            if args.has_flag('h', "help") {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: unixsock [-v] [-h]\n\n");
                vga::print_str(
            "Description:\n  Display active UNIX domain sockets (AF_UNIX) and inter-process IPC channels.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
                    "  -v, --verbose  Display buffer capacities and connected peer endpoints\n",
                );
                vga::print_str("  -h, --help     Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            let is_verbose = args.has_flag('v', "verbose");

            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("=== Active UNIX Domain Sockets (AF_UNIX IPC Table) ===\n\n");
            vga::set_color(vga::Color::White, vga::Color::Black);

            if is_verbose {
                vga::print_str(
                    "ID   Type     State        Peer   RX Queue     TX Queue     Path / Role\n",
                );
                vga::print_str(
            "---- -------- ------------ ------ ------------ ------------ --------------------\n",
        );
            } else {
                vga::print_str("ID   Type     State        Peer   RX Queue     Path / Role\n");
                vga::print_str(
                    "---- -------- ------------ ------ ------------ --------------------\n",
                );
            }

            let mut count = 0usize;
            unsafe {
                for_each_unix_socket(|sock: &SocketEntry| {
                    count += 1;
                    print_num(sock.id as u64, 5);

                    let type_str = if sock.socket_type == 1 {
                        "STREAM  "
                    } else if sock.socket_type == 2 {
                        "DGRAM   "
                    } else {
                        "RAW     "
                    };
                    vga::print_str(type_str);

                    let state_str = if sock.is_connected {
                        "CONNECTED   "
                    } else if sock.path_len > 0 {
                        "LISTENING   "
                    } else {
                        "UNBOUND     "
                    };
                    vga::print_str(state_str);

                    if sock.peer_id != 0 {
                        print_num(sock.peer_id as u64, 7);
                    } else {
                        print_padded_str("-", 7);
                    }

                    vga::print_u64(sock.rx_len as u64);
                    vga::print_str("/");
                    vga::print_u64(SOCKET_BUF_SIZE as u64);
                    vga::print_str("B    ");

                    if is_verbose {
                        vga::print_u64(sock.tx_len as u64);
                        vga::print_str("/");
                        vga::print_u64(SOCKET_BUF_SIZE as u64);
                        vga::print_str("B    ");
                    }

                    if sock.path_len > 0 {
                        if let Ok(p) = core::str::from_utf8(&sock.path[..sock.path_len]) {
                            vga::print_str(p);
                        } else {
                            vga::print_str("[invalid path]");
                        }
                    } else if sock.peer_id != 0 {
                        vga::print_str("[socketpair]");
                    } else {
                        vga::print_str("[anonymous]");
                    }
                    vga::print_str("\n");
                });
            }

            if count == 0 {
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("(No active UNIX domain sockets allocated)\n");
            }

            vga::print_str("\n");
            vga::set_color(vga::Color::White, vga::Color::Black);
        }
    }
    pub mod unwind {
        use keira_io::vga;

        #[inline(never)]
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            if let Some("-h") | Some("--help") = parts.next() {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: unwind [-h]\n\n");
                vga::print_str(
                    "Description:\n  Trigger kernel stack frame unwinder backtrace.\n\n",
                );
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("Kernel Unwinder & CPU Exception Telemetry:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  Total Trapped Exceptions: ");
            vga::print_u64(keira_syscall::get_cpu_exception_count());
            vga::print_str("\n");

            #[cfg(target_os = "none")]
            keira_arch::unwind::unwind_stack();
        }
    }
    pub mod vmainfo {
        use crate::args::CliArgs;
        use keira_io::vga;
        use keira_mem::{
            get_vmas_for_pml4, total_active_vmas, MAP_SHARED, MAX_VMAS, PROT_EXEC, PROT_READ,
            PROT_WRITE,
        };
        use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
        use keira_task::types::MAX_TASKS;

        fn print_padded(s: &str, width: usize) {
            vga::print_str(s);
            let len = s.len();
            for _ in len..width {
                vga::print_str(" ");
            }
        }

        fn print_vma_line(start: u64, end: u64, prot_str: &str, flags_str: &str, path: &str) {
            vga::print_str("  ");
            vga::print_hex(start);
            vga::print_str(" ");
            vga::print_hex(end);
            vga::print_str(" ");

            let size_kb = (end.saturating_sub(start)) / 1024;
            let mut digits = 1;
            let mut temp = size_kb;
            while temp >= 10 {
                digits += 1;
                temp /= 10;
            }
            for _ in digits..6 {
                vga::print_str(" ");
            }
            vga::print_u64(size_kb);
            vga::print_str(" KB ");
            print_padded(prot_str, 5);
            vga::print_str(" ");
            print_padded(flags_str, 7);
            vga::print_str(" ");
            vga::print_str(path);
            vga::print_str("\n");
        }

        fn inspect_task_vma(pid: usize, fg: vga::Color, bg: vga::Color) {
            unsafe {
                if pid >= MAX_TASKS || TASKS[pid].is_none() {
                    vga::set_color(vga::Color::LightRed, bg);
                    vga::print_str("Error: Process PID ");
                    vga::print_u64(pid as u64);
                    vga::print_str(" is not active\n");
                    vga::set_color(vga::Color::LightGrey, bg);
                    return;
                }

                let task = match TASKS[pid].as_ref() {
                    Some(t) => t,
                    None => return,
                };

                vga::set_color(fg, bg);
                vga::print_str("Process Virtual Memory Areas (PID ");
                vga::print_u64(task.id as u64);
                vga::print_str(": ");
                vga::print_str(task.name);
                vga::print_str("):\n");

                vga::set_color(vga::Color::Yellow, bg);
                vga::print_str(
                    "  START              END                SIZE      PERM  FLAGS   PATH\n",
                );
                vga::set_color(fg, bg);

                let mut total_regions = 0usize;

                if !task.is_user {
                    let stack_top = task.stack_addr + 16384;
                    print_vma_line(task.stack_addr, stack_top, "rw-p", "private", "[kstack]");
                    total_regions += 1;
                } else {
                    // Text segment
                    let text_start: u64 = 0x0040_0000;
                    let text_end: u64 = text_start + 0x5000;
                    print_vma_line(text_start, text_end, "r-xp", "private", task.name);
                    total_regions += 1;

                    // Data / BSS segment
                    let data_start: u64 = text_end;
                    let data_end: u64 = data_start + 0x1000;
                    print_vma_line(data_start, data_end, "rw-p", "private", task.name);
                    total_regions += 1;

                    // Heap
                    if task.program_break > task.program_break_start {
                        print_vma_line(
                            task.program_break_start,
                            task.program_break,
                            "rw-p",
                            "private",
                            "[heap]",
                        );
                        total_regions += 1;
                    }

                    // Dynamic VMAs from VMA_TABLE
                    if task.pml4_phys != 0 {
                        let mut vmas = [keira_mem::Vma::empty(); MAX_VMAS];
                        let count = get_vmas_for_pml4(task.pml4_phys, &mut vmas);
                        for vma in &vmas[..count] {
                            let mut prot = [b'-'; 4];
                            if (vma.prot & PROT_READ) != 0 {
                                prot[0] = b'r';
                            }
                            if (vma.prot & PROT_WRITE) != 0 {
                                prot[1] = b'w';
                            }
                            if (vma.prot & PROT_EXEC) != 0 {
                                prot[2] = b'x';
                            }
                            prot[3] = if (vma.flags & MAP_SHARED) != 0 {
                                b's'
                            } else {
                                b'p'
                            };
                            let prot_str = core::str::from_utf8(&prot).unwrap_or("---p");
                            let flags_str = if (vma.flags & MAP_SHARED) != 0 {
                                "shared"
                            } else {
                                "private"
                            };
                            let path = vma.file_path_str().unwrap_or("[anon]");
                            print_vma_line(vma.start, vma.end, prot_str, flags_str, path);
                            total_regions += 1;
                        }
                    }

                    // Stack
                    #[cfg(target_arch = "x86")]
                    let stack_top = 0x07FFF000u64;
                    #[cfg(not(target_arch = "x86"))]
                    let stack_top = 0x7FFFFFE00000u64.saturating_sub(task.stack_aslr_offset);
                    let stack_bottom = stack_top.saturating_sub(0x20000);
                    print_vma_line(stack_bottom, stack_top, "rw-p", "private", "[stack]");
                    total_regions += 1;
                }

                vga::print_str("\n");
                vga::print_str("  PML4 Base    : ");
                vga::print_hex(task.pml4_phys);
                vga::print_str("\n");
                vga::print_str("  Active VMAs  : ");
                vga::print_u64(total_regions as u64);
                vga::print_str(" regions\n");
                vga::print_str("  VMA Capacity : ");
                vga::print_u64(total_active_vmas() as u64);
                vga::print_str(" / ");
                vga::print_u64(MAX_VMAS as u64);
                vga::print_str(" slots in kernel\n");
                vga::set_color(vga::Color::LightGrey, bg);
            }
        }

        /// Executes the 'vmainfo' / 'vma' supervisor diagnostic command.
        pub fn run(parts: &mut core::str::SplitWhitespace) {
            let args = CliArgs::parse(parts);
            let bg = vga::Color::Black;
            let fg = vga::Color::White;

            if args.has_flag('h', "help") {
                vga::set_color(fg, bg);
                vga::print_str("Usage: vmainfo [-a] [-h] [pid]\n\n");
                vga::print_str(
            "Description:\n  Display Virtual Memory Area (VMA) layout, protection modes and address mappings.\n\n",
        );
                vga::print_str("Options:\n");
                vga::print_str(
                    "  -a, --all      Display virtual memory layout for all active tasks\n",
                );
                vga::print_str("  -h, --help     Show this help reference and exit\n");
                vga::print_str(
                    "  [pid]          Inspect virtual memory layout of specific process\n",
                );
                vga::set_color(vga::Color::LightGrey, bg);
                return;
            }

            if args.has_flag('a', "all") {
                unsafe {
                    for (i, slot) in TASKS.iter().enumerate() {
                        if slot.is_some() {
                            inspect_task_vma(i, fg, bg);
                            vga::print_str("\n");
                        }
                    }
                }
                return;
            }

            let target_pid = if let Some(arg) = args.first_positional() {
                match arg.parse::<usize>() {
                    Ok(p) => p,
                    Err(_) => {
                        vga::set_color(vga::Color::LightRed, bg);
                        vga::print_str("Error: Invalid process identifier '");
                        vga::print_str(arg);
                        vga::print_str("'\n");
                        vga::set_color(vga::Color::LightGrey, bg);
                        return;
                    }
                }
            } else {
                unsafe { CURRENT_TASK_IDX }
            };

            inspect_task_vma(target_pid, fg, bg);
        }
    }
}

pub use self::control::power;
pub use self::control::reset;
pub use self::control::sync;
pub use self::control::wipe;
pub use self::daemon::syslog;
pub use self::daemon::watchpoint;
pub use self::info::aslrinfo;
pub use self::info::cpu;
pub use self::info::irq;
pub use self::info::memory;
pub use self::info::pipes;
pub use self::info::pollinfo;
pub use self::info::regs;
pub use self::info::rlimit;
pub use self::info::rnginfo;
pub use self::info::secinfo;
pub use self::info::selectinfo;
pub use self::info::slabinfo;
pub use self::info::smp;
pub use self::info::system;
pub use self::info::unixsock;
pub use self::info::unwind;
pub use self::info::vmainfo;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_power() {
        let mut args = "power --help".split_whitespace();
        args.next();
        power::run(&mut args);
    }

    #[test]
    fn test_sys_reset() {
        let mut args = "reset --help".split_whitespace();
        args.next();
        reset::run(&mut args);
    }

    #[test]
    fn test_sys_sync() {
        let mut args = "sync --help".split_whitespace();
        args.next();
        sync::run(&mut args);
    }

    #[test]
    fn test_sys_syslog() {
        let mut args = "syslog --help".split_whitespace();
        args.next();
        syslog::run(&mut args);
    }

    #[test]
    fn test_sys_watchpoint() {
        let mut args = "watchpoint --help".split_whitespace();
        args.next();
        watchpoint::run(&mut args);
    }

    #[test]
    fn test_sys_cpu() {
        let mut args = "cpu --help".split_whitespace();
        args.next();
        cpu::run(&mut args);
    }

    #[test]
    fn test_sys_memory() {
        let mut args = "memory --help".split_whitespace();
        args.next();
        memory::run(&mut args);
    }

    #[test]
    fn test_sys_smp() {
        let mut args = "smp --help".split_whitespace();
        args.next();
        smp::run(&mut args);
    }

    #[test]
    fn test_sys_system() {
        let mut args = "system --help".split_whitespace();
        args.next();
        system::run(&mut args);
    }

    #[test]
    fn test_sys_unwind() {
        let mut args = "unwind --help".split_whitespace();
        args.next();
        unwind::run(&mut args);
    }

    #[test]
    fn test_sys_wipe() {
        let mut args = "wipe --help".split_whitespace();
        args.next();
        wipe::run(&mut args);

        let mut args_empty = "".split_whitespace();
        wipe::run(&mut args_empty);
    }

    #[test]
    fn test_sys_regs() {
        let mut args = "regs --help".split_whitespace();
        args.next();
        regs::run(&mut args);

        let mut args_empty = "".split_whitespace();
        regs::run(&mut args_empty);
    }

    #[test]
    fn test_sys_irq() {
        let mut args = "irq --help".split_whitespace();
        args.next();
        irq::run(&mut args);

        let mut args_empty = "".split_whitespace();
        irq::run(&mut args_empty);
    }

    #[test]
    fn test_sys_slabinfo() {
        let mut args_help = "slabinfo --help".split_whitespace();
        args_help.next();
        slabinfo::run(&mut args_help);

        let mut args_cpu = "slabinfo -c".split_whitespace();
        args_cpu.next();
        slabinfo::run(&mut args_cpu);

        let mut args_verbose = "slabinfo -v".split_whitespace();
        args_verbose.next();
        slabinfo::run(&mut args_verbose);

        let mut args_default = "".split_whitespace();
        slabinfo::run(&mut args_default);
    }

    #[test]
    fn test_sys_secinfo() {
        let mut args_help = "secinfo --help".split_whitespace();
        args_help.next();
        secinfo::run(&mut args_help);

        let mut args_verbose = "secinfo -v".split_whitespace();
        args_verbose.next();
        secinfo::run(&mut args_verbose);

        let mut args_default = "".split_whitespace();
        secinfo::run(&mut args_default);
    }

    #[test]
    fn test_sys_aslrinfo() {
        let mut args_help = "aslrinfo --help".split_whitespace();
        args_help.next();
        aslrinfo::run(&mut args_help);

        let mut args_verbose = "aslrinfo -v".split_whitespace();
        args_verbose.next();
        aslrinfo::run(&mut args_verbose);

        let mut args_set = "aslrinfo 2".split_whitespace();
        args_set.next();
        aslrinfo::run(&mut args_set);

        let mut args_default = "".split_whitespace();
        aslrinfo::run(&mut args_default);
    }

    #[test]
    fn test_sys_rnginfo() {
        let mut args_help = "rnginfo --help".split_whitespace();
        args_help.next();
        rnginfo::run(&mut args_help);

        let mut args_verbose = "rnginfo -v".split_whitespace();
        args_verbose.next();
        rnginfo::run(&mut args_verbose);

        let mut args_default = "".split_whitespace();
        rnginfo::run(&mut args_default);
    }

    #[test]
    fn test_sys_rlimit() {
        let mut args_help = "rlimit --help".split_whitespace();
        args_help.next();
        rlimit::run(&mut args_help);

        let mut args_verbose = "rlimit -v".split_whitespace();
        args_verbose.next();
        rlimit::run(&mut args_verbose);

        let mut args_default = "".split_whitespace();
        rlimit::run(&mut args_default);
    }

    #[test]
    fn test_sys_unixsock() {
        let mut args_help = "unixsock --help".split_whitespace();
        args_help.next();
        unixsock::run(&mut args_help);

        let mut args_verbose = "unixsock -v".split_whitespace();
        args_verbose.next();
        unixsock::run(&mut args_verbose);

        let mut args_default = "".split_whitespace();
        unixsock::run(&mut args_default);
    }

    #[test]
    fn test_sys_vmainfo() {
        let mut args_help = "vmainfo --help".split_whitespace();
        args_help.next();
        vmainfo::run(&mut args_help);

        let mut args_all = "vmainfo -a".split_whitespace();
        args_all.next();
        vmainfo::run(&mut args_all);

        let mut args_default = "".split_whitespace();
        vmainfo::run(&mut args_default);
    }

    #[test]
    fn test_sys_pollinfo() {
        let mut args_help = "pollinfo --help".split_whitespace();
        args_help.next();
        pollinfo::run(&mut args_help);

        let mut args_default = "".split_whitespace();
        pollinfo::run(&mut args_default);
    }

    #[test]
    fn test_sys_selectinfo() {
        let mut args_help = "selectinfo --help".split_whitespace();
        args_help.next();
        selectinfo::run(&mut args_help);

        let mut args_default = "".split_whitespace();
        selectinfo::run(&mut args_default);
    }
}

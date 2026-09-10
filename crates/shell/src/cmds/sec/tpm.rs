// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Trusted Platform Module (TPM 2.0) hardware security enclave, PCR banks, and measured boot log (Syscall 79).

use keira_crypto::tpm::{
    extend_pcr, get_event_log_slice, get_status, quote_pcrs, read_pcr, TPM_PCR_COUNT,
};
use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    let subcmd = parts.next();

    if let Some("-h") | Some("--help") = subcmd {
        print_help();
        return;
    }

    match subcmd {
        None | Some("status") => {
            print_status();
        }
        Some("pcr") => {
            if let Some(idx_str) = parts.next() {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    print_single_pcr(idx);
                } else {
                    print_err("Invalid PCR index (0..23).");
                }
            } else {
                print_all_pcrs();
            }
        }
        Some("extend") => {
            let idx_opt = parts.next().and_then(|s| s.parse::<usize>().ok());
            let data_opt = parts.next();

            if let (Some(idx), Some(data)) = (idx_opt, data_opt) {
                match extend_pcr(idx, data.as_bytes(), "SHELL_EXTEND") {
                    Ok(new_digest) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str("TPM2_PCR_Extend successful on PCR[");
                        vga::print_u64(idx as u64);
                        vga::print_str("].\n");

                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("     New Digest: ");
                        print_hex_bytes(&new_digest);
                        vga::print_str("\n");
                    }
                    Err(e) => {
                        print_err(e);
                    }
                }
            } else {
                print_err("Usage: tpm extend <pcr_index> <data_string>");
            }
        }
        Some("log") => {
            print_event_log();
        }
        Some("quote") => {
            let mask = parts
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0x0000_00FF); // Default PCR 0..7 platform mask
            let quote = quote_pcrs(mask);

            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("TPM 2.0 Attestation Quote Generated:\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("  PCR Selection Mask : 0x");
            print_hex_u32(mask);
            vga::print_str("\n  Quote Digest (SHA2): ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            print_hex_bytes(&quote);
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            vga::print_str("\n");
        }
        Some(other) => {
            print_err("Unknown subcommand. Run 'tpm --help' for usage.");
        }
    }
}

fn print_status() {
    let status = get_status();

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("TPM 2.0 Hardware Security Enclave Status:\n");
    vga::print_str("  Device State      : ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("Active (TCG TPM 2.0 Specification rev 01.59)\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Locality Base     : 0x");
    print_hex_u64(status.mmio_base);
    vga::print_str(" (Locality 0 MMIO Enclave)\n");
    vga::print_str("  Hardware Probe    : VID 0x");
    vga::print_hex(status.hardware.vendor_id as u64);
    vga::print_str(" DID 0x");
    vga::print_hex(status.hardware.device_id as u64);
    vga::print_str(if status.hardware.present {
        " [CONNECTED]\n"
    } else {
        " [NO HARDWARE / FLOATING]\n"
    });

    vga::print_str("  Active PCR Banks  : SHA-256 (24 Registers [PCR 0..23])\n");
    vga::print_str("  Total Measurements: ");
    vga::print_u64(status.total_measurements);
    vga::print_str("\n  Event Log Records : ");
    vga::print_u64(status.event_count as u64);
    vga::print_str("\n  Syscall Interface : Syscall 79 (SYS_TPM2)\n");
}

fn print_single_pcr(idx: usize) {
    match read_pcr(idx) {
        Ok(digest) => {
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("PCR[");
            if idx < 10 {
                vga::print_str("0");
            }
            vga::print_u64(idx as u64);
            vga::print_str("] (SHA-256): ");
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            print_hex_bytes(&digest);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        }
        Err(e) => {
            print_err(e);
        }
    }
}

fn print_all_pcrs() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("TPM 2.0 SHA-256 Platform Configuration Registers (PCRs):\n");

    for idx in 0..TPM_PCR_COUNT {
        if let Ok(digest) = read_pcr(idx) {
            vga::set_color(vga::Color::Cyan, vga::Color::Black);
            vga::print_str("  PCR[");
            if idx < 10 {
                vga::print_str("0");
            }
            vga::print_u64(idx as u64);
            vga::print_str("]: ");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            print_hex_bytes(&digest);

            // Add annotation for standard TCG assignments
            match idx {
                0 => vga::print_str(" (Firmware / BIOS)"),
                1 => vga::print_str(" (Host Config)"),
                2 => vga::print_str(" (Option ROM)"),
                4 => vga::print_str(" (Bootloader / Kernel)"),
                7 => vga::print_str(" (Secure Boot Policy)"),
                10 => vga::print_str(" (IMA / Runtime)"),
                _ => {}
            }
            vga::print_str("\n");
        }
    }
}

fn print_event_log() {
    let log = unsafe { get_event_log_slice() };

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("TPM 2.0 Measured Boot & Integrity Event Log:\n");
    vga::set_color(vga::Color::Cyan, vga::Color::Black);
    vga::print_str("  PCR  TYPE        EVENT NAME           SHA-256 DIGEST (PREFIX)\n");
    vga::print_str("  ---  ----------  -------------------  -----------------------\n");

    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    let mut count = 0;
    for ev_opt in log.iter() {
        if let Some(ev) = ev_opt {
            count += 1;
            vga::print_str("  [");
            if ev.pcr_index < 10 {
                vga::print_str("0");
            }
            vga::print_u64(ev.pcr_index as u64);
            vga::print_str("]  0x");
            print_hex_u32(ev.event_type);
            vga::print_str("  ");

            if let Ok(desc) = core::str::from_utf8(&ev.desc[..ev.desc_len]) {
                vga::print_str(desc);
                for _ in 0..(21usize.saturating_sub(desc.len())) {
                    vga::print_str(" ");
                }
            } else {
                vga::print_str("EVENT_UNKNOWN        ");
            }

            // Print first 8 bytes of hash for concise display
            print_hex_bytes(&ev.digest[..8]);
            vga::print_str("...\n");
        }
    }

    if count == 0 {
        vga::print_str("  Event log is currently empty.\n");
    }
}

fn print_hex_bytes(bytes: &[u8]) {
    let hex_chars = b"0123456789abcdef";
    for &b in bytes {
        let hi = hex_chars[(b >> 4) as usize];
        let lo = hex_chars[(b & 0x0F) as usize];
        let slice = [hi, lo];
        if let Ok(s) = core::str::from_utf8(&slice) {
            vga::print_str(s);
        }
    }
}

fn print_hex_u32(val: u32) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 8];
    for i in 0..8 {
        let shift = (7 - i) * 4;
        buf[i] = hex_chars[((val >> shift) & 0x0F) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

fn print_hex_u64(val: u64) {
    let hex_chars = b"0123456789ABCDEF";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        let shift = (15 - i) * 4;
        buf[i] = hex_chars[((val >> shift) & 0x0F) as usize];
    }
    if let Ok(s) = core::str::from_utf8(&buf) {
        vga::print_str(s);
    }
}

fn print_help() {
    vga::print_str("Usage: tpm [status|pcr [idx]|extend <idx> <data>|log|quote [mask]]\n\n");
    vga::print_str("Description:\n");
    vga::print_str("  Inspect and manage Trusted Platform Module (TPM 2.0) hardware security enclave (Syscall 79).\n\n");
    vga::print_str("Subcommands:\n");
    vga::print_str("  status                         Show TPM 2.0 hardware controller state and capabilities (default)\n");
    vga::print_str("  pcr [idx]                      Dump SHA-256 Platform Configuration Register banks or specific index\n");
    vga::print_str("  extend <idx> <data>            Cryptographically extend PCR digest: SHA256(PCR || SHA256(data))\n");
    vga::print_str(
        "  log                            Dump measured boot and runtime integrity event log\n",
    );
    vga::print_str("  quote [mask]                   Generate cryptographic attestation quote over selected PCRs\n");
    vga::print_str("  -h, --help                     Show this help message and exit\n");
}

fn print_err(msg: &str) {
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("[ERROR] ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(msg);
    vga::print_str("\n");
}

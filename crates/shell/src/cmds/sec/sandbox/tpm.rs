// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Trusted Platform Module (TPM 2.0) hardware security enclave, PCR banks, sealed storage, and measured boot log (Syscall 79).

#![allow(unused_variables, unused_unsafe)]

use keira_crypto::tpm::{
    extend_pcr, get_event_log_slice, get_status, quote_pcrs, read_pcr, seal_secret, unseal_secret,
    TpmSealedBlob, TPM_PCR_COUNT,
};
use keira_io::vga;

/// Active sealed storage slot in kernel security buffer for shell verification.
static mut ACTIVE_SEALED_BLOB: Option<TpmSealedBlob> = None;

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
                .unwrap_or(0x0000_00FF);
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
        Some("seal") => {
            let first = parts.next();
            let second = parts.next();

            let (mask, secret_str) = match (first, second) {
                (Some(m_str), Some(s_str)) => {
                    let mask_val = parse_mask(m_str).unwrap_or(0x0000_00FF);
                    (mask_val, s_str)
                }
                (Some(s_str), None) => (0x0000_00FF, s_str),
                _ => {
                    print_err("Usage: tpm seal [pcr_mask] <secret_data>");
                    return;
                }
            };

            match seal_secret(secret_str.as_bytes(), mask) {
                Ok(blob) => {
                    unsafe {
                        ACTIVE_SEALED_BLOB = Some(blob);
                    }
                    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                    vga::print_str("[OK] ");
                    vga::set_color(vga::Color::White, vga::Color::Black);
                    vga::print_str("Secret sealed successfully under PCR mask 0x");
                    print_hex_u32(mask);
                    vga::print_str("\n");

                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    vga::print_str("  Expected Quote: ");
                    print_hex_bytes(&blob.expected_quote);
                    vga::print_str("\n  AES-GCM AuthTag: ");
                    print_hex_bytes(&blob.auth_tag);
                    vga::print_str("\n  Ciphertext Len: ");
                    vga::print_u64(blob.data_len as u64);
                    vga::print_str(" bytes\n");
                }
                Err(e) => {
                    print_err(e);
                }
            }
        }
        Some("unseal") => {
            let blob_opt = unsafe { ACTIVE_SEALED_BLOB };
            if let Some(blob) = blob_opt {
                let mut out_buf = [0u8; 128];
                match unseal_secret(&blob, &mut out_buf) {
                    Ok(len) => {
                        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
                        vga::print_str("[OK] ");
                        vga::set_color(vga::Color::White, vga::Color::Black);
                        vga::print_str(
                            "Secret unsealed successfully (PCR policy attestation verified):\n",
                        );

                        vga::set_color(vga::Color::Yellow, vga::Color::Black);
                        vga::print_str("  Plaintext: ");
                        if let Ok(s) = core::str::from_utf8(&out_buf[..len]) {
                            vga::print_str(s);
                        } else {
                            print_hex_bytes(&out_buf[..len]);
                        }
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        vga::print_str("\n");
                    }
                    Err(e) => {
                        print_err(e);
                    }
                }
            } else {
                print_err("No active sealed secret. Run 'tpm seal <secret>' first.");
            }
        }
        Some("test") => {
            run_selftest();
        }
        Some(other) => {
            print_err("Unknown subcommand. Run 'tpm --help' for usage.");
        }
    }
}

fn parse_mask(s: &str) -> Option<u32> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse::<u32>().ok()
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
            vga::set_color(vga::Color::White, vga::Color::Black);
            vga::print_str("  PCR[");
            if idx < 10 {
                vga::print_str("0");
            }
            vga::print_u64(idx as u64);
            vga::print_str("]: ");

            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            print_hex_bytes(&digest);

            match idx {
                0 => vga::print_str(" (Firmware / BIOS IVT)"),
                1 => vga::print_str(" (Host Config)"),
                2 => vga::print_str(" (Host Bus Topology)"),
                4 => vga::print_str(" (Kernel .text Segment)"),
                5 => vga::print_str(" (Initrd USTAR Archive)"),
                7 => vga::print_str(" (Secure Boot Policy)"),
                10 => vga::print_str(" (IMA / Executed Binary)"),
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
    vga::set_color(vga::Color::White, vga::Color::Black);
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

            print_hex_bytes(&ev.digest[..8]);
            vga::print_str("...\n");
        }
    }

    if count == 0 {
        vga::print_str("  Event log is currently empty.\n");
    }
}

fn run_selftest() {
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Running Bare-Metal TPM 2.0 Security Subsystem Selftest:\n");

    // 1. Attestation Quote
    let mask = 0x0000_0013; // PCR 0, 1, 4
    let quote = quote_pcrs(mask);
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [1/4] Generating Attestation Quote (PCR 0,1,4): ");
    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("OK\n");

    // 2. Sealing Secret
    let secret = b"KEIRA_MASTER_SECURITY_KEY";
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [2/4] Sealing Data under PCR Mask 0x00000013: ");
    let blob = match seal_secret(secret, mask) {
        Ok(b) => {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("OK\n");
            b
        }
        Err(e) => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("FAIL (");
            vga::print_str(e);
            vga::print_str(")\n");
            return;
        }
    };

    // 3. Unsealing in Authenticated State
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [3/4] Unsealing Secret (Authentic State): ");
    let mut out_buf = [0u8; 64];
    match unseal_secret(&blob, &mut out_buf) {
        Ok(len) if &out_buf[..len] == secret => {
            vga::set_color(vga::Color::LightGreen, vga::Color::Black);
            vga::print_str("OK (Verified)\n");
        }
        _ => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("FAIL (Plaintext mismatch)\n");
            return;
        }
    }

    // 4. Platform State Tampering & Rejection Verification
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  [4/4] Tamper Detection (PCR Extension & Rejection): ");
    let _ = extend_pcr(0, b"UNAUTHORIZED_FIRMWARE_MODIFICATION", "ATTACK_TEST");
    let unseal_tampered = unseal_secret(&blob, &mut out_buf);
    if unseal_tampered.is_err() {
        vga::set_color(vga::Color::LightGreen, vga::Color::Black);
        vga::print_str("OK (Correctly Rejected)\n");
    } else {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("FAIL (Unseal succeeded after tamper!)\n");
        return;
    }

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("\n[PASS] All TPM 2.0 Hardware Security Assertions Verified Successfully.\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
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
    vga::print_str("Usage: tpm [status|pcr [idx]|extend <idx> <data>|seal [mask] <secret>|unseal|log|quote [mask]|test]\n\n");
    vga::print_str("Description:\n");
    vga::print_str("  Inspect and manage Trusted Platform Module (TPM 2.0) hardware security enclave (Syscall 79).\n\n");
    vga::print_str("Subcommands:\n");
    vga::print_str("  status                         Show TPM 2.0 hardware controller state and capabilities (default)\n");
    vga::print_str("  pcr [idx]                      Dump SHA-256 Platform Configuration Register banks or specific index\n");
    vga::print_str("  extend <idx> <data>            Cryptographically extend PCR digest: SHA256(PCR || SHA256(data))\n");
    vga::print_str("  seal [mask] <secret>           Seal secret data bound to PCR attestation policy mask (HKDF/AES-GCM)\n");
    vga::print_str("  unseal                         Unseal active secret data, validating PCR policy and GMAC tag\n");
    vga::print_str(
        "  log                            Dump measured boot and runtime integrity event log\n",
    );
    vga::print_str("  quote [mask]                   Generate cryptographic attestation quote over selected PCRs\n");
    vga::print_str(
        "  test                           Run automated bare-metal TPM 2.0 security selftest\n",
    );
    vga::print_str("  -h, --help                     Show this help message and exit\n");
}

fn print_err(msg: &str) {
    vga::set_color(vga::Color::LightRed, vga::Color::Black);
    vga::print_str("[ERROR] ");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(msg);
    vga::print_str("\n");
}

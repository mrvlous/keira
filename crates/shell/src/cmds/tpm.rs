// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//!
//! Inspect TPM 2.0 enclave status & PCR SHA-256 measurement banks.

use keira_io::vga;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        unsafe {
            vga::print_str("Usage: tpm [pcr|status]\n\n");
            vga::print_str("Description:\n  Inspect TPM 2.0 hardware security enclave & PCR SHA-256 measurement banks.\n\n");
            vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        }
        return;
    }

    unsafe {
        vga::set_color(vga::Color::LightCyan, vga::Color::Black);
        vga::print_str("TPM 2.0 Hardware Security Enclave Status\n");
        keira_crypto::tpm::init();
        let _ = keira_crypto::tpm::read_pcr(0);
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }
}

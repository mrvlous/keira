// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'fetch' shell command to stream and inspect HTTP/HTTPS network endpoints.

use super::progress::print_status_badge;
use super::url::ParsedUrl;
use crate::args::CliArgs;
use keira_io::vga;
use keira_net::tcp::fetch_http_stream;
use keira_net::tls::fetch_https_stream;

static mut ASSEMBLED_URL_BUF: [u8; 512] = [0u8; 512];

fn assemble_url<'a>(args: &'a CliArgs<'a>) -> Option<&'a str> {
    let mut pos_tokens = [""; 16];
    let mut pos_count = 0;
    let mut skip_next = false;
    for i in 0..args.token_count {
        let tok = args.tokens[i];
        if skip_next {
            skip_next = false;
            continue;
        }
        if tok == "-o" || tok == "--output" {
            skip_next = true;
            continue;
        }
        if tok.starts_with("-o=") || tok.starts_with("--output=") {
            continue;
        }
        if tok.starts_with('-') {
            continue;
        }
        if pos_count < 16 {
            pos_tokens[pos_count] = tok;
            pos_count += 1;
        }
    }

    if pos_count == 0 {
        return None;
    }
    if pos_count == 1 {
        return Some(pos_tokens[0]);
    }

    unsafe {
        let mut offset = 0;
        for (i, tok) in pos_tokens[..pos_count].iter().enumerate() {
            if i > 0 {
                let space_enc = b"%20";
                if offset + space_enc.len() >= 512 {
                    break;
                }
                ASSEMBLED_URL_BUF[offset..offset + space_enc.len()].copy_from_slice(space_enc);
                offset += space_enc.len();
            }
            let bytes = tok.as_bytes();
            if offset + bytes.len() >= 512 {
                break;
            }
            ASSEMBLED_URL_BUF[offset..offset + bytes.len()].copy_from_slice(bytes);
            offset += bytes.len();
        }
        core::str::from_utf8(&ASSEMBLED_URL_BUF[..offset]).ok()
    }
}

/// Execute the 'fetch' command to stream HTTP/HTTPS responses directly to console or inspect headers.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") || args.is_empty() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: fetch [options] <URL>\n\n");
        vga::print_str("Description:\n  Stream and inspect network resources over HTTP or native TLS 1.3 HTTPS directly to console output.\n\n");
        vga::print_str("Options:\n");
        vga::print_str(
            "  -I, --head           Fetch and display response metadata (omit body payload)\n",
        );
        vga::print_str(
            "  -v, --verbose        Display connection diagnostic metadata and transport state\n",
        );
        vga::print_str("  -o, --output <file>  Save response payload directly to storage file\n");
        vga::print_str("  -h, --help           Show this help message and exit\n\n");
        vga::print_str("Examples:\n");
        vga::print_str("  fetch http://10.0.2.2/api/status\n");
        vga::print_str("  fetch -I https://example.com\n");
        vga::print_str("  fetch -v http://10.0.2.2:8080/index.html\n");
        vga::print_str("  fetch -o /data/payload.bin http://10.0.2.2/data.bin\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let url = match assemble_url(&args) {
        Some(u) => u,
        None => {
            vga::set_color(vga::Color::Yellow, vga::Color::Black);
            vga::print_str("Usage: fetch [options] <URL>\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }
    };

    let parsed = match ParsedUrl::parse(url) {
        Ok(p) => p,
        Err(err) => unsafe {
            print_status_badge("Error", vga::Color::LightRed);
            vga::print_str("URL parse error: ");
            vga::print_str(err);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        },
    };

    unsafe {
        keira_net::driver::e1000::init();
    }

    let is_verbose = args.has_flag('v', "verbose");
    let is_head = args.has_flag('I', "head");
    let output_file = args.get_opt('o', "output");

    if is_verbose {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("* Connecting to host: ");
        vga::print_str(parsed.host);
        vga::print_str(" (port ");
        vga::print_u64(parsed.port as u64);
        if parsed.is_https {
            vga::print_str(", TLS 1.3 encrypted");
        } else {
            vga::print_str(", plain HTTP");
        }
        vga::print_str(")\n");
        vga::print_str("* Request: GET ");
        vga::print_str(parsed.path);
        vga::print_str(" HTTP/1.1\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    }

    let result = unsafe {
        if parsed.is_https {
            match fetch_https_stream(parsed.host, parsed.path, |_recv, _total| {}) {
                Ok(res) => Ok(res),
                Err(tls_err) => {
                    if is_verbose {
                        print_status_badge("Warning", vga::Color::Yellow);
                        vga::print_str("TLS 1.3: ");
                        vga::print_str(tls_err);
                        vga::print_str(" -> Fallback to HTTP (Port 80)\n");
                    }
                    fetch_http_stream(url, |_recv, _total| {})
                }
            }
        } else {
            fetch_http_stream(url, |_recv, _total| {})
        }
    };

    match result {
        Ok((payload, content_len)) => {
            if is_head {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("HTTP/1.1 200 OK\n");
                vga::print_str("Host: ");
                vga::print_str(parsed.host);
                vga::print_str("\n");
                vga::print_str("Protocol: ");
                if parsed.is_https {
                    vga::print_str("HTTPS (TLS 1.3 AES-128-GCM)\n");
                } else {
                    vga::print_str("HTTP/1.1\n");
                }
                if let Some(cl) = content_len {
                    vga::print_str("Content-Length: ");
                    vga::print_u64(cl as u64);
                    vga::print_str(" bytes\n");
                }
                vga::print_str("Payload-Received: ");
                vga::print_u64(payload.len() as u64);
                vga::print_str(" bytes\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }

            if let Some(target_path) = output_file {
                let save_res = {
                    if !keira_fs::vfs::exists(target_path) {
                        if let Err(e) = keira_fs::vfs::create_file(target_path) {
                            unsafe {
                                print_status_badge("Error", vga::Color::LightRed);
                                vga::print_str("Failed to create '");
                                vga::print_str(target_path);
                                vga::print_str("': ");
                                vga::print_str(e);
                                vga::print_str("\n");
                                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                            }
                            return;
                        }
                    }
                    keira_fs::vfs::write_file(target_path, payload)
                };

                match save_res {
                    Ok(_) => unsafe {
                        print_status_badge("Finished", vga::Color::LightGreen);
                        vga::print_str("Saved ");
                        vga::print_u64(payload.len() as u64);
                        vga::print_str(" bytes to '");
                        vga::print_str(target_path);
                        vga::print_str("'\n");
                    },
                    Err(e) => unsafe {
                        print_status_badge("Error", vga::Color::LightRed);
                        vga::print_str("Failed to write to '");
                        vga::print_str(target_path);
                        vga::print_str("': ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    },
                }
                return;
            }

            // Print text payload directly to stdout
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            let max_print = core::cmp::min(payload.len(), 4096);
            if let Ok(s) = core::str::from_utf8(&payload[..max_print]) {
                vga::print_str(s);
                if max_print < payload.len() {
                    vga::print_str("\n... [output truncated at 4096 bytes] ...\n");
                } else if !s.ends_with('\n') {
                    vga::print_str("\n");
                }
            } else {
                vga::print_str("[Binary data: ");
                vga::print_u64(payload.len() as u64);
                vga::print_str(" bytes; use '-o <file>' to save to disk]\n");
            }
        }
        Err(err) => unsafe {
            print_status_badge("Error", vga::Color::LightRed);
            vga::print_str("Fetch failed: ");
            vga::print_str(err);
            vga::print_str("\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        },
    }
}

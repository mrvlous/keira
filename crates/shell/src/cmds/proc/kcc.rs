// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe, static_mut_refs)]

//! Implementation of the 'kcc' shell command to compile C source code into Ring 3 ELF binaries.

use core::str::SplitWhitespace;
use keira_io::vga;

static mut RAW_BUF: [u8; 32768] = [0u8; 32768];
static mut STAGED_BUF: [u8; 32768] = [0u8; 32768];
static mut LIB_BUF: [u8; 8192] = [0u8; 8192];
static mut ELF_BUF: [u8; 32768] = [0u8; 32768];

/// Execute the 'kcc' C compiler command from the interactive shell.
pub fn run(parts: &mut SplitWhitespace) {
    let mut source_file: Option<&str> = None;
    let mut output_file: &str = "/apps/bin/app.elf";

    while let Some(arg) = parts.next() {
        match arg {
            "-h" | "--help" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Usage: kcc [options] <source.c>\n\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                vga::print_str("Description:\n  Compile C source code into a freestanding Ring 3 ELF executable binary.\n\n");
                vga::print_str("Options:\n");
                vga::print_str("  -o, --output <file>  Specify output binary path (default: /apps/bin/app.elf)\n");
                vga::print_str(
                    "  -v, --version        Display compiler version and target architecture\n",
                );
                vga::print_str("  -h, --help           Display this help manual\n\n");
                vga::print_str("Examples:\n");
                vga::print_str("  kcc /apps/src/calc.c -o /apps/bin/calc.elf\n");
                vga::print_str("  kcc /apps/src/bench.c\n");
                return;
            }
            "-v" | "--version" => {
                vga::set_color(vga::Color::White, vga::Color::Black);
                vga::print_str("Keira C Compiler (KCC) v0.36.0\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                #[cfg(target_arch = "x86_64")]
                vga::print_str("Target: x86_64-keira-none (Freestanding Ring 3 ELF)\n");
                #[cfg(target_arch = "x86")]
                vga::print_str("Target: i686-keira-none (Freestanding Ring 3 ELF)\n");
                return;
            }
            "-o" | "--output" => {
                if let Some(out_path) = parts.next() {
                    output_file = out_path;
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: -o option requires an output filename\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            }
            s => {
                if !s.starts_with('-') {
                    source_file = Some(s);
                } else {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("Error: unknown option: ");
                    vga::print_str(s);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            }
        }
    }

    let src = match source_file {
        Some(s) => s,
        None => {
            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("kcc: no input file specified. Type 'kcc --help' for usage.\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);
            return;
        }
    };

    // 1. Verify source file exists
    if !keira_fs::vfs::exists(src) {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("kcc: error: cannot find source file: ");
        vga::print_str(src);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    // 2. Read source code into static buffer
    let read_len = unsafe {
        match keira_fs::vfs::read_file(src, &mut RAW_BUF) {
            Ok(len) => len,
            Err(e) => {
                vga::set_color(vga::Color::LightRed, vga::Color::Black);
                vga::print_str("kcc: error reading source file: ");
                vga::print_str(e);
                vga::print_str("\n");
                vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                return;
            }
        }
    };

    let mut staged_len = 0;

    let src_str = unsafe { core::str::from_utf8(&RAW_BUF[..read_len]).unwrap_or("") };
    let libs = [
        ("<math.h>", "\"math.h\"", "/system/lib/math.c"),
        ("<string.h>", "\"string.h\"", "/system/lib/string.c"),
        ("<stdlib.h>", "\"stdlib.h\"", "/system/lib/stdlib.c"),
        ("<unistd.h>", "\"unistd.h\"", "/system/lib/unistd.c"),
        ("<assert.h>", "\"assert.h\"", "/system/lib/assert.c"),
        ("<dirent.h>", "\"dirent.h\"", "/system/lib/dirent.c"),
        ("<sys/stat.h>", "\"stat.h\"", "/system/lib/stat.c"),
        ("<signal.h>", "\"signal.h\"", "/system/lib/signal.c"),
        ("<time.h>", "\"time.h\"", "/system/lib/time.c"),
        ("<setjmp.h>", "\"setjmp.h\"", "/system/lib/setjmp.c"),
    ];

    for &(hdr1, hdr2, lib_path) in &libs {
        if src_str.contains(hdr1) || src_str.contains(hdr2) {
            unsafe {
                if let Ok(lib_len) = keira_fs::vfs::read_file(lib_path, &mut LIB_BUF) {
                    if staged_len + lib_len + 1 <= 32768 {
                        STAGED_BUF[staged_len..staged_len + lib_len]
                            .copy_from_slice(&LIB_BUF[..lib_len]);
                        staged_len += lib_len;
                        STAGED_BUF[staged_len] = b'\n';
                        staged_len += 1;
                    }
                }
            }
        }
    }

    unsafe {
        if staged_len + read_len <= 32768 {
            STAGED_BUF[staged_len..staged_len + read_len].copy_from_slice(&RAW_BUF[..read_len]);
            staged_len += read_len;
        }
    }

    // 3. Stage source code to /data/main.c for KCC compiler engine
    let stage_res = unsafe {
        let _ = keira_fs::fat::remove_entry("/data/main.c");
        let _ = keira_fs::fat::create_file("/data/main.c");
        keira_fs::fat::write_file_content("/data/main.c", &STAGED_BUF[..staged_len])
    };

    if let Err(e) = stage_res {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("kcc: error staging compilation buffer: ");
        vga::print_str(e);
        vga::print_str("\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Compiling: ");
    vga::print_str(src);
    vga::print_str(" -> ");
    vga::print_str(output_file);
    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);

    // 4. Delete old app.elf and run compiler in Ring 3
    unsafe {
        let _ = keira_fs::fat::remove_entry("/apps/bin/app.elf");
    }

    if !super::run::run_direct("/system/bin/kcc.elf") {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("kcc: error: compiler binary /system/bin/kcc.elf not found\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    // 5. If a custom output path was requested, copy from /apps/bin/app.elf to target
    if output_file != "/apps/bin/app.elf" {
        unsafe {
            match keira_fs::fat::read_file_content("/apps/bin/app.elf", &mut ELF_BUF) {
                Ok(elf_len) => {
                    let _ = keira_fs::fat::remove_entry(output_file);
                    let _ = keira_fs::fat::create_file(output_file);
                    if let Err(e) =
                        keira_fs::fat::write_file_content(output_file, &ELF_BUF[..elf_len])
                    {
                        vga::set_color(vga::Color::LightRed, vga::Color::Black);
                        vga::print_str("kcc: failed writing output binary: ");
                        vga::print_str(e);
                        vga::print_str("\n");
                        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                        return;
                    }
                }
                Err(e) => {
                    vga::set_color(vga::Color::LightRed, vga::Color::Black);
                    vga::print_str("kcc: compilation failed to generate binary: ");
                    vga::print_str(e);
                    vga::print_str("\n");
                    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
                    return;
                }
            }
        }
    } else if !keira_fs::vfs::exists("/apps/bin/app.elf") {
        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("kcc: compilation failed to generate binary\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("[OK] Executable ready at ");
    vga::print_str(output_file);
    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("Hint: Execute with 'run ");
    vga::print_str(output_file);
    vga::print_str("'\n");
}

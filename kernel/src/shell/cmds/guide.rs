#![allow(unused_variables, unused_unsafe)]
// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Keira Kernel: Shell Command 'guide'
//!
//! Implementation of the 'guide' shell command to list available commands and detail their usages.

use crate::io::vga;
use crate::shell::state::*;

pub fn run(parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let sub = parts.next();
        let bg = CURRENT_THEME.text_bg;
        match sub {
            Some("-h") | Some("--help") => {
                vga::print_str("Usage: guide [command]\n\n");
                vga::print_str("Description:\n  Interactive system command directory and documentation guide system.\n\n");
                vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
            }
            None => {
                vga::set_color(vga::Color::LightBlue, bg);
                vga::print_str("Keira Kernel System Guide\n");
                vga::print_str("Type 'guide <command>' to view detailed usage instructions.\n\n");

                vga::set_color(vga::Color::LightBlue, bg);
                vga::print_str("System & Hardware:\n");
                vga::set_color(vga::Color::White, bg);
                vga::print_str(
                    "  system    cpu       runtime   time      memory    devices   network\n",
                );
                vga::print_str("  download  env       framebuffer usb       https\n\n");

                vga::set_color(vga::Color::LightBlue, bg);
                vga::print_str("Storage & Filesystem:\n");
                vga::set_color(vga::Color::White, bg);
                vga::print_str("  drives    use       disk      ramdisk   list      go\n");
                vga::print_str("  view      create    folder    delete    edit      write\n");
                vga::print_str("  copy      move      initrd    search    sync      protect\n");
                vga::print_str("  fileinfo\n\n");

                vga::set_color(vga::Color::LightBlue, bg);
                vga::print_str("Process & Execution:\n");
                vga::set_color(vga::Color::White, bg);
                vga::print_str("  tasks     stop      wait      script    run\n\n");

                vga::set_color(vga::Color::LightBlue, bg);
                vga::print_str("User Account & Privileges:\n");
                vga::set_color(vga::Color::White, bg);
                vga::print_str("  please    login     user      hostname\n\n");

                vga::set_color(vga::Color::LightBlue, bg);
                vga::print_str("Utilities & Console:\n");
                vga::set_color(vga::Color::White, bg);
                vga::print_str("  guide     help      history   theme     say       play\n");
                vga::print_str("  hda       wipe      reset\n");
                vga::set_color(CURRENT_THEME.text_fg, bg);
            }
            Some("system") => {
                vga::print_str("Usage: system\nShow system hardware specifications, memory statistics, and uptime.\n");
            }
            Some("cpu") => {
                vga::print_str(
                    "Usage: cpu\nDisplay the CPU vendor signature string (e.g. AuthenticAMD).\n",
                );
            }
            Some("devices") => {
                vga::print_str("Usage: devices\nScan and list all detected PCI hardware devices and their vendor IDs.\n");
            }
            Some("network") => {
                vga::print_str("Usage: network [dhcp|resolve <domain>|ping <target_ip>]\nDisplay interface state, configure DHCP, resolve DNS, or send ICMP ping.\n");
            }
            Some("download") => {
                vga::print_str("Usage: download <URL> [target_file_path]\nFetch network payload over encrypted HTTPS (Native TLS 1.3 Engine) or plain HTTP and save to FAT16 disk storage.\n");
            }
            Some("runtime") => {
                vga::print_str(
                    "Usage: runtime\nShow the time elapsed since the system booted in ms.\n",
                );
            }
            Some("time") => {
                vga::print_str("Usage: time\nDisplay the current real-time clock (RTC) date and time in UTC.\n");
            }
            Some("memory") => {
                vga::print_str("Usage: memory\nDisplay kernel heap allocations and physical page frame statistics.\n");
            }
            Some("initrd") => {
                vga::print_str(
                    "Usage: initrd\nList all files preloaded in the read-only Initrd RAM disk.\n",
                );
            }
            Some("disk") => {
                vga::print_str("Usage: disk\nDisplay primary storage drive geometry and active filesystem details.\n");
            }
            Some("list") => {
                vga::print_str("Usage: list [path] [-a|-all]\nList the files and directories located in the specified or current directory.\nOptions:\n  -a, -all   Show hidden/system files and dot/dotdot entries.\n");
            }
            Some("view") => {
                vga::print_str("Usage: view <filename>\nRead and display the contents of a file from the active storage drive (falls back to initrd).\n");
            }
            Some("search") => {
                vga::print_str("Usage: search <pattern> [filename]\nSearch for lines matching <pattern> in [filename] or from the pipe input stream.\n");
            }
            Some("create") => {
                vga::print_str(
                    "Usage: create <filename>\nCreate an empty file in the active directory.\n",
                );
            }
            Some("folder") => {
                vga::print_str("Usage: folder <foldername>\nCreate a new subdirectory in the active directory.\n");
            }
            Some("delete") => {
                vga::print_str("Usage: delete <name>\nDelete a file or empty folder from the active directory.\n");
            }
            Some("edit") => {
                vga::print_str("Usage: edit <filename>\nOpen the text editor to create or edit a file on the active storage drive.\n");
            }
            Some("go") => {
                vga::print_str("Usage: go <path>\nChange the current working directory on the active drive (supports '.' and '..').\n");
            }
            Some("script") => {
                vga::print_str("Usage: script <filename.sh>\nRead and execute commands from specified file line-by-line.\n");
            }
            Some("tasks") => {
                vga::print_str("Usage: tasks\nList all running processes, their state, and IDs in the scheduler.\n");
            }
            Some("stop") => {
                vga::print_str(
                    "Usage: stop <PID>\nTerminate a running process by its Process ID (PID).\n",
                );
            }
            Some("wait") => {
                vga::print_str("Usage: wait <ms>\nSuspend the shell execution for a specified number of milliseconds.\n");
            }
            Some("guide") => {
                vga::print_str("Usage: guide [command]\nShow the list of commands, or details about a specific command.\n");
            }
            Some("say") => {
                vga::print_str("Usage: say <message>\nEcho back the arguments typed by the user to the screen.\n");
            }
            Some("play") => {
                vga::print_str("Usage: play <mario|nokia|starwars|beep>\nPlay retro tunes or a simple beep on the PC speaker.\n");
            }
            Some("wipe") => {
                vga::print_str("Usage: wipe\nClear the VGA screen and reset the cursor to the top-left position.\n");
            }
            Some("reset") => {
                vga::print_str(
                    "Usage: reset\nReboot the virtual machine using a keyboard controller reset.\n",
                );
            }
            Some("drives") => {
                vga::print_str("Usage: drives\nList all registered block storage devices, their sizes and mount status.\n");
            }
            Some("use") => {
                vga::print_str("Usage: use <device_name>\nMount a block storage device and dynamically initialize its FAT16 filesystem.\n");
            }
            Some("ramdisk") => {
                vga::print_str("Usage: ramdisk create <size_kb>\nDynamically allocate a RAM Disk in memory, auto-format as FAT16, and register it.\n");
            }
            Some("please") => {
                vga::print_str("Usage: please <command>\nExecute a command with temporary administrative privileges (asks for password).\n");
            }
            Some("login") => {
                vga::print_str("Usage: login <username>\nSwitch active user context. Admin logs in without password. Other users require password.\n");
            }
            Some("user") => {
                vga::print_str("Usage: user <create|delete|list|password|info>\nManage user accounts. Accounts stored in /system/etc/passwd on FAT16 disk.\n");
            }
            Some("hostname") => {
                vga::print_str("Usage: hostname [new_name]\nView or set the system hostname. Persisted to /system/etc/hostname on FAT16 disk.\n");
            }
            Some("run") => {
                vga::print_str("Usage: run <program.elf>\nLoad and execute a freestanding user mode ELF program in Ring 3.\n");
            }
            Some("write") => {
                vga::print_str("Usage: write [-a|--append|>>] <filename> <text>\nWrite or append text content to a file on the active FAT16 storage drive.\n");
            }
            Some("copy") => {
                vga::print_str("Usage: copy <src_file> <dest_file>\nCopy a file from the source path to the destination path.\n");
            }
            Some("help") => {
                vga::print_str("Usage: help [command]\nFriendly redirect to the guide system (same as guide).\n");
            }
            Some("history") => {
                vga::print_str(
                    "Usage: history\nPrint the ring buffer of recently entered shell commands.\n",
                );
            }
            Some("move") => {
                vga::print_str("Usage: move <src_file> <dest_file>\nMove or rename a file from the source path to the destination path.\n");
            }
            Some("theme") => {
                vga::print_str("Usage: theme [retro|matrix|arch|classic|dracula]\nChange the active shell background, foreground, and accent colors dynamically.\n");
            }
            Some("hda") => {
                vga::print_str("Usage: hda <play [freq]|stop|status>\nPlay sound waveforms using the Intel High Definition Audio (HDA) controller.\n");
            }
            Some("env") => {
                vga::print_str("Usage: env [key] [value]\nView or modify environment variables ($USER, $HOME, $PATH, $SHELL).\n");
            }
            Some("sync") => {
                vga::print_str(
                    "Usage: sync\nFlush dirty filesystem block cache sectors to storage device.\n",
                );
            }
            Some("protect") => {
                vga::print_str("Usage: protect <file_path> <readonly|readwrite>\nToggle read-only or read-write attribute protection on FAT16 file entry.\n");
            }
            Some("fileinfo") => {
                vga::print_str("Usage: fileinfo <file_path>\nInspect detailed FAT16 file metadata, cluster index, size, and protection flags.\n");
            }
            Some("framebuffer") => {
                vga::print_str("Usage: framebuffer <info|demo|test>\nQuery VBE 1024x768 32-bpp graphics info, test linear framebuffer, or launch desktop demo.\n");
            }
            Some("usb") => {
                vga::print_str("Usage: usb <info|scan|devices>\nEnumerate PCI USB host controllers (xHCI/EHCI) and list connected USB devices.\n");
            }
            Some("https") => {
                vga::print_str("Usage: https <url|info|sha256>\nPerform encrypted HTTPS GET request over Native TLS 1.3 Engine (AES-128-GCM, X25519).\n");
            }
            Some(other) => {
                vga::set_color(vga::Color::LightRed, bg);
                vga::print_str("Error: Unknown command '");
                vga::print_str(other);
                vga::print_str("'. Type 'guide' to see all commands.\n");
                vga::set_color(CURRENT_THEME.text_fg, bg);
            }
        }
    }
}

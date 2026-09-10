// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![allow(unused_variables, unused_unsafe)]

//! Implementation of the 'help' shell command.

use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    unsafe {
        let bg = vga::Color::Black;
        vga::set_color(vga::Color::White, bg);
        vga::print_str("Keira Kernel Built-in Commands\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str(
            "Use '<command> --help' to view options and detailed usage for any command.\n\n",
        );

        vga::set_color(vga::Color::White, bg);
        vga::print_str("System & Hardware:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  system    cpu       smp       memory    devices   time      runtime\n");
        vga::print_str("  power     reset     env       hostname  user      login     syslog\n");
        vga::print_str(
            "  drivers   lkm       watchpoint unwind    perf      kvm       framebuffer\n\n",
        );

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Storage & Filesystem:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  drives    use       disk      ramdisk   initrd    sync      ext4\n");
        vga::print_str("  list      go        folder    create    delete    copy      move\n");
        vga::print_str(
            "  view      write     edit      fileinfo  protect   swap      lvm       raid\n\n",
        );

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Process & Services:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  tasks     run       stop      kill      jobs      fg        bg\n");
        vga::print_str("  cgroups   futex     eventfd   epoll     mqueue    timer     service\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Network & Security:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  network   download  https     firewall  iptables  ipcs      ipcrm\n");
        vga::print_str("  bpf       seccomp   mac       tpm\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("General Utilities:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  help      history   wipe      search\n");
    }
}

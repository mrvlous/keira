// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Implementation of the 'help' shell command.

use keira_io::vga;

pub fn run(_parts: &mut core::str::SplitWhitespace) {
    {
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
        vga::print_str(
            "  power     reset     hostname  syslog    drivers   lkm       watchpoint\n",
        );
        vga::print_str("  unwind    perf      kvm       framebuffer usb\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Storage & Filesystem:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  drives    use       disk      ramdisk   initrd    sync      ext4\n");
        vga::print_str("  list      go        folder    create    delete    copy      move\n");
        vga::print_str("  view      write     edit      fileinfo  swap      lvm       raid\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Process, Scheduling & IPC:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  tasks     run       stop      kill      jobs      fg        bg\n");
        vga::print_str("  cgroups   futex     eventfd   epoll     mqueue    timer     kcc\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("Network & Security:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  network   download  fetch     https     firewall  bpf       seccomp\n");
        vga::print_str("  mac       tpm\n\n");

        vga::set_color(vga::Color::White, bg);
        vga::print_str("General Utilities:\n");
        vga::set_color(vga::Color::LightGrey, bg);
        vga::print_str("  help      history   wipe      search\n");
    }
}

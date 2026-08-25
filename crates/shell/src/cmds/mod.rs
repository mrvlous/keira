// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//!
//! Exposes all modular command handlers implemented in the `cmds/` directory.

pub mod bg;
pub mod bpf;
pub mod cgroups;
pub mod copy;
pub mod cpu;
pub mod create;
pub mod delete;
pub mod devices;
pub mod disk;
pub mod download;
pub mod drivers;
pub mod drives;
pub mod edit;
pub mod env;
pub mod epoll;
pub mod eventfd;
pub mod ext4;
pub mod fg;
pub mod fileinfo;
pub mod firewall;
pub mod folder;
pub mod framebuffer;
pub mod futex;
pub mod go;
pub mod guide;
pub mod help;
pub mod history;
pub mod hostname;
pub mod https;
pub mod initrd;
pub mod ipcrm;
pub mod ipcs;
pub mod iptables;
pub mod jobs;
pub mod kill;
pub mod kvm;
pub mod list;
pub mod lkm;
pub mod login;
pub mod lvm;
pub mod mac;
pub mod memory;
pub mod r#move;
pub mod mqueue;
pub mod network;
pub mod nvme;
pub mod perf;
pub mod power;
pub mod protect;
pub mod raid;
pub mod ramdisk;
pub mod reset;
pub mod run;
pub mod runtime;
pub mod script;
pub mod search;
pub mod seccomp;
pub mod service;
pub mod stop;
pub mod swap;
pub mod sync;
pub mod syslog;
pub mod system;
pub mod tasks;
pub mod time;
pub mod timer;
pub mod tpm;
pub mod unwind;
pub mod usb;
pub mod r#use;
pub mod user;
pub mod view;
pub mod wait;
pub mod wipe;
pub mod write;

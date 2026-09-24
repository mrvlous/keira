// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for executor environment expansion and redirection parsing.

use super::dispatch::hardware::{count_pci_devices, print_2digit};
use super::env::expand::expand_env_vars;
use super::pipeline::redirect::{parse_input_redirection, parse_output_redirection};

#[test]
fn test_expand_env_vars_literal() {
    let mut buf = [0u8; 64];
    let len = expand_env_vars("echo hello world", &mut buf);
    assert_eq!(&buf[..len], b"echo hello world");
}

#[test]
fn test_parse_input_redirection() {
    let (cmd, file) = parse_input_redirection("view < /users/admin/notes.txt");
    assert_eq!(cmd, "view");
    assert_eq!(file, Some("/users/admin/notes.txt"));

    let (cmd2, file2) = parse_input_redirection("view");
    assert_eq!(cmd2, "view");
    assert_eq!(file2, None);
}

#[test]
fn test_parse_output_redirection() {
    let (cmd, file, append) = parse_output_redirection("help > /data/help.txt");
    assert_eq!(cmd, "help");
    assert_eq!(file, Some("/data/help.txt"));
    assert!(!append);

    let (cmd2, file2, append2) = parse_output_redirection("help >> /data/log.txt");
    assert_eq!(cmd2, "help");
    assert_eq!(file2, Some("/data/log.txt"));
    assert!(append2);
}

#[test]
fn test_count_pci_devices_safe() {
    let _ = count_pci_devices();
    print_2digit(5);
    print_2digit(42);
}

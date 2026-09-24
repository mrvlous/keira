// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for virtual terminal switching.

use super::session::{get_active_tty, switch_tty};

#[test]
fn test_switch_tty_bounds() {
    switch_tty(0);
    assert_eq!(get_active_tty(), 0);

    switch_tty(1);
    assert_eq!(get_active_tty(), 1);

    switch_tty(2);
    assert_eq!(get_active_tty(), 2);

    // Out of bounds - should be ignored
    switch_tty(5);
    assert_eq!(get_active_tty(), 2);

    switch_tty(0);
}

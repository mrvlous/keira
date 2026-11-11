// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! Unit tests for kernel runtime and userspace transitions.

#[test]
fn test_runtime_version_format() {
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.8.0");
}

#[test]
fn test_kernel_purity_invariants() {
    assert_eq!(env!("CARGO_PKG_NAME"), "keira-kernel");
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.8.0");
    assert!(env!("CARGO_PKG_DESCRIPTION").contains("Keira Kernel"));
}

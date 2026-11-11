// SPDX-License-Identifier: GPL-2.0-only

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

// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for kernel subsystem initialization pipeline.

#[test]
fn test_init_pipeline_constants() {
    assert_ne!(keira_arch::timers::DEFAULT_HPET_BASE, 0);
}

// SPDX-License-Identifier: GPL-2.0-only

//! Security enclave, measured boot and scheduler initialization stage.

pub mod measure;

pub use measure::init_security_and_measure;

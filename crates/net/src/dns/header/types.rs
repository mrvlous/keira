// SPDX-License-Identifier: GPL-2.0-only

//! DNS protocol wire format headers and flags.

/// Standard 12-byte DNS packet header.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DnsHeader {
    pub transaction_id: u16,
    pub flags: u16,
    pub questions: u16,
    pub answer_rrs: u16,
    pub authority_rrs: u16,
    pub additional_rrs: u16,
}

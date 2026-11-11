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

//! TCP state machine states and ephemeral client port allocation.

use core::sync::atomic::{AtomicU16, Ordering};

static NEXT_SRC_PORT: AtomicU16 = AtomicU16::new(49152);

/// Allocate next unique source port in ephemeral range 49152..65000.
pub fn get_next_src_port() -> u16 {
    let port = NEXT_SRC_PORT.fetch_add(1, Ordering::Relaxed);
    if !(49152..=65000).contains(&port) {
        NEXT_SRC_PORT.store(49152, Ordering::Relaxed);
        49152
    } else {
        port
    }
}

/// Transmission Control Protocol connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    SynSent,
    Established,
    FinWait,
}

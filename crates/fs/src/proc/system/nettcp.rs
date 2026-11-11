// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/net/tcp` node reporting active kernel TCP sockets.

use crate::proc::system::hooks::NET_TCP_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted active TCP socket table into destination buffer.
pub fn read_net_tcp(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_TCP_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode"
    );

    Ok(writer.len())
}

// SPDX-License-Identifier: GPL-2.0-only

//! Dynamic `/proc/net/route` node reporting in-kernel IPv4 routing table.

use crate::proc::system::hooks::NET_ROUTE_HOOK;
use crate::proc::writer::BufWriter;
use core::fmt::Write;

/// Reads formatted kernel IPv4 routing table into destination buffer.
pub fn read_net_route(buf: &mut [u8]) -> Result<usize, &'static str> {
    if let Some(hook) = unsafe { NET_ROUTE_HOOK } {
        return Ok(hook(buf));
    }

    let mut writer = BufWriter::new(buf);

    let _ = writeln!(
        writer,
        "Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT"
    );

    Ok(writer.len())
}

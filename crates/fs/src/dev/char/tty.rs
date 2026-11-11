// SPDX-License-Identifier: GPL-2.0-only

//! Character device forwarding for `/dev/tty` and `/dev/ptmx`.

use keira_io::vga;

/// Reads characters from active TTY line discipline or PS/2 keyboard queue.
///
/// # Safety
///
/// Accesses hardware keyboard buffer queue.
pub unsafe fn read(buf: &mut [u8]) -> Result<usize, &'static str> {
    let n = keira_io::tty::read_tty(buf);
    Ok(n)
}

/// Writes byte slice to VGA console display.
pub fn write(buf: &[u8]) -> Result<usize, &'static str> {
    if let Ok(s) = core::str::from_utf8(buf) {
        vga::print_str(s);
    }
    Ok(buf.len())
}

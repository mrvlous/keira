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

//! 16550A Universal Asynchronous Receiver-Transmitter (UART) driver.

pub mod driver {
    use core::sync::atomic::{AtomicBool, Ordering};
    use keira_arch::cpu::{inb, outb};

    pub const COM1: u16 = 0x3F8;

    static SERIAL_INITIALIZED: AtomicBool = AtomicBool::new(false);

    pub fn init() {
        unsafe {
            // 1. Disable all interrupts
            outb(COM1 + 1, 0x00);

            // 2. Enable DLAB (set baud rate divisor latch)
            outb(COM1 + 3, 0x80);

            // 3. Set divisor to 1 (low byte 0x01, high byte 0x00) -> 115200 baud
            outb(COM1, 0x01);
            outb(COM1 + 1, 0x00);

            // 4. 8 bits, no parity, 1 stop bit (8N1)
            outb(COM1 + 3, 0x03);

            // 5. Enable FIFO, clear TX/RX FIFOs, set 14-byte threshold
            outb(COM1 + 2, 0xC7);

            // 6. Set RTS/DSR, Auxiliary Output 2 (IRQs enabled)
            outb(COM1 + 4, 0x0B);

            SERIAL_INITIALIZED.store(true, Ordering::SeqCst);
        }
    }

    #[inline(always)]
    pub fn is_transmit_empty() -> bool {
        #[cfg(not(target_os = "none"))]
        {
            true
        }
        #[cfg(target_os = "none")]
        unsafe {
            (inb(COM1 + 5) & 0x20) != 0
        }
    }

    #[inline(always)]
    pub fn has_byte() -> bool {
        if !SERIAL_INITIALIZED.load(Ordering::Relaxed) {
            init();
        }
        // Port I/O requires unsafe block, caller guarantees valid port access.
        unsafe { (inb(COM1 + 5) & 0x01) != 0 }
    }

    #[inline(always)]
    pub fn read_byte() -> u8 {
        if has_byte() {
            unsafe { inb(COM1) }
        } else {
            0
        }
    }

    pub fn putchar(c: u8) {
        if !SERIAL_INITIALIZED.load(Ordering::Relaxed) {
            init();
        }
        while !is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            outb(COM1, c);
        }
    }

    #[no_mangle]
    pub extern "C" fn serial_putchar(c: u8) {
        putchar(c);
    }
}

pub mod writer {
    use super::driver::putchar;
    use core::fmt;

    #[derive(Debug, Default, Clone, Copy)]
    pub struct SerialWriter;

    impl fmt::Write for SerialWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            print_str(s);
            Ok(())
        }
    }

    pub fn print(bytes: &[u8]) {
        for &b in bytes {
            putchar(b);
        }
    }

    pub fn print_str(s: &str) {
        print(s.as_bytes());
    }

    pub fn print_u64(val: u64) {
        if val == 0 {
            putchar(b'0');
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 20;
        let mut temp = val;
        while temp > 0 {
            i -= 1;
            buf[i] = b'0' + (temp % 10) as u8;
            temp /= 10;
        }
        print(&buf[i..]);
    }

    pub fn print_hex(val: u64) {
        print_str("0x");
        let chars = b"0123456789ABCDEF";
        let mut buf = [0u8; 16];
        let mut i = 16;
        let mut temp = val;
        if temp == 0 {
            putchar(b'0');
            return;
        }
        while temp > 0 {
            i -= 1;
            buf[i] = chars[(temp & 0x0F) as usize];
            temp >>= 4;
        }
        print(&buf[i..]);
    }

    pub fn format_u64_buffer(val: u64, buf: &mut [u8; 20]) -> usize {
        if val == 0 {
            buf[19] = b'0';
            return 19;
        }
        let mut i = 20;
        let mut temp = val;
        while temp > 0 {
            i -= 1;
            buf[i] = b'0' + (temp % 10) as u8;
            temp /= 10;
        }
        i
    }

    pub fn format_hex_buffer(val: u64, buf: &mut [u8; 16]) -> usize {
        let chars = b"0123456789ABCDEF";
        if val == 0 {
            buf[15] = b'0';
            return 15;
        }
        let mut i = 16;
        let mut temp = val;
        while temp > 0 {
            i -= 1;
            buf[i] = chars[(temp & 0x0F) as usize];
            temp >>= 4;
        }
        i
    }
}

pub use self::driver::*;
pub use self::writer::*;

#[cfg(test)]
mod tests {
    use super::writer::{format_hex_buffer, format_u64_buffer};

    #[test]
    fn test_u64_formatting() {
        let mut buf = [0u8; 20];

        let start = format_u64_buffer(0, &mut buf);
        assert_eq!(&buf[start..], b"0");

        let start = format_u64_buffer(123456789, &mut buf);
        assert_eq!(&buf[start..], b"123456789");

        let start = format_u64_buffer(u64::MAX, &mut buf);
        assert_eq!(&buf[start..], b"18446744073709551615");
    }

    #[test]
    fn test_hex_formatting() {
        let mut buf = [0u8; 16];

        let start = format_hex_buffer(0, &mut buf);
        assert_eq!(&buf[start..], b"0");

        let start = format_hex_buffer(0x1234_ABCD, &mut buf);
        assert_eq!(&buf[start..], b"1234ABCD");

        let start = format_hex_buffer(0xDEAD_BEEF_CAFE_BABE, &mut buf);
        assert_eq!(&buf[start..], b"DEADBEEFCAFEBABE");
    }
}

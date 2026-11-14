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

//! Traditional kernel pipe FIFO buffer and read/write operations.

pub mod buffer {
    pub const MAX_PIPES: usize = 32;

    pub const PIPE_BUFFER_CAPACITY: usize = 4096;

    pub const PIPE_BUFFER_SIZE: usize = PIPE_BUFFER_CAPACITY;

    pub const O_NONBLOCK: u32 = 0x800;

    pub const O_CLOEXEC: u32 = 0x80000;

    pub struct PipeBuffer {
        pub data: [u8; 1024],
        pub read_pos: usize,
        pub write_pos: usize,
        pub count: usize,
    }

    pub static mut SYSTEM_PIPE: PipeBuffer = PipeBuffer {
        data: [0; 1024],
        read_pos: 0,
        write_pos: 0,
        count: 0,
    };

    #[derive(Copy, Clone, Debug)]
    pub struct PipeChannel {
        pub id: u32,
        pub in_use: bool,
        pub data: [u8; PIPE_BUFFER_CAPACITY],
        pub head: usize,
        pub tail: usize,
        pub count: usize,
        pub readers_count: u32,
        pub writers_count: u32,
        pub flags: u32,
    }

    impl PipeChannel {
        /// Const constructor for an unallocated pipe channel slot.
        pub const fn empty() -> Self {
            Self {
                id: 0,
                in_use: false,
                data: [0u8; PIPE_BUFFER_CAPACITY],
                head: 0,
                tail: 0,
                count: 0,
                readers_count: 0,
                writers_count: 0,
                flags: 0,
            }
        }
    }

    pub static mut PIPE_TABLE: [PipeChannel; MAX_PIPES] = [PipeChannel::empty(); MAX_PIPES];

    pub static mut NEXT_PIPE_ID: u32 = 1;
}

pub mod ops {
    use crate::pipe::fifo::buffer::{
        PipeChannel, MAX_PIPES, NEXT_PIPE_ID, O_NONBLOCK, PIPE_BUFFER_CAPACITY, PIPE_TABLE,
        SYSTEM_PIPE,
    };

    pub unsafe fn write_pipe(buf: &[u8]) -> usize {
        let pipe_ptr = &raw mut SYSTEM_PIPE;
        let pipe = &mut *pipe_ptr;
        let mut written = 0usize;

        for &b in buf {
            if pipe.count >= 1024 {
                break;
            }
            pipe.data[pipe.write_pos] = b;
            pipe.write_pos = (pipe.write_pos + 1) % 1024;
            pipe.count += 1;
            written += 1;
        }

        written
    }

    pub unsafe fn read_pipe(buf: &mut [u8]) -> usize {
        let pipe_ptr = &raw mut SYSTEM_PIPE;
        let pipe = &mut *pipe_ptr;
        let mut read_bytes = 0usize;

        for slot in buf.iter_mut() {
            if pipe.count == 0 {
                break;
            }
            *slot = pipe.data[pipe.read_pos];
            pipe.read_pos = (pipe.read_pos + 1) % 1024;
            pipe.count -= 1;
            read_bytes += 1;
        }

        read_bytes
    }

    pub unsafe fn create_pipe() -> Result<(usize, usize), &'static str> {
        let pipe_ptr = &raw mut SYSTEM_PIPE;
        let pipe = &mut *pipe_ptr;
        pipe.read_pos = 0;
        pipe.write_pos = 0;
        pipe.count = 0;
        Ok((3, 4))
    }

    pub unsafe fn create_pipe_channel(flags: u32) -> Result<u32, &'static str> {
        for slot in PIPE_TABLE.iter_mut() {
            if !slot.in_use {
                let id = NEXT_PIPE_ID;
                NEXT_PIPE_ID = NEXT_PIPE_ID.wrapping_add(1);
                if NEXT_PIPE_ID == 0 {
                    NEXT_PIPE_ID = 1;
                }

                *slot = PipeChannel {
                    id,
                    in_use: true,
                    data: [0u8; PIPE_BUFFER_CAPACITY],
                    head: 0,
                    tail: 0,
                    count: 0,
                    readers_count: 1,
                    writers_count: 1,
                    flags,
                };
                return Ok(id);
            }
        }
        Err("Pipe table capacity exhausted")
    }

    pub unsafe fn read_pipe_channel(pipe_id: u32, buf: &mut [u8]) -> Result<usize, i64> {
        for slot in PIPE_TABLE.iter_mut() {
            if slot.in_use && slot.id == pipe_id {
                if slot.count > 0 {
                    let to_read = buf.len().min(slot.count);
                    for out in buf.iter_mut().take(to_read) {
                        *out = slot.data[slot.head];
                        slot.head = (slot.head + 1) % PIPE_BUFFER_CAPACITY;
                        slot.count -= 1;
                    }
                    return Ok(to_read);
                }

                // Buffer is empty: check if writer endpoints remain open
                if slot.writers_count == 0 {
                    return Ok(0); // POSIX EOF
                }

                if (slot.flags & O_NONBLOCK) != 0 {
                    return Err(-11); // -EAGAIN
                }

                return Ok(0);
            }
        }
        Err(-9) // -EBADF
    }

    pub unsafe fn write_pipe_channel(pipe_id: u32, buf: &[u8]) -> Result<usize, i64> {
        for slot in PIPE_TABLE.iter_mut() {
            if slot.in_use && slot.id == pipe_id {
                // Broken pipe condition: no active reader endpoints
                if slot.readers_count == 0 {
                    return Err(-32); // -EPIPE
                }

                let available = PIPE_BUFFER_CAPACITY - slot.count;
                if available == 0 {
                    if (slot.flags & O_NONBLOCK) != 0 {
                        return Err(-11); // -EAGAIN
                    }
                    return Ok(0);
                }

                let to_write = buf.len().min(available);
                for &byte in buf.iter().take(to_write) {
                    slot.data[slot.tail] = byte;
                    slot.tail = (slot.tail + 1) % PIPE_BUFFER_CAPACITY;
                    slot.count += 1;
                }
                return Ok(to_write);
            }
        }
        Err(-9) // -EBADF
    }

    pub unsafe fn close_pipe_read(pipe_id: u32) {
        for slot in PIPE_TABLE.iter_mut() {
            if slot.in_use && slot.id == pipe_id {
                if slot.readers_count > 0 {
                    slot.readers_count -= 1;
                }
                if slot.readers_count == 0 && slot.writers_count == 0 {
                    *slot = PipeChannel::empty();
                }
                return;
            }
        }
    }

    pub unsafe fn close_pipe_write(pipe_id: u32) {
        for slot in PIPE_TABLE.iter_mut() {
            if slot.in_use && slot.id == pipe_id {
                if slot.writers_count > 0 {
                    slot.writers_count -= 1;
                }
                if slot.readers_count == 0 && slot.writers_count == 0 {
                    *slot = PipeChannel::empty();
                }
                return;
            }
        }
    }

    pub unsafe fn dup_pipe_endpoint(pipe_id: u32, is_write: bool) {
        for slot in PIPE_TABLE.iter_mut() {
            if slot.in_use && slot.id == pipe_id {
                if is_write {
                    slot.writers_count = slot.writers_count.saturating_add(1);
                } else {
                    slot.readers_count = slot.readers_count.saturating_add(1);
                }
                return;
            }
        }
    }

    pub unsafe fn get_pipe_stats() -> (usize, usize) {
        let mut active = 0usize;
        let mut buffered = 0usize;
        for slot in PIPE_TABLE.iter() {
            if slot.in_use {
                active += 1;
                buffered += slot.count;
            }
        }
        (active, buffered)
    }

    pub unsafe fn get_pipe_table() -> &'static [PipeChannel; MAX_PIPES] {
        &*core::ptr::addr_of!(PIPE_TABLE)
    }

    pub unsafe fn pipe_table_reset() {
        for slot in PIPE_TABLE.iter_mut() {
            *slot = PipeChannel::empty();
        }
        NEXT_PIPE_ID = 1;
    }

    pub unsafe fn query_pipe_readiness(pipe_id: u32, is_write: bool) -> (bool, bool, bool, bool) {
        for slot in PIPE_TABLE.iter() {
            if slot.in_use && slot.id == pipe_id {
                if is_write {
                    let err = slot.readers_count == 0;
                    let writable = !err && slot.count < PIPE_BUFFER_CAPACITY;
                    return (false, writable, false, err);
                } else {
                    let hangup = slot.writers_count == 0;
                    let readable = slot.count > 0 || hangup;
                    return (readable, false, hangup, false);
                }
            }
        }
        (false, false, false, true)
    }
}

pub use self::buffer::*;
pub use self::ops::*;

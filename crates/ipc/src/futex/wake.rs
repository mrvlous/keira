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

//! Futex wake and requeue engine.

pub mod requeue {
    use crate::futex::hash::table::{FUTEX_TABLE, TOTAL_FUTEX_REQUEUES, TOTAL_FUTEX_WAKES};
    
    pub unsafe fn futex_wake(uaddr: usize, count: u32, bitset: u32) -> Result<u32, &'static str> {
        let mut woken = 0u32;
        let mask = if bitset == 0 { 0xFFFFFFFF } else { bitset };
    
        for slot in FUTEX_TABLE.iter_mut() {
            if slot.in_use && slot.uaddr == uaddr && (slot.bitset & mask) != 0 {
                slot.in_use = false;
                woken += 1;
                TOTAL_FUTEX_WAKES += 1;
                if woken >= count {
                    break;
                }
            }
        }
        Ok(woken)
    }
    
    pub unsafe fn futex_requeue(uaddr: usize, uaddr2: usize, count: u32) -> Result<u32, &'static str> {
        let mut requeued = 0u32;
    
        for slot in FUTEX_TABLE.iter_mut() {
            if slot.in_use && slot.uaddr == uaddr {
                slot.uaddr = uaddr2;
                requeued += 1;
                TOTAL_FUTEX_REQUEUES += 1;
                if requeued >= count {
                    break;
                }
            }
        }
        Ok(requeued)
    }
}

pub use self::requeue::*;

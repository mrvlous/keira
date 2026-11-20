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

//! In-kernel dynamic relocation computation and patching engine.
//!
//! Applies architectural relocations for 64-bit (`x86_64`) and 32-bit (`i686`)
//! relocatable kernel object (`.ko`) files during in-memory module linking.

use crate::error::KernelError;

/// x86_64 relocation types defined by System V AMD64 ABI.
pub const R_X86_64_NONE: u32 = 0;
pub const R_X86_64_64: u32 = 1;
pub const R_X86_64_PC32: u32 = 2;
pub const R_X86_64_32: u32 = 10;
pub const R_X86_64_32S: u32 = 11;

/// i386 relocation types defined by System V i386 ABI.
pub const R_386_NONE: u32 = 0;
pub const R_386_32: u32 = 1;
pub const R_386_PC32: u32 = 2;

/// Returns the textual representation of an architectural relocation type.
pub fn relocation_type_name(rel_type: u32, is_64bit: bool) -> &'static str {
    if is_64bit {
        match rel_type {
            R_X86_64_NONE => "R_X86_64_NONE",
            R_X86_64_64 => "R_X86_64_64",
            R_X86_64_PC32 => "R_X86_64_PC32",
            R_X86_64_32 => "R_X86_64_32",
            R_X86_64_32S => "R_X86_64_32S",
            _ => "R_X86_64_UNKNOWN",
        }
    } else {
        match rel_type {
            R_386_NONE => "R_386_NONE",
            R_386_32 => "R_386_32",
            R_386_PC32 => "R_386_PC32",
            _ => "R_386_UNKNOWN",
        }
    }
}

/// Computes and applies a single relocation directly to the module memory buffer.
///
/// Parameters:
/// - `target_buf`: Mutable slice of the section memory to patch.
/// - `offset`: Offset within `target_buf` where the relocated value must be written.
/// - `rel_type`: Architectural relocation code.
/// - `is_64bit`: Indicates whether target binary is 64-bit or 32-bit.
/// - `s`: Symbol value (absolute virtual address of referenced symbol).
/// - `a`: Relocation addend (explicit from `RELA` or implicit from target memory).
/// - `p`: Virtual memory address of the place being relocated (target memory address).
pub fn apply_relocation(
    target_buf: &mut [u8],
    offset: usize,
    rel_type: u32,
    is_64bit: bool,
    s: u64,
    a: i64,
    p: u64,
) -> Result<(), KernelError> {
    if is_64bit {
        match rel_type {
            R_X86_64_NONE => Ok(()),
            R_X86_64_64 => {
                // Direct 64-bit address: S + A
                if offset + 8 > target_buf.len() {
                    return Err(KernelError::Fault);
                }
                let val = (s as i64).wrapping_add(a) as u64;
                target_buf[offset..offset + 8].copy_from_slice(&val.to_le_bytes());
                Ok(())
            }
            R_X86_64_PC32 => {
                // PC-relative 32-bit signed displacement: S + A - P
                if offset + 4 > target_buf.len() {
                    return Err(KernelError::Fault);
                }
                let disp = (s as i64).wrapping_add(a).wrapping_sub(p as i64);
                let val = disp as i32;
                target_buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
                Ok(())
            }
            R_X86_64_32 => {
                // Direct 32-bit unsigned: S + A
                if offset + 4 > target_buf.len() {
                    return Err(KernelError::Fault);
                }
                let val = ((s as i64).wrapping_add(a) as u32).to_le_bytes();
                target_buf[offset..offset + 4].copy_from_slice(&val);
                Ok(())
            }
            R_X86_64_32S => {
                // Direct 32-bit signed: S + A
                if offset + 4 > target_buf.len() {
                    return Err(KernelError::Fault);
                }
                let val = ((s as i64).wrapping_add(a) as i32).to_le_bytes();
                target_buf[offset..offset + 4].copy_from_slice(&val);
                Ok(())
            }
            _ => Err(KernelError::NotSupported),
        }
    } else {
        match rel_type {
            R_386_NONE => Ok(()),
            R_386_32 => {
                // Direct 32-bit address: S + A
                if offset + 4 > target_buf.len() {
                    return Err(KernelError::Fault);
                }
                let implicit_addend = if a == 0 {
                    let mut b = [0u8; 4];
                    b.copy_from_slice(&target_buf[offset..offset + 4]);
                    u32::from_le_bytes(b) as i64
                } else {
                    a
                };
                let val = ((s as i64).wrapping_add(implicit_addend) as u32).to_le_bytes();
                target_buf[offset..offset + 4].copy_from_slice(&val);
                Ok(())
            }
            R_386_PC32 => {
                // PC-relative 32-bit displacement: S + A - P
                if offset + 4 > target_buf.len() {
                    return Err(KernelError::Fault);
                }
                let implicit_addend = if a == 0 {
                    let mut b = [0u8; 4];
                    b.copy_from_slice(&target_buf[offset..offset + 4]);
                    i32::from_le_bytes(b) as i64
                } else {
                    a
                };
                let disp = (s as i64)
                    .wrapping_add(implicit_addend)
                    .wrapping_sub(p as i64);
                let val = (disp as i32).to_le_bytes();
                target_buf[offset..offset + 4].copy_from_slice(&val);
                Ok(())
            }
            _ => Err(KernelError::NotSupported),
        }
    }
}

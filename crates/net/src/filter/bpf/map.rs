// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Extended Berkeley Packet Filter (eBPF) map storage and lookup tables.

#![allow(static_mut_refs)]

pub const MAX_BPF_MAPS: usize = 8;
pub const MAX_MAP_ENTRIES: usize = 16;

/// In-kernel map types for stateful BPF programs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BpfMapType {
    Hash = 1,
    Array = 2,
    ProgArray = 3,
}

impl BpfMapType {
    /// Return human-readable name of BPF map type.
    pub fn as_str(&self) -> &'static str {
        match self {
            BpfMapType::Hash => "hash",
            BpfMapType::Array => "array",
            BpfMapType::ProgArray => "prog_array",
        }
    }
}

/// Generic BPF map descriptor and static storage table.
#[derive(Debug, Copy, Clone)]
pub struct BpfMap {
    pub id: u32,
    pub map_type: BpfMapType,
    pub key_size: u32,
    pub val_size: u32,
    pub max_entries: u32,
    pub entries: [(u32, u64); MAX_MAP_ENTRIES],
    pub entries_count: usize,
    pub name: [u8; 16],
    pub name_len: usize,
    pub in_use: bool,
}

pub static mut BPF_MAPS: [BpfMap; MAX_BPF_MAPS] = [BpfMap {
    id: 0,
    map_type: BpfMapType::Hash,
    key_size: 4,
    val_size: 8,
    max_entries: MAX_MAP_ENTRIES as u32,
    entries: [(0, 0); MAX_MAP_ENTRIES],
    entries_count: 0,
    name: [0u8; 16],
    name_len: 0,
    in_use: false,
}; MAX_BPF_MAPS];

/// Helper to create an in-kernel map.
///
/// # Safety
/// Caller must ensure exclusive access to BPF_MAPS.
pub unsafe fn create_map_internal(
    slot: usize,
    id: u32,
    map_type: BpfMapType,
    name: &str,
    key_size: u32,
    val_size: u32,
    max_entries: u32,
) {
    if slot >= MAX_BPF_MAPS {
        return;
    }
    let mut name_buf = [0u8; 16];
    let b = name.as_bytes();
    let to_copy = b.len().min(16);
    name_buf[..to_copy].copy_from_slice(&b[..to_copy]);

    BPF_MAPS[slot] = BpfMap {
        id,
        map_type,
        key_size,
        val_size,
        max_entries,
        entries: [(0, 0); MAX_MAP_ENTRIES],
        entries_count: 0,
        name: name_buf,
        name_len: to_copy,
        in_use: true,
    };
}

/// Get all registered BPF maps.
pub fn get_maps() -> [BpfMap; MAX_BPF_MAPS] {
    crate::filter::bpf::vm::ensure_initialized();
    unsafe { BPF_MAPS }
}

/// Look up value in map.
pub fn map_lookup(map_id: u32, key: u32) -> Option<u64> {
    crate::filter::bpf::vm::ensure_initialized();
    unsafe {
        for map in BPF_MAPS.iter() {
            if map.in_use && map.id == map_id {
                for i in 0..map.entries_count {
                    if map.entries[i].0 == key {
                        return Some(map.entries[i].1);
                    }
                }
            }
        }
    }
    None
}

/// Update key-value in map.
pub fn map_update(map_id: u32, key: u32, val: u64) -> Result<(), &'static str> {
    crate::filter::bpf::vm::ensure_initialized();
    unsafe {
        for map in BPF_MAPS.iter_mut() {
            if map.in_use && map.id == map_id {
                for i in 0..map.entries_count {
                    if map.entries[i].0 == key {
                        map.entries[i].1 = val;
                        return Ok(());
                    }
                }
                if map.entries_count < MAX_MAP_ENTRIES {
                    let idx = map.entries_count;
                    map.entries[idx] = (key, val);
                    map.entries_count += 1;
                    return Ok(());
                } else {
                    return Err("Map is full");
                }
            }
        }
    }
    Err("Map not found")
}

/// Delete key from map.
pub fn map_delete(map_id: u32, key: u32) -> Result<(), &'static str> {
    crate::filter::bpf::vm::ensure_initialized();
    unsafe {
        for map in BPF_MAPS.iter_mut() {
            if map.in_use && map.id == map_id {
                for i in 0..map.entries_count {
                    if map.entries[i].0 == key {
                        for j in i..(map.entries_count - 1) {
                            map.entries[j] = map.entries[j + 1];
                        }
                        map.entries_count -= 1;
                        return Ok(());
                    }
                }
                return Err("Key not found in map");
            }
        }
    }
    Err("Map not found")
}

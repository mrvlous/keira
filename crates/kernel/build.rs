// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra
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

//! Build script propagating root Kconfig `.config` toggles into Rust `--cfg` flags.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

const TRACKED_KCONFIG_KEYS: &[&str] = &[
    "USERLAND",
    "SMP",
    "ASLR",
    "W_XOR_X",
    "BUDDY_ALLOCATOR",
    "MEMORY_ZONES",
    "SCHED_WORK_STEALING",
    "WAIT_QUEUES",
    "WORK_QUEUES",
    "IO_URING",
    "FS_FAT",
    "FS_EXT4",
    "FS_PROCFS",
    "NET_STACK",
    "NET_E1000",
    "NET_SOCKETS",
    "RLIMIT",
    "CRYPTO_ENTROPY",
];

fn parse_kconfig_booleans(content: &str) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if let Some(rest) = line.strip_prefix("CONFIG_") {
            if let Some((key, val)) = rest.split_once('=') {
                map.insert(key.to_string(), val == "y");
            }
        } else if let Some(rest) = line.strip_prefix("# CONFIG_") {
            if let Some(key) = rest.strip_suffix(" is not set") {
                map.insert(key.to_string(), false);
            }
        }
    }
    map
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_default());
    let config_path = manifest_dir.join("../../.config");

    println!("cargo:rerun-if-changed={}", config_path.display());
    println!("cargo:rerun-if-env-changed=KEIRA_USERLAND");

    let parsed = fs::read_to_string(&config_path)
        .ok()
        .map(|s| parse_kconfig_booleans(&s));

    for &key in TRACKED_KCONFIG_KEYS {
        let cfg_name = format!("kconfig_{}", key.to_ascii_lowercase());
        println!("cargo::rustc-check-cfg=cfg({cfg_name})");

        let mut enabled = match &parsed {
            Some(map) => *map.get(key).unwrap_or(&false),
            None => true,
        };

        if key == "USERLAND" {
            if let Ok(env_val) = env::var("KEIRA_USERLAND") {
                enabled = env_val != "0" && !env_val.is_empty();
            }
        }

        if enabled {
            println!("cargo::rustc-cfg={cfg_name}");
        }
    }
}

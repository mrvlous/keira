// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for background service records and configuration parser.

use super::config::{parse_conf_into_record, parse_u32};
use super::daemon::{ServiceLogEntry, ServiceRecord, ServiceState, MAX_LOG_LINES};

#[test]
fn test_service_log_entry_empty() {
    let entry = ServiceLogEntry::empty();
    assert_eq!(entry.len, 0);
    assert_eq!(entry.timestamp_ms, 0);
    assert_eq!(entry.as_str(), "");
}

#[test]
fn test_service_record_empty() {
    let record = ServiceRecord::empty();
    assert_eq!(record.name_len, 0);
    assert_eq!(record.name_str(), "");
    assert_eq!(record.desc_str(), "");
    assert_eq!(record.conf_path_str(), "");
    assert_eq!(record.state, ServiceState::Stopped);
    assert!(!record.enabled);
}

#[test]
fn test_service_record_log_event() {
    let mut record = ServiceRecord::empty();
    record.log_event("Started test service", 1000);
    assert_eq!(record.log_head, 1);
    assert_eq!(record.logs[0].as_str(), "Started test service");
    assert_eq!(record.logs[0].timestamp_ms, 1000);

    for i in 1..10 {
        record.log_event("Tick event", (i as u64) * 1000);
    }
    assert_eq!(record.log_head, 10);
    let last_idx = (record.log_head - 1) % MAX_LOG_LINES;
    assert_eq!(record.logs[last_idx].as_str(), "Tick event");
}

#[test]
fn test_parse_u32() {
    assert_eq!(parse_u32("123"), Some(123));
    assert_eq!(parse_u32("0"), Some(0));
    assert_eq!(parse_u32(""), None);
    assert_eq!(parse_u32("abc"), None);
}

#[test]
fn test_parse_conf_into_record() {
    let conf = r#"
# Configuration comment
name=testd
description=Test Daemon Subsystem
enabled=true
auto_restart=yes
port=8080
interval=15
"#;
    let mut record = ServiceRecord::empty();
    parse_conf_into_record(conf, &mut record);

    assert_eq!(record.name_str(), "testd");
    assert_eq!(record.desc_str(), "Test Daemon Subsystem");
    assert!(record.enabled);
    assert!(record.auto_restart);
    assert_eq!(record.port, 8080);
    assert_eq!(record.interval_secs, 15);
}

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Multi-User Accounts & Authentication Subsystem

This document details user account management, password hashing, credential verification, and user sessions in Keira Kernel.

---

## Authentication Architecture

```mermaid
graph TD
    Login["login / auth command"] --> AuthEngine["Kernel Authentication Engine"]
    AuthEngine --> Hash["SHA-256 / HMAC Password Hashing"]
    AuthEngine --> PasswdDB["/config/sys/passwd on FAT16 Storage"]
    PasswdDB --> Validate["Verify Password Hash Match"]
    Validate -->|Success| SetSession["Set Task UID / GID Context"]
    Validate -->|Failure| Deny["Access Denied"]
```

---

## User Database File Format (`/config/sys/passwd`)

```text
username:password_hash:uid:gid:home_dir:shell
admin:5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8:0:0:/users/admin:/system/bin/shell
user:e6c279f628cffd8023d9205572d9b00607f404c816765f546438fb184d3b9a4c:1000:1000:/users/user:/system/bin/shell
```

---

## Core API (`crates/task/src/security/mod.rs`)

```rust
/// Validate user credentials against persistent password database.
pub fn authenticate_user(user: &str, pass: &str) -> bool;

/// Query UID and GID for a target username.
pub fn get_user_id(username: &str) -> Option<(u32, u32)>;
```

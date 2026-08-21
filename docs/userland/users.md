<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Multi-User Accounts & Authentication Subsystem

This document details user account management, password hashing, and login authentication in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides multi-user privilege separation (`user` command) backed by `/config/sys/passwd` storage on FAT16 disk storage.

---

## 2. Password Database Format

Entries in `/config/sys/passwd` follow standard UNIX format:
```text
username:password
admin:keira
```

---

## 3. Kernel APIs

*   `pub fn authenticate_user(user: &str, pass: &str) -> bool`: Validates credentials against SHA-256 password hash.

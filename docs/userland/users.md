# Multi-User Accounts & Authentication Subsystem

This document details user account management, password hashing, and login authentication in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides multi-user privilege separation (`user` command) backed by `/system/etc/passwd` storage on FAT16 disk storage.

---

## 2. Password Database Format

Entries in `/system/etc/passwd` follow standard UNIX format:
```text
username:password_hash:uid:gid:home_dir:shell
admin:$sha256$...:0:0:/users/admin:/system/bin/shell
```

---

## 3. Kernel APIs

*   `pub fn authenticate_user(user: &str, pass: &str) -> bool`: Validates credentials against SHA-256 password hash.

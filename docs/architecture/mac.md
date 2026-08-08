<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Mandatory Access Control (MAC / SELinux) Security Engine

This document details path-based access control, process capability bounding, and security policy enforcement in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides Mandatory Access Control ([mac.rs](../../kernel/src/task/mac.rs)) enforcing path-based security rules to prevent unprivileged processes from accessing restricted kernel files.

---

## 2. Security Check Evaluation

Before executing VFS inode operations (`read`, `write`, `unlink`, `exec`), the kernel queries `check_path_access(pid, path, mask)`.

---

## 3. Kernel APIs

*   `pub fn check_path_access(pid: u64, path: &str, mask: u32) -> bool`: Evaluates path access rules for process `pid`.

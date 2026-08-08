<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# POSIX File Security & Protection Flags

This document details file attribute security, read-only protection, and access permissions in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides file attribute protection (`protect` command) setting FAT16 read-only and read-write flags to prevent unauthorized file deletion or mutation.

---

## 2. Protection Flags

*   `FAT16_ATTR_READ_ONLY` (`0x01`): Protects file from write and delete operations.
*   `FAT16_ATTR_SYSTEM` (`0x04`): Marks file as system core binary.

---

## 3. Kernel APIs

*   `pub fn set_file_protection(path: &str, readonly: bool) -> Result<(), &'static str>`: Modifies directory entry attribute byte.

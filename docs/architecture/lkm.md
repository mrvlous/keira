<!--
SPDX-License-Identifier: GPL-2.0-only

Keira Kernel - Operating System Kernel
Copyright (C) 2026 Moh. Ananda Firmansyah Putra
-->

# Loadable Kernel Modules (LKM) & Dynamic Symbol Resolution

This document details dynamically loadable kernel modules, `kallsyms` symbol lookup, and module lifecycle management in Keira Kernel.

---

## 1. Subsystem Overview

Keira Kernel provides dynamic kernel module loading (**Syscall 34 `sys_init_module`** and **Syscall 35 `sys_delete_module`**) allowing kernel code extension at runtime.

---

## 2. System Call Interface

```c
// Syscall 34: Load ELF kernel module
long sys_init_module(void *module_image, unsigned long len, const char *param_values);

// Syscall 35: Unload kernel module
long sys_delete_module(const char *name, unsigned int flags);
```

---

## 3. Kernel APIs

*   `pub fn init_module(image_ptr: *const u8, len: usize) -> Result<(), &'static str>`: Relocates ELF symbols and executes `init_module()`.
*   `pub fn delete_module(name: &str) -> Result<(), &'static str>`: Executes `cleanup_module()` and frees kernel heap memory.

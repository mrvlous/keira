<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 23: KFS Coreutils File Operations Architecture (`mkdir`, `touch`, `rm`) & Syscall 88

## Overview

Milestone 23 advances Keira toward **KFS (Keira From Scratch)** and v0.8.0 self-hosting capabilities by establishing the canonical userspace filesystem manipulation plane. Prior to this milestone, directory and file node creation depended on host image generation tools (`mmd`, `mkfs.fat`). With Milestone 23, userspace tasks executing in Ring 3 can independently instantiate, touch and remove files and nested directory trees directly within the running system.

---

## Architectural Breakdown

### 1. Kernel Syscall 88 (`SYS_MKDIR`)
- **System Call Table**: Added `SYS_MKDIR = 88` in `crates/syscall/src/table/numbers/io.rs`, registered in `table/vectors/names.rs` and categorized under `is_io_syscall`.
- **Dispatcher & Handler**: Implemented `handle_mkdir(arg1, arg2)` in `crates/syscall/src/dispatcher/handlers/fs.rs`, enforcing Mandatory Access Control (`MAC_WRITE`) and routing path strings into `keira_fs::vfs::create_dir`.
- **Error Codes**: Introduced `EISDIR` (21) and `ENOTEMPTY` (39) to standard errno codes in `crates/syscall/src/user_copy/errno/codes.rs`.
- **Unlink Enhancement**: Updated `handle_unlink` to map empty directory deletion and return `ENOTEMPTY` when directories retain active children.

### 2. Freestanding C Standard Library (libc) Bindings
- **Kernel Syscall Wrapper**: Added `sys_mkdir(const char *pathname, mode_t mode)` in `userland/lib/syscall/fs/file.c` via `syscall2(SYS_MKDIR, ...)`.
- **POSIX `mkdir()` Binding**: Replaced the legacy stub in `userland/lib/stat/mode/chmod.c` with a fully functioning `mkdir()` function that translates kernel return codes into `errno` (`EEXIST`, `ENOENT`, `EACCES`, `EINVAL`).
- **POSIX `rmdir()` Binding**: Added `rmdir(const char *pathname)` in `userland/lib/unistd/fs/fs.c` wrapping `sys_unlink`.

### 3. Userspace Core Utilities (`/bin`)
- **`/bin/mkdir.elf` (`userland/bin/mkdir/main.c`)**:
  - Implements directory creation with standard error reporting.
  - Supports `-p` (`--parents`) for multi-level hierarchical tree creation.
  - Supports `-v` (`--verbose`) for created directory diagnostics.
- **`/bin/touch.elf` (`userland/bin/touch/main.c`)**:
  - Instantiates empty files via `open(path, O_WRONLY | O_CREAT, 0644)`.
  - Supports `-c` (`--no-create`) to prevent creating files that do not exist.
- **`/bin/rm.elf` (`userland/bin/rm/main.c`)**:
  - Unlinks regular files and prevents deleting directories unless `-r` (`-R`, `--recursive`) is specified.
  - Uses `opendir`/`readdir` to walk and delete nested directory hierarchies.
  - Supports `-f` (`--force`) to ignore nonexistent targets cleanly.

---

## Verification Summary

- **Unit Test Battery**: 100% pass across all workspace crates (`cargo test --workspace`).
- **Dual-Architecture Build**: `make all ARCH=x86_64` and `make all ARCH=i686` succeeded with 0 errors and 0 warnings.
- **Live QEMU Validation**:
  - Tested `mkdir -pv /tmp/src/keira` -> directories created cleanly.
  - Tested `touch /tmp/src/keira/main.c` -> empty file instantiated and verified via `ls`.
  - Tested `rm /tmp/src/keira/main.c` -> file deleted cleanly.
  - Tested `rm -rv /tmp/src` -> recursive tree unlinked cleanly.

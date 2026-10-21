<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 32: VFS Symbolic Links, POSIX Traversal & Freestanding Core Link Utility

## Overview

Milestone 32 introduces canonical symbolic link abstractions to Keira's monolithic Virtual File System (VFS), system call dispatch layer, freestanding C library (libc) and Ring 3 core utilities. Following Keira's bare-metal purity invariants, symbolic links are managed entirely within Ring 0 without requiring host distro services or foreign runtime environments. Transparent path dereferencing enables all file, stream and binary execution operations to navigate link targets seamlessly up to a recursion depth limit of 8.

Milestone 32 delivers five primary achievements:

1. **In-Kernel VFS Symbolic Link Registry (`crates/fs`)**:
   - Implemented `NodeKind::Symlink` and `is_symlink: bool` in `FileStat` metadata structures.
   - Built a thread-safe, interrupt-safe spinlock-protected symbolic link registry (`SYMLINK_TABLE`) with fixed-capacity entries (`MAX_SYMLINKS = 64`, `MAX_SYMLINK_PATH = 128` bytes).
   - Implemented core link management routines: `create_symlink`, `read_symlink`, `remove_symlink`, `is_symlink` and recursive `resolve_symlinks`.
   - Wired transparent dereferencing across VFS dispatch operations: `open`, `read`, `write`, `get_file_size`, `is_dir` and `load_elf`.
   - Guaranteed unlink independence: removing a symbolic link with `remove_entry()` unbinds the link descriptor without altering or destroying the target file.

2. **System Call Vector Expansion (`crates/syscall`)**:
   - Implemented Vector 92 (`SYS_SYMLINK`) and Vector 93 (`SYS_READLINK`) in the system call table, name registry and categorization subsystem (`SyscallCategory::Io`).
   - Implemented secure handlers in `crates/syscall/src/dispatcher/handlers/fs.rs`:
     - `handle_symlink`: Validates user string buffers, normalizes relative paths against process working directory (`cwd`), verifies Mandatory Access Control write permissions (`MAC_WRITE`) and registers the link.
     - `handle_readlink`: Validates user buffer bounds, normalizes path, checks read permissions (`MAC_READ`), extracts link target path and safely copies bytes back to user memory via `copy_to_user()`.
   - Added standard POSIX error codes: `ENAMETOOLONG` (36) and `ELOOP` (40).

3. **Freestanding C Library Integration (`userland/include` & `userland/lib`)**:
   - Defined `SYS_SYMLINK 92`, `SYS_READLINK 93`, `ENAMETOOLONG 36` and `ELOOP 40` across userland headers (`sys/syscall.h`, `unistd.h`, `errno.h`).
   - Implemented assembly traps `sys_symlink()` and `sys_readlink()` in `userland/lib/syscall/fs/file.c`.
   - Implemented POSIX-compliant wrappers `symlink()` and `readlink()` in `userland/lib/unistd/fs/fs.c` with standard `errno` translation.

4. **Freestanding Ring 3 Core Utility `/bin/ln` (`userland/bin/ln/`)**:
   - Developed freestanding `/bin/ln` utility supporting standard POSIX options:
     - `-s`, `--symbolic`: Create symbolic links.
     - `-f`, `--force`: Remove existing destination links before linking.
     - `-v`, `--verbose`: Print link confirmation output (`'link' -> 'target'`).
     - `-h`, `--help`: Display usage manual.
   - Integrated binary into `Makefile` (`LN_ELF`) and packaged into canonical root filesystem (`/bin/ln.elf` and `/bin/ln`).
   - Updated userspace shell manual (`/bin/sh`) to list `ln` in the core utilities directory table.

5. **Universal Verification & Dual-Architecture Parity**:
   - 100% clean builds across both 64-bit (`x86_64`) and 32-bit (`i686`) targets (`make full`).
   - 100% clean linting and formatting without warnings (`make format && make lint`).
   - 100% unit test success across all 12 workspace crates (`make test-unit`).
   - Verified end-to-end QEMU hardware operations: symlink creation, file reading through symlink, force overwrite, direct binary execution through symlink, unlinking, native `kcc` compilation, network API fetch/download and Ring 0 supervisor diagnostics.

---

## Architectural Interaction

```mermaid
graph TD
    subgraph Ring3["Ring 3 Userspace"]
        LnUtil["/bin/ln -s /etc/kernel.cfg /tmp/cfg_link"]
        CatUtil["/bin/cat /tmp/cfg_link"]
        ExecLink["/tmp/my_ls /etc (Symlink to /bin/ls)"]
        RmUtil["/bin/rm /tmp/cfg_link"]
    end

    subgraph Ring0Syscall["Ring 0 System Call Boundary"]
        SysSymlink["SYS_SYMLINK (Vector 92)<br/><i>Normalized CWD + MAC_WRITE Check</i>"]
        SysReadlink["SYS_READLINK (Vector 93)<br/><i>Target Extraction + Copy to User</i>"]
        SysOpen["SYS_OPEN (Vector 6)<br/><i>Follows Symbolic Links</i>"]
        SysUnlink["SYS_UNLINK (Vector 87)<br/><i>Unbinds Link Descriptor</i>"]
    end

    subgraph Ring0VFS["Ring 0 Virtual File System (crates/fs)"]
        SymTable["In-Kernel Symlink Table (SYMLINK_TABLE)<br/><i>Max 64 links, max 128 bytes, max depth 8</i>"]
        PathRouter["VFS Path Router & Dereferencing Engine<br/><i>Transparently routes target to backend</i>"]
        FatFs["FAT16 Filesystem Driver (/etc/kernel.cfg)"]
    end

    LnUtil -->|Invokes| SysSymlink
    SysSymlink -->|Registers Entry| SymTable
    CatUtil -->|Invokes open()| SysOpen
    SysOpen -->|Dereferences via| PathRouter
    PathRouter -->|Queries Target| SymTable
    PathRouter -->|Accesses Target File| FatFs
    ExecLink -->|load_elf() reads through| PathRouter
    RmUtil -->|Invokes unlink()| SysUnlink
    SysUnlink -->|Removes Link Only| SymTable
```

---

## Verification Artifacts

The following visual artifacts record the execution sequence in QEMU:

| Artifact | Phase | Description |
| :--- | :--- | :--- |
| `m32_01_boot.png` | Phase 1 | System boot and initialization into interactive Ring 3 shell |
| `m32_02_ln_create.png` | Phase 2 | Symbolic link creation via `/bin/ln -s /etc/kernel.cfg /tmp/cfg_link` |
| `m32_03_cat_symlink.png` | Phase 3 | Transparent file reading through symlink verifying target configuration contents |
| `m32_04_ln_force_verbose.png` | Phase 4 | Force overwrite and confirmation output using `/bin/ln -sfv /bin/ls /tmp/my_ls` |
| `m32_05_exec_via_symlink.png` | Phase 5 | Direct binary execution of `/bin/ls` invoked through symbolic link `/tmp/my_ls /etc` |
| `m32_06_rm_symlink.png` | Phase 6 | Unlinking symbolic link with `/bin/rm` and verifying original target file remains intact |
| `m32_07_kcc_compiler.png` | Phase 7 | Native in-kernel KCC compilation and execution in Ring 3 (`/tmp/main.c` -> `/tmp/hello`) |
| `m32_08_net_fetch_download.png` | Phase 8 | Network reachability ping, HTTP JSON API fetch and remote payload download |
| `m32_09_supervisor_cmds.png` | Phase 9 | Dropping to Ring 0 supervisor console and running complete diagnostic command suite |
| `m32_10_reenter_userland.png` | Phase 10 | Seamless re-entry into Ring 3 userland shell from supervisor console via `sh` |

---

## Summary Metrics

- **System Call Vectors**: 93 active POSIX syscall vectors.
- **Freestanding Core Utilities**: 30 canonical userland binaries in `/bin`.
- **Unit Tests**: 100% pass rate across all 12 crates (keira-arch, keira-core, keira-crypto, keira-fs, keira-io, keira-ipc, keira-kernel, keira-mem, keira-net, keira-shell, keira-syscall and keira-task).
- **Architecture Targets**: Parity across `x86_64` and `i686` without warnings.

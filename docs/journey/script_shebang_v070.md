<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Milestone 21: Canonical Shell Script Execution & Kernel Shebang (`#!`) Support

## Overview

Milestone 21 advances Keira's operating system architecture along the **Road to v0.7.0** by delivering native POSIX/Linux-grade shell script execution and kernel-level shebang (`#!`) interpreter recognition across both the supervisor control plane and unprivileged Ring 3 userspace.

Prior to Milestone 21, Keira's program execution subsystem operated exclusively on compiled ELF executables (such as `sysinfo.elf` or `kcc.elf`). Attempting to execute a script file resulted in an invalid ELF header format error. With Milestone 21, Keira bridges this gap by introducing:

1. **Kernel Shebang (`#!`) Parser**: A freestanding `#![no_std]` interpreter header parser (`crates/fs/src/elf/loader/shebang.rs`) that identifies shebang magic bytes (`#!`), extracts the interpreter path (e.g. `/bin/sh` or `/bin/sh.elf`) and parses optional interpreter arguments.
2. **`sys_execve` Shebang Delegation**: The kernel process dispatcher (`crates/syscall/src/dispatcher/handlers/process.rs`) inspects file headers upon `execve`. When a shebang script is detected, the kernel re-routes execution to the specified interpreter, prepending interpreter flags and the script path to the argument vector.
3. **Userspace Shell Script Engine**: The standalone Ring 3 shell (`userland/bin/sh/main.c`) gains script execution capabilities (`sh <script_file>`), file comment stripping (`#`), blank line filtering and sequential command execution via standard `execve()` and `waitpid()` system calls.
4. **Direct Supervisor `.sh` Invocation**: Users can invoke `.sh` scripts directly from the supervisor root console (`keira:/# demo.sh` or `keira:/# /etc/init.sh`) without manual `sh` prefixes, matching direct binary execution ergonomics.
5. **System Bootstrap Scripts**: Canonical system scripts (`/etc/init.sh` and `/bin/demo.sh`) are integrated into the FAT16 disk image, providing initial configuration automation and verified shebang demonstration.

---

## Architectural Motivation: Scripting in Pure Kernel Systems

In canonical UNIX and Linux systems, script execution is deeply integrated into the kernel program loader. When a process invokes `execve()` on a text file starting with `#!`, the kernel does not reject it as an invalid binary format. Instead, the kernel parses the interpreter line, loads the interpreter ELF binary into the process address space and passes the script file path as `argv[1]`.

```
+-------------------------------------------------------------------------+
|                        PROGRAM INVOCATION ATTEMPT                       |
|                       sys_execve("/bin/demo.sh")                        |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                     KERNEL SHEBANG RECOGNITION                          |
|               Magic Bytes Check: bytes[0..2] == "#!"                   |
|                                                                         |
|  [#!/bin/sh] -> Interpreter: /bin/sh.elf, Script: /bin/demo.sh          |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                        INTERPRETER DISPATCH                             |
|          sys_execve("/bin/sh.elf", ["/bin/sh.elf", "/bin/demo.sh"])     |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                   USERSPACE SCRIPT EXECUTION (RING 3)                    |
|                                                                         |
|   1. Open /bin/demo.sh via open()                                       |
|   2. Read line-by-line via fgets()                                      |
|   3. Strip comments ('#') and whitespace                                |
|   4. Execute commands sequentially via execve() and waitpid()           |
+-------------------------------------------------------------------------+
```

Without this mechanism, automating system startup sequences, running test harnesses and orchestrating userland software requires hardcoded C binaries. Introducing shebang recognition and shell script execution provides the foundational building block for initialization scripts (`/etc/init.sh`), daemon runners and automated test pipelines.

---

## Technical Implementation

### 1. Freestanding Shebang Parser (`crates/fs/src/elf/loader/shebang.rs`)

The shebang parser operates entirely in `#![no_std]` without dynamic heap allocation, analyzing the raw leading bytes of candidate executables:

```rust
pub fn parse_shebang(bytes: &[u8]) -> Option<(&str, Option<&str>)> {
    if bytes.len() < 2 || bytes[0] != b'#' || bytes[1] != b'!' {
        return None;
    }

    // Find end of first line
    let mut eol = 2;
    while eol < bytes.len() && bytes[eol] != b'\n' && bytes[eol] != b'\r' {
        eol += 1;
    }

    let line = &bytes[2..eol];
    // Extract interpreter path and optional single argument...
    Some((interp_str, opt_arg))
}
```

The parser correctly handles varying whitespace, trailing carriage returns (`\r\n`), optional interpreter flags (such as `#!/bin/sh -e`) and gracefully falls back to `None` for pure ELF binaries.

### 2. Kernel `sys_execve` Shebang Delegation (`crates/syscall/src/dispatcher/handlers/process.rs`)

When `sys_execve` (Syscall 5) is invoked, the handler reads the initial 128 bytes from the target file via VFS offset reading:

```rust
let mut shebang_hdr = [0u8; 128];
if let Ok(hdr_len) = keira_fs::vfs::read_file_offset(filename_str, 0, &mut shebang_hdr) {
    if hdr_len >= 2 && shebang_hdr[0] == b'#' && shebang_hdr[1] == b'!' {
        if let Some((interpreter, opt_arg)) =
            keira_fs::elf::parse_shebang(&shebang_hdr[..hdr_len])
        {
            // Resolve /bin/sh to /bin/sh.elf if necessary
            let resolved = if keira_fs::vfs::exists(interpreter) {
                interpreter
            } else if interpreter == "/bin/sh" && keira_fs::vfs::exists("/bin/sh.elf") {
                "/bin/sh.elf"
            } else {
                interpreter
            };
            // Restructure arguments: [interpreter, opt_arg, script_path, ...]
            ...
        }
    }
}
```

This ensures any userland process calling `execve()` on a script file automatically delegates to the configured interpreter transparently.

### 3. Userspace Shell Script Runner (`userland/bin/sh/main.c`)

The standalone shell was enhanced to support script execution mode:

- **Comment Stripping**: The tokenizer detects `#` characters outside quotes and terminates argument parsing for that line.
- **Script File Reading (`run_script_file`)**: Opens the target file via standard C file I/O (`fopen`), reads lines sequentially via `fgets`, strips line terminators and feeds non-empty lines into `run_command_string`.
- **Process Replacement**: Built-ins (`cd`, `pwd`, `echo`, `export`) execute inline, while external commands invoke `execve()` directly followed by `waitpid()`, ensuring predictable serial execution without orphaned child processes.

### 4. Direct `.sh` Execution in Supervisor Console (`crates/shell/src/cmds/proc/task/run.rs`)

The supervisor program runner identifies `.sh` files during command dispatch:
- Searches target paths with `.sh` extensions in both current directory and `/bin/`.
- Displays contextual status: `Executing script: <path>` for scripts and `Loading ELF binary: <path>` for binaries.
- Automatically invokes `/bin/sh.elf` with the script path when a shebang or `.sh` extension is present.

### 5. Disk Image Provisioning (`Makefile`)

The build system automatically copies system scripts into the FAT16 system disk image:
- `/bin/demo.sh`: Demonstration script verifying shebang execution, `echo` and `pwd`.
- `/etc/init.sh`: System initialization script invoking `sysinfo.elf` and displaying boot telemetry.

---

## Verification & Stability Matrix

All subsystems across both `x86_64` and `i686` targets were verified to **100% zero errors and zero warnings**:

| Verification Target | Command / Harness | Result | Notes |
| :--- | :--- | :--- | :--- |
| **Workspace Unit Tests** | `cargo test --workspace` | `PASS (100%)` | 55 shell tests, 24 syscall tests, 27 task tests and 4 shebang parser unit tests passed |
| **Rust Linter** | `cargo clippy --workspace` | `PASS (0 warn)` | Clean across dev and release configurations |
| **Code Formatting** | `make format` | `PASS` | All Rust, C and Assembly sources formatted to style guide |
| **C Static Analysis** | `make lint` (`clang-tidy`) | `PASS (0 err)` | Clean static analysis across all userland sources |
| **Dual-Arch Build** | `make full` | `PASS` | `x86_64` and `i686` ISO and FAT16 disk images compiled |
| **Direct Script Invocation** | QEMU Live Shell | `PASS` | `demo.sh` executed directly via shebang: `[DEMO] Shebang execution active via /bin/sh` |
| **Shell Script Execution** | QEMU Live Shell | `PASS` | `sh /etc/init.sh` ran line-by-line, spawned `sysinfo.elf` and completed cleanly |
| **KCC Compilation & Run** | `test_kcc_single.py` | `PASS` | KCC native compiler generated working Ring 3 ELF binary |
| **Live Network APIs** | `test_multi_api.py` | `PASS` | IP geolocation, HTTP echo and download tested |
| **Dual-Arch Command Battery** | `test_dual_arch_all_cmds.py` | `PASS` | 61 commands executed on both architectures |

---

## Conclusion

Milestone 21 establishes full script execution capabilities for Keira. By implementing kernel-level shebang recognition, unprivileged shell script interpretation and direct console execution, Keira achieves genuine UNIX-like scripting ergonomics while preserving memory safety and kernel isolation.

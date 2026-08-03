# POSIX File Permissions, Redirection & Multi-TTY Subsystem

This document details the POSIX file security model, file redirection engine, and Multi-Virtual Terminal (TTY) subsystem in Keira Kernel.

---

## 1. POSIX File Security & Permission Model

Keira Kernel enforces POSIX-compliant multi-user access control and file security:

*   **User Ownership & ACL Enforcement**: File access permissions are evaluated against the active user session context (`CURRENT_USER`).
*   **Administrative Superuser Context (`admin`)**: Users logged in under the `admin` account possess root-level system privileges and bypass standard DAC file permission restrictions (*UNIX root behavior*).
*   **User Directory Isolation**: Private home directory paths (e.g. `/users/<username>/`) enforce strict access control. Attempts by non-admin users to read, write, create, or delete entries inside another user's home directory yield `Permission denied`.
*   **System Configuration Protection**: Critical kernel configuration paths (e.g. `/system/etc/passwd`, `/system/etc/hostname`) restrict write and modification access to administrative contexts.

### Command Interface (`protect` / `chmod`)

Implemented in [protect.rs](../../kernel/src/shell/cmds/protect.rs):

```bash
# Set numeric octal POSIX permission modes
protect secret.txt 700   # Owner read/write/execute; others no access
protect note.txt 755     # Owner read/write/execute; others read/execute
protect file.txt 644     # Owner read/write; others read-only

# Legacy text mode attributes
protect file.txt readonly   # Mode 0444
protect file.txt readwrite  # Mode 0644
```

---

## 2. Standard I/O Redirection & Pipe Pipeline Engine

Implemented in [executor.rs](../../kernel/src/shell/executor.rs):

### Output Overwrite Redirection (`>`)
Redirects standard console output to overwrite target file:
```bash
list > file_list.txt
```

### Output Append Redirection (`>>`)
Redirects standard console output to append to end of existing file:
```bash
say "New log entry" >> log.txt
```

### Input File Redirection (`<`)
Feeds file content stream as standard input:
```bash
search error < system.log
```

### Multi-Stage Pipe Pipeline (`|`)
Chains multiple commands sequentially via kernel memory pipes:
```bash
list /system/bin | search elf | view
```

---

## 3. Multi-Virtual Terminal Subsystem (`/dev/tty`)

Implemented in [tty.rs](../../kernel/src/io/tty.rs):

*   **Virtual Terminals**: Keira Kernel supports 3 concurrent virtual terminal instances (`TTY 1`, `TTY 2`, `TTY 3`).
*   **Real-time Switching**: Terminal instances maintain isolated display buffers and session states. Users can switch between active terminals seamlessly using keyboard shortcuts:
    *   `Alt + F1`: Switch to Virtual Terminal 1 (`/dev/tty1`)
    *   `Alt + F2`: Switch to Virtual Terminal 2 (`/dev/tty2`)
    *   `Alt + F3`: Switch to Virtual Terminal 3 (`/dev/tty3`)

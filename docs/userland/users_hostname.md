# Multi-User Account Management & System Hostname Subsystem

This document details the multi-user account management architecture, persistent authentication storage, system hostname configuration, and UNIX-compliant privilege separation in Keira Kernel.

---

## 1. Subsystem Architecture Overview

Keira Kernel provides a native multi-user subsystem and dynamic hostname management engine:
*   **Persistent User Storage**: Accounts and credentials are stored in `/system/etc/passwd` on the FAT16 disk storage using the format `username:password` per record line.
*   **Persistent Hostname Storage**: System identity is stored in `/system/etc/hostname` on the FAT16 disk storage.
*   **Automatic Parent Directory Provisioning**: The user and hostname drivers automatically ensure that directory hierarchies (`/system`, `/system/etc`, `/users`) exist on the active storage partition before performing file operations.

---

## 2. Boot Behavior & Initial Privileges

1.  **Boot Defaults**:
    *   During boot, `shell::run_boot_script()` loads the persisted hostname from `/system/etc/hostname` (defaulting to `keira`) into global state.
    *   The kernel initializes the interactive terminal session directly under the **`admin`** superuser context with administrative privileges enabled (`IS_ADMIN = true`).
2.  **Interactive Terminal Prompt**:
    *   The prompt dynamically reflects the active context using the standard format: `user@hostname pathsymbol`.
    *   Examples: `admin@keira ~/` or `marvelous@keira-box ~/`.

---

## 3. Privilege Separation Security Model (UNIX/Linux Standard)

Keira Kernel enforces standard UNIX/Linux privilege separation rules:

| Source Context | Target Login Context | Password Prompt | Security Rule |
| :--- | :--- | :---: | :--- |
| **`admin`** | Regular User (e.g. `marvelous`) | ❌ **No** | Instant context switch (*UNIX root `su` behavior*) |
| Regular User (`marvelous`) | **`admin`** | ✅ **Yes** | Validates against `/system/etc/passwd` (default `keira`) |
| Regular User (`marvelous`) | Another User (`budi`) | ✅ **Yes** | Validates against target user password in `/system/etc/passwd` |

### Authentication Retry Fallback System
When executing `login <user>` or `please <command>` as a non-admin user:
*   The authentication engine permits up to **3 consecutive retry attempts** before terminating the request.
*   If an incorrect password is submitted, the terminal notifies the user with `attempt X/3` and prompts for re-entry.
*   Upon 3 failed attempts, authentication aborts, resetting retry counters and returning control to the interactive shell.

---

## 4. Shell Command Interfaces

### User Account Management (`user`)
Implemented in [user.rs](../../kernel/src/shell/cmds/user.rs):
*   `user create <username> <password>`: Creates a new user account entry in `/system/etc/passwd` and provisions home directory `/users/<username>/`. *(Password parameter is mandatory)*.
*   `user delete <username>`: Deletes a registered user account from `/system/etc/passwd`. Account `admin` is protected from deletion.
*   `user list`: Lists all registered accounts on the active volume and highlights active user sessions.
*   `user password <username> <new_password>`: Updates password credentials for target account (including `admin`).
*   `user info`: Displays active username, home directory location, and privilege level.

### System Hostname Interface (`hostname`)
Implemented in [hostname.rs](../../kernel/src/shell/cmds/hostname.rs):
*   `hostname`: Queries and displays current system hostname string.
*   `hostname <new_name>`: Updates active hostname in memory and flushes new hostname string to `/system/etc/hostname` on FAT16 disk storage.

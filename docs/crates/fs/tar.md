<!-- SPDX-License-Identifier: GPL-2.0-only -->

# USTAR Initrd RAM Disk Reader

Documentation for initrd archive reader in [`crates/fs/src/tar/`](../../../crates/fs/src/tar).

## Features
- Parses standard POSIX USTAR 512-byte headers (magic `ustar\0`).
- Provides read-only access to embedded kernel boot assets, shell command stubs, system drivers, and default user filesystems.

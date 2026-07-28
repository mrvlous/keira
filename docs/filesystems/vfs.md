# Virtual Filesystem (VFS) Layer

This document details the abstractions, design patterns, and locking mechanisms of the Virtual Filesystem (VFS) layer in Keira Kernel.

## 1. VFS Design and Abstractions
The VFS layer ([vfs.rs](../../kernel/src/fs/vfs.rs)) defines standard traits to present a unified directory tree to user space, abstracting the differences between physical filesystems (like FAT) and virtual ones (like TAR initrd).

### Inode and File System Node Representation
The filesystem exposes resources using standard structures:
*   **File Metadata**: Captures permissions, size, and location info.
*   **VFS Nodes**: Represent directories, regular files, devices, or symbolic links.
*   **File Descriptors**: Map integers in user space to open file descriptors in kernel space.

---

## 2. Mount Points and Directory Routing
Keira organizes all filesystems into a single unified directory tree:
*   `/` (Root): Routed to the read-only TAR boot initrd.
*   `/dev/`: Contains virtual character and block devices.
*   Disk Mounts: Logical FAT volumes are mapped to subdirectories (e.g. `/disk`).

The virtual file dispatcher inspects file paths and routes requests:
1.  **Resolve Path**: Translates relative path components into absolute paths.
2.  **Route Mount**: Finds the longest matching mount prefix in the VFS mount registry.
3.  **Forward Operation**: Delegates the read, write, open, or close call to the registered filesystem driver (e.g. FAT, TAR, DevFS).

---

## 3. File Locking Subsystem
To prevent race conditions during write operations, a task-aware locking system is implemented in [lock.rs](../../kernel/src/fs/lock.rs).

### Lock Representation
Locks are managed using a global static list of `FileLock` entries:
```rust
pub struct FileLock {
    pub path: &'static str,
    pub task_id: usize,
}
```

### Locking Protocol
1.  **Acquire Lock (`acquire_lock`)**:
    *   Iterates through active locks. If the target path is already locked by another task ID, the call returns `Err("File is locked by another process")`.
    *   If no lock exists, a new `FileLock` is added, assigning ownership to the calling task ID.
2.  **Release Lock (`release_lock`)**:
    *   Removes the path lock entry if it is owned by the calling task ID.
3.  **Automatic Task Cleanup (`release_all_locks_for_task`)**:
    *   When a task exits, the scheduler calls this function to reclaim and delete all locks held by the exiting task ID, preventing resource leaks.

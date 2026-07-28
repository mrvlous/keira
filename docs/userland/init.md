# The Init Process

This document details the initialization sequence of the first user-space process (`bin/init`) spawned during system boot.

## 1. Process Spawn
After completing kernel initialization, the scheduler spawns the first user-space process:
*   **Source File**: [init.c](../../user/bin/init.c)
*   **Virtual Mapping**: The kernel loads the compiled ELF binary of `init` and maps it into the user space virtual memory area.
*   **Privilege Level**: Starts in Ring 3.

---

## 2. Bootstrapping and Process Spawning
The entry point `_start` in `init.c` performs the following operations:
1.  **Welcome Message**: Outputs system messages indicating user-space initialization.
2.  **Mount Verification**: Assures that filesystem partitions (e.g. `/disk`) are mounted and available.
3.  **Shell Spawn**: Spawns `/system/bin/shell` processes:
    *   Calls the `fork()` system call to clone the active process context.
    *   In the child process, calls `exec()` to replace the memory space with the interactive shell executable.

---

## 3. System Task Monitoring
After launching system services, the `init` thread enters an infinite monitoring loop:
*   **Orphan Cleanups**: Acts as the parent process to adopt orphan threads when their creators exit.
*   **Task Waiting**: Calls the `wait()` system call to block until child processes exit, reading their status codes to perform task cleanup.
*   **Crash Recovery**: If a critical shell process exits or crashes, the monitoring loop automatically spawns a new shell instance to maintain system availability.

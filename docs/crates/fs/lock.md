<!-- SPDX-License-Identifier: GPL-2.0-only -->

# File Locking & Concurrency Management

Documentation for file locks in [`crates/fs/src/lock/`](../../../crates/fs/src/lock).

## Features
- Task-aware file locks prevent multiple processes from corrupting files during concurrent writes.
- Automatically releases orphaned file locks when tasks terminate.

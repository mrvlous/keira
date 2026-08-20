<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Device Nodes (`/dev/`)

Documentation for device nodes in [`crates/fs/src/dev/`](../../../crates/fs/src/dev).

## Node Types
- `/dev/null`: Discards all written data; returns EOF on read.
- `/dev/zero`: Returns infinite streams of `0x00` bytes.
- `/dev/random`: Cryptographic pseudorandom byte generator.
- `/dev/tty`: Direct access to active virtual terminal.

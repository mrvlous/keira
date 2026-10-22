<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Anonymous Pipes & Zero-Copy Splice

Pipes provide unidirectional byte streaming between processes (`crates/ipc/src/pipe/`).

- [`fifo.md`](fifo.md): Dynamic multi-channel anonymous pipes and POSIX stream semantics.
- [`zero_copy.md`](zero_copy.md): Zero-copy buffer splicing and page movement.

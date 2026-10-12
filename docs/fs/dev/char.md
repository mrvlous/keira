<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Character Device Implementation

* `null`: Discards all writes and returns EOF on read.
* `zero`: Supplies continuous zero bytes on read.
* `random`: Provides pseudo-random entropy bytes.
* `urandom`: Provides cryptographically secure random bytes.
* `tty`: Interactive terminal character stream interface.
* `ptmx`: Pseudoterminal multiplexer master endpoint.
* `sda`: Primary storage block device node alias.

Virtual directory enumeration (`SYS_GETDENTS`) is supported over `/dev`, returning canonical directory entries for all active device nodes.

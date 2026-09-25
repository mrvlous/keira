<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Character Device Implementation

* `console`: Read from keyboard, write to VGA screen.
* `null`: Discards all writes, returns EOF on read.
* `zero`: Supplies continuous zero bytes on read.
* `urandom`: Provides cryptographically secure random bytes.

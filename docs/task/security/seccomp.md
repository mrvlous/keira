<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Seccomp System Call Filtering

* **Strict Mode**: Only permits `read`, `write`, `exit`, and `sigreturn`.
* **Filter Mode**: 128-bit bitmask whitelist allowing fine-grained syscall restrictions.

<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Signal Frame & `sigreturn` Flow

When delivering a signal to a userland handler:
1. Construct a `SigContext` frame on the user stack preserving registers.
2. Push a trampoline return address calling `sys_sigreturn()`.
3. Set `RIP` to the userland signal handler function.
4. When handler returns, `sys_sigreturn()` restores original execution context seamlessly.

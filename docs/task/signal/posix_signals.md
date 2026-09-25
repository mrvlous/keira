<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Signal Delivery & Masking

* **Pending Queue**: Bitmask of signals pending delivery.
* **Blocked Mask**: Configurable signal mask via `sigprocmask()`.
* **Handlers**: Default action (`Terminate`, `Ignore`, `CoreDump`) or custom userland signal handler.

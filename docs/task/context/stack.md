<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel & User Stack Layout

* **Kernel Stack**: 16 KiB page-aligned dedicated stack per task.
* **User Stack**: Allocated at high virtual memory (`0x7FFFFFFF0000` on x86_64, `0xBFFF0000` on i686).

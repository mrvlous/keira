<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Kernel and User Stack Layout

* **Kernel Stack**: Dedicated contiguous frames allocated per task (`TASK_STACK_PAGES = 4` / 16 KiB on x86_64, `TASK_STACK_PAGES = 2` / 8 KiB on i686) matching `PER_CPU_STACK_SIZE` to prevent stack overflow during deep VFS and syscall dispatch chains.
* **User Stack**: Allocated at high virtual memory (`0x7FFFFFFF0000` on x86_64, `0xBFFF0000` on i686).

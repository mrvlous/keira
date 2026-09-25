<!-- SPDX-License-Identifier: GPL-2.0-only -->

# User Pointer Bounds Validation

* **Validation**: `validate_user_ptr(ptr, len, write_mode)` checks that `ptr + len` falls strictly within `[USER_MIN_ADDR, USER_MAX_ADDR]`.
* **Kernel Protection**: Any attempt to read or write kernel space from userland immediately returns `-EFAULT`.

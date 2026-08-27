<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Safe User Space Memory Access (`user_copy`)

The `user_copy` submodule provides validated pointer checks and memory transfer primitives between Ring 0 (kernel space) and Ring 3 (user space).

## Key Functions

- `validate_user_ptr(ptr: *const u8, len: usize) -> Result<(), &'static str>`: Verifies that the requested memory buffer lies strictly within valid userland virtual address bounds.
- `copy_from_user<T: Copy>(user_ptr: *const T) -> Result<T, &'static str>`: Safely copies a value of type `T` from user memory into kernel space after boundary and alignment validation.
- `copy_to_user<T: Copy>(user_ptr: *mut T, val: &T) -> Result<(), &'static str>`: Safely writes a value from kernel space into user memory.
- `read_user_string(user_ptr: *const u8, max_len: usize) -> Result<&'static str, &'static str>`: Reads a null-terminated UTF-8 string from user space safely.
- `errno_to_ret(err: &'static str) -> u64`: Converts descriptive error strings into standard POSIX-compatible negative error numbers (`-EFAULT`, `-EINVAL`, `-EACCES`, etc.).

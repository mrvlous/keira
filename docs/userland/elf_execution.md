<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Ring 3 ELF Execution & Memory Isolation

Documentation for running user mode programs inside Keira Kernel.

## Execution Lifecycle
1. User invokes `run <program.elf>` in the shell.
2. Kernel clones a new PML4 address space via `vmm::clone_kernel_pml4()`.
3. Switches to child PML4 address space (`vmm::switch_address_space()`).
4. Loads ELF segments (`load_elf()`) and allocates a 16-page user stack at `0x7FFFFFE00000`.
5. Executes `jump_to_user(entry_point, user_stack_top)` via assembly trampoline (`sysretq`).
6. On program exit (`sys_exit`), kernel reclaims all user memory pages via `vmm::free_user_pages()`.

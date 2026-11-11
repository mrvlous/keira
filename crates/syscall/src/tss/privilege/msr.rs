// SPDX-License-Identifier: GPL-2.0-only

//! Model Specific Register (MSR) configuration for fast syscall/sysret transitions.

#[cfg(all(target_arch = "x86_64", not(test)))]
extern "C" {
    fn init_syscall_msrs();
}

#[cfg(all(target_arch = "x86_64", test))]
unsafe fn init_syscall_msrs() {}

/// Configures syscall/sysret MSRs (STAR, LSTAR, FMASK) and per-cpu stacks.
///
/// # Safety
/// Writes model-specific registers (MSRs) and per-CPU kernel stack pointers directly.
pub unsafe fn configure_syscall_msrs(stack_top: usize) {
    #[cfg(target_arch = "x86_64")]
    {
        init_syscall_msrs();
        #[cfg(not(test))]
        {
            keira_arch::cpu::percpu::init_percpu(0);
            keira_arch::cpu::percpu::set_current_kernel_stack(stack_top as u64);
        }
        #[cfg(test)]
        {
            let _ = stack_top;
        }
    }

    #[cfg(target_arch = "x86")]
    {
        let _ = stack_top;
    }
}

// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2026 Moh. Ananda Firmansyah Putra <mrvlous@proton.me>
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; version 2 of the License.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 */

//! CPU Control Registers (CR0-CR4, XCR0) and Model-Specific Registers (MSR).

pub mod cr {
    #[cfg(target_os = "none")]
    use core::arch::asm;
    
    #[inline(always)]
    pub unsafe fn read_cr0() -> usize {
        #[cfg(not(target_os = "none"))]
        {
            0x8000_0001
        }
        #[cfg(target_os = "none")]
        {
            let cr0: usize;
            asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
            cr0
        }
    }
    
    #[inline(always)]
    pub unsafe fn write_cr0(val: usize) {
        #[cfg(not(target_os = "none"))]
        {
            let _ = val;
        }
        #[cfg(target_os = "none")]
        {
            asm!("mov cr0, {}", in(reg) val, options(nomem, nostack, preserves_flags));
        }
    }
    
    #[inline(always)]
    pub unsafe fn read_cr2() -> usize {
        #[cfg(not(target_os = "none"))]
        {
            0
        }
        #[cfg(target_os = "none")]
        {
            let cr2: usize;
            asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
            cr2
        }
    }
    
    #[inline(always)]
    pub unsafe fn read_cr3() -> usize {
        #[cfg(not(target_os = "none"))]
        {
            0x1000
        }
        #[cfg(target_os = "none")]
        {
            let cr3: usize;
            asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack, preserves_flags));
            cr3
        }
    }
    
    #[inline(always)]
    pub unsafe fn write_cr3(val: usize) {
        #[cfg(not(target_os = "none"))]
        {
            let _ = val;
        }
        #[cfg(target_os = "none")]
        {
            asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags));
        }
    }
    
    #[inline(always)]
    pub unsafe fn read_cr4() -> usize {
        #[cfg(not(target_os = "none"))]
        {
            0x20
        }
        #[cfg(target_os = "none")]
        {
            let cr4: usize;
            asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
            cr4
        }
    }
    
    #[inline(always)]
    pub unsafe fn write_cr4(val: usize) {
        #[cfg(not(target_os = "none"))]
        {
            let _ = val;
        }
        #[cfg(target_os = "none")]
        {
            asm!("mov cr4, {}", in(reg) val, options(nomem, nostack, preserves_flags));
        }
    }
    
    #[inline(always)]
    pub unsafe fn read_rflags() -> usize {
        #[cfg(not(target_os = "none"))]
        {
            0x202
        }
        #[cfg(all(target_os = "none", target_arch = "x86_64"))]
        {
            let rflags: usize;
            asm!("pushfq; pop {}", out(reg) rflags, options(nomem, preserves_flags));
            rflags
        }
        #[cfg(all(target_os = "none", target_arch = "x86"))]
        {
            let eflags: usize;
            asm!("pushfd; pop {}", out(reg) eflags, options(nomem, preserves_flags));
            eflags
        }
    }
}

pub use self::cr::*;

pub mod msr {
    #[cfg(target_os = "none")]
    use core::arch::asm;
    
    pub const IA32_APIC_BASE_MSR: u32 = 0x1B;
    pub const IA32_EFER_MSR: u32 = 0xC0000080;
    pub const IA32_STAR_MSR: u32 = 0xC0000081;
    pub const IA32_LSTAR_MSR: u32 = 0xC0000082;
    pub const IA32_FMASK_MSR: u32 = 0xC0000084;
    pub const IA32_FS_BASE_MSR: u32 = 0xC0000100;
    pub const IA32_GS_BASE_MSR: u32 = 0xC0000101;
    pub const IA32_KERNEL_GS_BASE_MSR: u32 = 0xC0000102;
    pub const IA32_THERM_STATUS_MSR: u32 = 0x19C;
    pub const IA32_TEMPERATURE_TARGET_MSR: u32 = 0x1A2;
    pub const AMD_HARDWARE_THERMAL_STATUS_MSR: u32 = 0xC0010064;
    pub const IA32_PERF_STATUS_MSR: u32 = 0x198;
    
    #[inline(always)]
    pub unsafe fn rdmsr(msr: u32) -> u64 {
        #[cfg(not(target_os = "none"))]
        {
            let _ = msr;
            0
        }
        #[cfg(target_os = "none")]
        {
            let low: u32;
            let high: u32;
            asm!("rdmsr", in("ecx") msr, out("eax") low, out("edx") high, options(nomem, nostack, preserves_flags));
            ((high as u64) << 32) | (low as u64)
        }
    }
    
    #[inline(always)]
    pub unsafe fn wrmsr(msr: u32, val: u64) {
        #[cfg(not(target_os = "none"))]
        {
            let _ = (msr, val);
        }
        #[cfg(target_os = "none")]
        {
            let low = val as u32;
            let high = (val >> 32) as u32;
            asm!("wrmsr", in("ecx") msr, in("eax") low, in("edx") high, options(nomem, nostack, preserves_flags));
        }
    }
}

pub use self::msr::*;

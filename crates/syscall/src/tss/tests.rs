// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for Task State Segment (TSS) initialization and stack manipulation.

#[cfg(test)]
pub mod test {
    use crate::tss::*;

    #[test]
    fn test_tss_size_and_fields() {
        let size = core::mem::size_of::<TaskStateSegment>();
        #[cfg(target_arch = "x86_64")]
        assert_eq!(size, 104);
        #[cfg(target_arch = "x86")]
        assert_eq!(size, 104);
    }

    #[test]
    fn test_set_kernel_stack() {
        let test_stack = 0xFFFF_8000_1234_5000usize;
        unsafe {
            set_kernel_stack(test_stack);
            #[cfg(target_arch = "x86_64")]
            {
                let rsp0 = TSS.rsp0;
                assert_eq!(rsp0, test_stack as u64);
            }
            #[cfg(target_arch = "x86")]
            {
                let esp0 = TSS.esp0;
                assert_eq!(esp0, test_stack as u32);
            }
        }
    }

    #[test]
    fn test_init_user_mode_safe_call() {
        unsafe {
            init_user_mode();
            assert!(get_boot_kernel_stack() > 0);
        }
    }
}

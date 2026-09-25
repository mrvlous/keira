// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! Unit tests for CPU exception decoding, signal resolution, and frame layout.

#[cfg(test)]
pub mod test {
    use crate::exception::*;
    use keira_task::signal::*;

    #[test]
    fn test_exception_vector_to_signal() {
        assert_eq!(exception_vector_to_signal(0), SIGFPE);
        assert_eq!(exception_vector_to_signal(4), SIGFPE);
        assert_eq!(exception_vector_to_signal(6), SIGILL);
        assert_eq!(exception_vector_to_signal(11), SIGBUS);
        assert_eq!(exception_vector_to_signal(12), SIGBUS);
        assert_eq!(exception_vector_to_signal(13), SIGSEGV);
        assert_eq!(exception_vector_to_signal(14), SIGSEGV);
        assert_eq!(exception_vector_to_signal(17), SIGBUS);
        assert_eq!(exception_vector_to_signal(99), SIGSEGV);
    }

    #[test]
    fn test_exception_and_signal_names() {
        assert_eq!(exception_name(0), "Division by Zero (#DE)");
        assert_eq!(exception_name(6), "Invalid Opcode (#UD)");
        assert_eq!(exception_name(13), "General Protection Fault (#GP)");
        assert_eq!(exception_name(14), "Page Fault (#PF)");
        assert_eq!(exception_name(128), "Unknown Exception");

        assert_eq!(signal_name(SIGSEGV), "SIGSEGV");
        assert_eq!(signal_name(SIGKILL), "SIGKILL");
        assert_eq!(signal_name(SIGILL), "SIGILL");
        assert_eq!(signal_name(SIGTERM), "SIGTERM");
        assert_eq!(signal_name(99), "UNKNOWN");
    }

    #[test]
    fn test_exception_frame_layout() {
        #[cfg(target_arch = "x86_64")]
        {
            let size = core::mem::size_of::<ExceptionStackFrame>();
            // 22 fields of 8 bytes = 176 bytes
            assert_eq!(size, 176);
        }
        #[cfg(target_arch = "x86")]
        {
            let size = core::mem::size_of::<ExceptionStackFrame>();
            assert_eq!(size, 60);
        }
    }

    #[test]
    fn test_exception_telemetry_counters() {
        let count = get_cpu_exception_count();
        let _ = count;
        assert_eq!(
            TOTAL_CPU_EXCEPTIONS.load(core::sync::atomic::Ordering::Relaxed) as u64,
            count
        );
    }
}

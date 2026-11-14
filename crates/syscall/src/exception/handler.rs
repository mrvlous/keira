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

//! Rust exception dispatcher for CPU exceptions, signal delivery and core dump generation.

pub mod signals {
    pub fn exception_vector_to_signal(vector: u64) -> u32 {
        match vector {
            0 => keira_task::signal::SIGFPE,
            4 => keira_task::signal::SIGFPE,
            5 => keira_task::signal::SIGSEGV,
            6 => keira_task::signal::SIGILL,
            7 => keira_task::signal::SIGFPE,
            11 => keira_task::signal::SIGBUS,
            12 => keira_task::signal::SIGBUS,
            13 => keira_task::signal::SIGSEGV,
            14 => keira_task::signal::SIGSEGV,
            16 => keira_task::signal::SIGFPE,
            17 => keira_task::signal::SIGBUS,
            19 => keira_task::signal::SIGFPE,
            _ => keira_task::signal::SIGSEGV,
        }
    }

    pub fn exception_name(vector: u64) -> &'static str {
        match vector {
            0 => "Division by Zero (#DE)",
            1 => "Debug Exception (#DB)",
            2 => "Non-Maskable Interrupt (NMI)",
            3 => "Breakpoint (#BP)",
            4 => "Overflow (#OF)",
            5 => "Bound Range Exceeded (#BR)",
            6 => "Invalid Opcode (#UD)",
            7 => "Device Not Available (#NM)",
            8 => "Double Fault (#DF)",
            10 => "Invalid TSS (#TS)",
            11 => "Segment Not Present (#NP)",
            12 => "Stack-Segment Fault (#SS)",
            13 => "General Protection Fault (#GP)",
            14 => "Page Fault (#PF)",
            16 => "x87 Floating-Point Exception (#MF)",
            17 => "Alignment Check (#AC)",
            18 => "Machine Check (#MC)",
            19 => "SIMD Floating-Point Exception (#XM)",
            _ => "Unknown Exception",
        }
    }

    pub fn signal_name(sig: u32) -> &'static str {
        match sig {
            1 => "SIGHUP",
            2 => "SIGINT",
            3 => "SIGQUIT",
            4 => "SIGILL",
            5 => "SIGTRAP",
            6 => "SIGABRT",
            7 => "SIGBUS",
            8 => "SIGFPE",
            9 => "SIGKILL",
            10 => "SIGUSR1",
            11 => "SIGSEGV",
            12 => "SIGUSR2",
            13 => "SIGPIPE",
            14 => "SIGALRM",
            15 => "SIGTERM",
            _ => "UNKNOWN",
        }
    }
}

pub mod dump {
    use crate::exception::handler::signals::{exception_name, signal_name};

    pub struct DumpWriter<'a> {
        buf: &'a mut [u8],
        offset: usize,
    }

    impl<'a> DumpWriter<'a> {
        pub fn new(buf: &'a mut [u8]) -> Self {
            Self { buf, offset: 0 }
        }

        pub fn as_bytes(&self) -> &[u8] {
            &self.buf[..self.offset]
        }
    }

    impl<'a> core::fmt::Write for DumpWriter<'a> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let bytes = s.as_bytes();
            let avail = self.buf.len().saturating_sub(self.offset);
            let to_write = bytes.len().min(avail);
            self.buf[self.offset..self.offset + to_write].copy_from_slice(&bytes[..to_write]);
            self.offset += to_write;
            Ok(())
        }
    }

    /// Generates a structured core dump payload and saves it into the root VFS at `/var/log/core_<PID>.dmp`.
    #[allow(clippy::too_many_arguments)]
    pub fn write_core_dump(
        pid: usize,
        task_name: &str,
        sig: u32,
        vector: u64,
        error_code: u64,
        rip: u64,
        rsp: u64,
        rbp: u64,
        rflags: u64,
        cr2: u64,
    ) {
        let mut dump_buf = [0u8; 1024];
        let mut writer = DumpWriter::new(&mut dump_buf);
        use core::fmt::Write;
        let _ = core::write!(
            writer,
            "=== KEIRA CORE DUMP ===\n\
         PID: {}\n\
         Name: {}\n\
         Signal: {} ({})\n\
         Vector: {} ({})\n\
         Error Code: 0x{:X}\n\
         RIP: 0x{:X}\n\
         RSP: 0x{:X}\n\
         RBP: 0x{:X}\n\
         RFLAGS: 0x{:X}\n\
         CR2: 0x{:X}\n\
         Status: TERMINATED BY SIGNAL\n",
            pid,
            task_name,
            sig,
            signal_name(sig),
            vector,
            exception_name(vector),
            error_code,
            rip,
            rsp,
            rbp,
            rflags,
            cr2
        );

        let mut path_buf = [0u8; 32];
        let mut p_writer = DumpWriter::new(&mut path_buf);
        let _ = core::write!(p_writer, "/var/log/core_{}.dmp", pid);
        if let Ok(path_str) = core::str::from_utf8(p_writer.as_bytes()) {
            let _ = keira_fs::vfs::create_file(path_str);
            let _ = keira_fs::vfs::write_file(path_str, writer.as_bytes());
        }
    }
}

pub mod panic {
    use keira_arch::debug::unwind::unwind_from_frame;
    use keira_io::serial;
    use keira_io::vga;

    use crate::exception::frame::ExceptionStackFrame;

    pub fn print_hex(val: u64) {
        let hex_chars = b"0123456789ABCDEF";
        let mut buf = [0u8; 16];
        for i in 0..16 {
            buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
        }
        if let Ok(s) = core::str::from_utf8(&buf) {
            vga::print_str(s);
        }
    }

    pub fn print_hex_serial(val: u64) {
        let hex_chars = b"0123456789ABCDEF";
        let mut buf = [0u8; 16];
        for i in 0..16 {
            buf[15 - i] = hex_chars[((val >> (i * 4)) & 0xF) as usize];
        }
        if let Ok(s) = core::str::from_utf8(&buf) {
            serial::print_str(s);
        }
    }

    pub fn print_decimal_serial(val: u64) {
        if val == 0 {
            serial::print_str("0");
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 0;
        let mut temp = val;
        while temp > 0 {
            buf[i] = b'0' + (temp % 10) as u8;
            temp /= 10;
            i += 1;
        }
        for idx in 0..i {
            let char_buf = [buf[i - 1 - idx]];
            if let Ok(s) = core::str::from_utf8(&char_buf) {
                serial::print_str(s);
            }
        }
    }

    /// Emits kernel panic diagnostic messages to both VGA and Serial, then halts execution.
    ///
    /// # Safety
    /// Dereferences the raw `frame_ptr` exception frame pointer and halts CPU execution indefinitely.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn panic_exception_dump(
        vector: u64,
        error_code: u64,
        frame_ptr: *const ExceptionStackFrame,
        rip: u64,
        rsp: u64,
        rbp: u64,
        cs: u64,
        ss: u64,
        rflags: u64,
        rax: u64,
        rbx: u64,
        rcx: u64,
        rdx: u64,
        rsi: u64,
        rdi: u64,
    ) -> ! {
        #[cfg(target_arch = "x86_64")]
        let frame = &*frame_ptr;
        #[cfg(not(target_arch = "x86_64"))]
        let _ = frame_ptr;

        vga::set_color(vga::Color::LightRed, vga::Color::Black);
        vga::print_str("\n*** KERNEL PANIC ***\n");
        vga::print_str("Unhandled CPU Exception: ");
        match vector {
            0 => vga::print_str("Divide Error (#DE)"),
            1 => vga::print_str("Debug (#DB)"),
            2 => vga::print_str("Non-Maskable Interrupt (NMI)"),
            3 => vga::print_str("Breakpoint (#BP)"),
            4 => vga::print_str("Overflow (#OF)"),
            5 => vga::print_str("Bound Range Exceeded (#BR)"),
            6 => vga::print_str("Invalid Opcode (#UD)"),
            7 => vga::print_str("Device Not Available (#NM)"),
            8 => vga::print_str("Double Fault (#DF)"),
            10 => vga::print_str("Invalid TSS (#TS)"),
            11 => vga::print_str("Segment Not Present (#NP)"),
            12 => vga::print_str("Stack-Segment Fault (#SS)"),
            13 => vga::print_str("General Protection Fault (#GP)"),
            14 => {
                vga::print_str("Page Fault (#PF)");
                #[cfg(not(test))]
                let cr2 = keira_arch::cpu::read_cr2() as u64;
                #[cfg(test)]
                let cr2 = 0u64;
                vga::print_str("\nFaulting Virtual Address (CR2): 0x");
                print_hex(cr2);
            }
            16 => vga::print_str("x87 Floating-Point Exception (#MF)"),
            17 => vga::print_str("Alignment Check (#AC)"),
            18 => vga::print_str("Machine Check (#MC)"),
            19 => vga::print_str("SIMD Floating-Point Exception (#XM)"),
            20 => vga::print_str("Virtualization Exception (#VE)"),
            21 => vga::print_str("Control Protection Exception (#CP)"),
            v => {
                vga::print_str("Reserved/Unknown Vector (");
                vga::print_u64(v);
                vga::print_str(")");
            }
        }
        vga::print_str("\n");

        vga::print_str("Error Code: 0x");
        print_hex(error_code);
        vga::print_str("\n");

        vga::print_str("\nRegister Dump:\n");
        vga::print_str("  RIP: 0x");
        print_hex(rip);
        vga::print_str("   RSP: 0x");
        print_hex(rsp);
        vga::print_str("\n");
        vga::print_str("  CS:  0x");
        print_hex(cs);
        vga::print_str("   SS:  0x");
        print_hex(ss);
        vga::print_str("   RFLAGS: 0x");
        print_hex(rflags);
        vga::print_str("\n");
        vga::print_str("  RAX: 0x");
        print_hex(rax);
        vga::print_str("   RBX: 0x");
        print_hex(rbx);
        vga::print_str("\n");
        vga::print_str("  RCX: 0x");
        print_hex(rcx);
        vga::print_str("   RDX: 0x");
        print_hex(rdx);
        vga::print_str("\n");
        vga::print_str("  RSI: 0x");
        print_hex(rsi);
        vga::print_str("   RDI: 0x");
        print_hex(rdi);
        vga::print_str("\n");
        vga::print_str("  RBP: 0x");
        print_hex(rbp);
        #[cfg(target_arch = "x86_64")]
        {
            vga::print_str("   R8:  0x");
            print_hex(frame.r8);
            vga::print_str("\n");
            vga::print_str("  R9:  0x");
            print_hex(frame.r9);
            vga::print_str("   R10: 0x");
            print_hex(frame.r10);
            vga::print_str("\n");
            vga::print_str("  R11: 0x");
            print_hex(frame.r11);
            vga::print_str("   R12: 0x");
            print_hex(frame.r12);
            vga::print_str("\n");
            vga::print_str("  R13: 0x");
            print_hex(frame.r13);
            vga::print_str("   R14: 0x");
            print_hex(frame.r14);
            vga::print_str("   R15: 0x");
            print_hex(frame.r15);
        }
        vga::print_str("\n");
        vga::print_str("\nSystem halted. Please reboot/reset your computer.\n");

        serial::print_str("\n*** KERNEL PANIC ***\n");
        serial::print_str("Unhandled exception vector: ");
        print_decimal_serial(vector);
        serial::print_str("\nRIP: 0x");
        print_hex_serial(rip);
        serial::print_str("\nRSP: 0x");
        print_hex_serial(rsp);
        serial::print_str("\nError Code: 0x");
        print_hex_serial(error_code);
        serial::print_str("\n");

        unwind_from_frame(rbp, rip);

        #[cfg(not(test))]
        loop {
            core::arch::asm!("cli; hlt");
        }
        #[cfg(test)]
        panic!("Unhandled kernel exception occurred");
    }
}

pub mod dispatch {
    use keira_io::serial;
    use keira_io::vga;
    use keira_task::scheduler::{exit_current, CURRENT_TASK_IDX, TASKS};

    use crate::exception::frame::ExceptionStackFrame;
    use crate::exception::handler::dump::write_core_dump;
    use crate::exception::handler::panic::{
        panic_exception_dump, print_decimal_serial, print_hex, print_hex_serial,
    };
    use crate::exception::handler::signals::{
        exception_name, exception_vector_to_signal, signal_name,
    };
    use core::sync::atomic::{AtomicUsize, Ordering};

    /// Total hardware exceptions intercepted by the exception dispatcher.
    pub static TOTAL_CPU_EXCEPTIONS: AtomicUsize = AtomicUsize::new(0);

    /// Retrieves total CPU exceptions trapped by the kernel.
    pub fn get_cpu_exception_count() -> u64 {
        TOTAL_CPU_EXCEPTIONS.load(Ordering::Relaxed) as u64
    }

    #[cfg(not(test))]
    extern "C" {
        fn abort_user_mode() -> !;
    }

    #[cfg(test)]
    unsafe fn abort_user_mode() -> ! {
        panic!("abort_user_mode called");
    }

    /// Central CPU exception dispatcher invoked by low-level assembly ISR handlers.
    ///
    /// # Safety
    /// Dereferences the raw `frame_ptr` pushed onto the CPU interrupt stack frame.
    #[no_mangle]
    pub unsafe extern "C" fn exception_dispatcher(frame_ptr: *const ExceptionStackFrame) {
        TOTAL_CPU_EXCEPTIONS.fetch_add(1, Ordering::Relaxed);
        let frame = &*frame_ptr;

        #[cfg(target_arch = "x86_64")]
        let (vector, error_code, rip, rsp, cs, ss, rflags, rax, rbx, rcx, rdx, rsi, rdi, rbp) = (
            frame.vector,
            frame.error_code,
            frame.rip,
            frame.rsp,
            frame.cs,
            frame.ss,
            frame.rflags,
            frame.rax,
            frame.rbx,
            frame.rcx,
            frame.rdx,
            frame.rsi,
            frame.rdi,
            frame.rbp,
        );

        #[cfg(target_arch = "x86")]
        let (vector, error_code, rip, rsp, cs, ss, rflags, rax, rbx, rcx, rdx, rsi, rdi, rbp) = (
            frame.vector as u64,
            frame.error_code as u64,
            frame.eip as u64,
            frame.user_esp as u64,
            frame.cs as u64,
            frame.user_ss as u64,
            frame.eflags as u64,
            frame.eax as u64,
            frame.ebx as u64,
            frame.ecx as u64,
            frame.edx as u64,
            frame.esi as u64,
            frame.edi as u64,
            frame.ebp as u64,
        );

        // Handle Hardware Debug Exception (#DB) triggered by DR0..DR3 watchpoints
        if vector == 1 {
            #[cfg(not(test))]
            let dr6 = keira_arch::debug::read_dr6();
            #[cfg(test)]
            let dr6 = 0u64;

            let mut handled = false;
            for slot in 0..4 {
                if (dr6 & (1 << slot)) != 0 {
                    serial::print_str("[HW WATCHPOINT] Slot ");
                    print_decimal_serial(slot as u64);
                    serial::print_str(" triggered at RIP: 0x");
                    print_hex_serial(rip);
                    serial::print_str("\n");
                    handled = true;
                }
            }
            if handled {
                #[cfg(not(test))]
                keira_arch::debug::write_dr6(dr6 & !0xF);
                return;
            }
        }

        // 1. Attempt to resolve user address Page Fault on-demand (Demand Paging / Stack Auto-Growth / Heap)
        if vector == 14 {
            #[cfg(not(test))]
            let cr2 = keira_arch::cpu::read_cr2() as u64;
            #[cfg(test)]
            let cr2 = 0u64;

            if (0x1000..=crate::user_copy::USER_MAX_ADDR).contains(&cr2) {
                if keira_mem::vmm::handle_page_fault(cr2, error_code, rsp) {
                    return;
                }

                // Demand Paging for task heap (program_break)
                if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                    if cr2 >= t.program_break_start && cr2 < t.program_break {
                        let fault_page = cr2 & !(keira_mem::pmm::PAGE_SIZE - 1);
                        if let Some(frame) = keira_mem::pmm::alloc_frame() {
                            core::ptr::write_bytes(
                                frame as *mut u8,
                                0,
                                keira_mem::pmm::PAGE_SIZE as usize,
                            );
                            #[cfg(target_arch = "x86_64")]
                            let flags = keira_mem::vmm::PAGE_PRESENT
                                | keira_mem::vmm::PAGE_WRITABLE
                                | keira_mem::vmm::PAGE_USER
                                | keira_mem::vmm::PAGE_NO_EXECUTE;
                            #[cfg(not(target_arch = "x86_64"))]
                            let flags = keira_mem::vmm::PAGE_PRESENT
                                | keira_mem::vmm::PAGE_WRITABLE
                                | keira_mem::vmm::PAGE_USER;
                            if keira_mem::vmm::map_page(fault_page, frame, flags).is_ok() {
                                #[cfg(not(test))]
                                keira_arch::cpu::invlpg(fault_page as usize);
                                return;
                            } else {
                                keira_mem::pmm::free_frame(frame);
                            }
                        }
                    }
                }
            }
        }

        if (cs & 3) == 3 {
            let sig = exception_vector_to_signal(vector);

            let handler = keira_task::signal::get_signal_handler(CURRENT_TASK_IDX, sig);
            if (0x10000..0x0000_8000_0000_0000).contains(&handler) {
                let already_in_handler = if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                    t.saved_sigcontext.is_some()
                } else {
                    false
                };

                if !already_in_handler {
                    #[cfg(target_arch = "x86_64")]
                    let ctx = keira_task::types::InterruptContext {
                        rip,
                        rsp,
                        rbp,
                        rflags,
                        rax,
                        rbx,
                        rcx,
                        rdx,
                        rsi,
                        rdi,
                        ..Default::default()
                    };
                    #[cfg(target_arch = "x86")]
                    let ctx = keira_task::types::InterruptContext {
                        rip: rip as u32,
                        rsp: rsp as u32,
                        rbp: rbp as u32,
                        rflags: rflags as u32,
                        rax: rax as u32,
                        rbx: rbx as u32,
                        rcx: rcx as u32,
                        rdx: rdx as u32,
                        rsi: rsi as u32,
                        rdi: rdi as u32,
                        ..Default::default()
                    };
                    keira_task::scheduler::set_saved_sigcontext(ctx);

                    let mut_frame = frame_ptr as *mut ExceptionStackFrame;
                    #[cfg(target_arch = "x86_64")]
                    {
                        (*mut_frame).rip = handler;
                        (*mut_frame).rdi = sig as u64;
                        let new_rsp = (rsp.saturating_sub(128)) & !0xF;
                        (*mut_frame).rsp = new_rsp;
                    }
                    #[cfg(target_arch = "x86")]
                    {
                        (*mut_frame).eip = handler as u32;
                        let new_esp = (rsp.saturating_sub(16)) & !0xF;
                        let stack_ptr = (new_esp as usize) as *mut u32;
                        if !stack_ptr.is_null() {
                            *stack_ptr.add(1) = sig;
                        }
                        (*mut_frame).user_esp = new_esp as u32;
                    }
                    return;
                }
            }

            let cr2 = if vector == 14 {
                #[cfg(not(test))]
                let val = keira_arch::cpu::read_cr2() as u64;
                #[cfg(test)]
                let val = 0u64;
                val
            } else {
                0
            };

            let task_name = if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                t.name
            } else {
                "unknown"
            };

            serial::print_str("[CRASH] Process PID ");
            print_decimal_serial(CURRENT_TASK_IDX as u64);
            serial::print_str(" (");
            serial::print_str(task_name);
            serial::print_str(") terminated by signal ");
            print_decimal_serial(sig as u64);
            serial::print_str(" (");
            serial::print_str(signal_name(sig));
            serial::print_str(") at RIP: 0x");
            print_hex_serial(rip);
            serial::print_str("\n");

            write_core_dump(
                CURRENT_TASK_IDX,
                task_name,
                sig,
                vector,
                error_code,
                rip,
                rsp,
                rbp,
                rflags,
                cr2,
            );

            vga::set_color(vga::Color::LightRed, vga::Color::Black);
            vga::print_str("\n*** USER PROCESS CRASHED (CORE DUMP) ***\n");
            vga::print_str("PID: ");
            vga::print_u64(CURRENT_TASK_IDX as u64);
            vga::print_str(" (");
            vga::print_str(task_name);
            vga::print_str(") | Signal: ");
            vga::print_u64(sig as u64);
            vga::print_str(" (");
            vga::print_str(signal_name(sig));
            vga::print_str(") | Exception: ");
            vga::print_str(exception_name(vector));
            vga::print_str(" (Vector ");
            vga::print_u64(vector);
            vga::print_str(")\n");
            vga::print_str("Registers: RIP=0x");
            print_hex(rip);
            vga::print_str(" RSP=0x");
            print_hex(rsp);
            vga::print_str(" RBP=0x");
            print_hex(rbp);
            vga::print_str("\n");
            if vector == 14 {
                vga::print_str("Faulting Virtual Address (CR2): 0x");
                print_hex(cr2);
                vga::print_str("\n");
            }
            vga::print_str("Core dump written to: /var/log/core_");
            vga::print_u64(CURRENT_TASK_IDX as u64);
            vga::print_str(".dmp\n");

            vga::print_str("Terminating crashed user process...\n");
            vga::set_color(vga::Color::LightGrey, vga::Color::Black);

            if CURRENT_TASK_IDX != 0 {
                exit_current(-(sig as i32));
            } else {
                abort_user_mode();
            }
        }

        panic_exception_dump(
            vector, error_code, frame_ptr, rip, rsp, rbp, cs, ss, rflags, rax, rbx, rcx, rdx, rsi,
            rdi,
        );
    }
}

pub use self::dispatch::{exception_dispatcher, get_cpu_exception_count, TOTAL_CPU_EXCEPTIONS};
pub use self::dump::{write_core_dump, DumpWriter};
pub use self::panic::{panic_exception_dump, print_decimal_serial, print_hex, print_hex_serial};
pub use self::signals::{exception_name, exception_vector_to_signal, signal_name};

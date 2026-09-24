// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! High-level exception dispatching and userland recovery routing.

use keira_io::serial;
use keira_io::vga;
use keira_task::scheduler::{exit_current, CURRENT_TASK_IDX, TASKS};

use crate::exception::frame::ExceptionStackFrame;
use crate::exception::handler::dump::write_core_dump;
use crate::exception::handler::panic::{
    panic_exception_dump, print_decimal_serial, print_hex, print_hex_serial,
};
use crate::exception::handler::signals::{exception_name, exception_vector_to_signal, signal_name};

#[cfg(not(test))]
extern "C" {
    fn abort_user_mode() -> !;
}

#[cfg(test)]
unsafe fn abort_user_mode() -> ! {
    panic!("abort_user_mode called");
}

/// Central CPU exception dispatcher invoked by low-level assembly ISR handlers.
#[no_mangle]
pub unsafe extern "C" fn exception_dispatcher(frame_ptr: *const ExceptionStackFrame) {
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

        if cr2 >= 0x1000 && cr2 <= crate::user_copy::USER_MAX_ADDR {
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
        if handler >= 0x10000 && handler < 0x0000_8000_0000_0000 {
            let already_in_handler = if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                t.saved_sigcontext.is_some()
            } else {
                false
            };

            if !already_in_handler {
                let mut ctx = keira_task::types::InterruptContext::default();
                ctx.rip = rip;
                ctx.rsp = rsp;
                ctx.rbp = rbp;
                ctx.rflags = rflags;
                ctx.rax = rax;
                ctx.rbx = rbx;
                ctx.rcx = rcx;
                ctx.rdx = rdx;
                ctx.rsi = rsi;
                ctx.rdi = rdi;
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
        vga::print_str("Core dump written to: /data/log/core_");
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
        vector, error_code, frame_ptr, rip, rsp, rbp, cs, ss, rflags, rax, rbx, rcx, rdx, rsi, rdi,
    );
}

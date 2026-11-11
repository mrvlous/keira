// SPDX-License-Identifier: GPL-2.0-only

//! Implementation of the 'vmainfo' supervisor virtual memory area shell command.

use crate::args::CliArgs;
use keira_io::vga;
use keira_mem::{
    get_vmas_for_pml4, total_active_vmas, MAP_SHARED, MAX_VMAS, PROT_EXEC, PROT_READ, PROT_WRITE,
};
use keira_task::scheduler::{CURRENT_TASK_IDX, TASKS};
use keira_task::types::MAX_TASKS;

fn print_padded(s: &str, width: usize) {
    vga::print_str(s);
    let len = s.len();
    for _ in len..width {
        vga::print_str(" ");
    }
}

fn print_vma_line(start: u64, end: u64, prot_str: &str, flags_str: &str, path: &str) {
    vga::print_str("  ");
    vga::print_hex(start);
    vga::print_str(" ");
    vga::print_hex(end);
    vga::print_str(" ");

    let size_kb = (end.saturating_sub(start)) / 1024;
    let mut digits = 1;
    let mut temp = size_kb;
    while temp >= 10 {
        digits += 1;
        temp /= 10;
    }
    for _ in digits..6 {
        vga::print_str(" ");
    }
    vga::print_u64(size_kb);
    vga::print_str(" KB ");
    print_padded(prot_str, 5);
    vga::print_str(" ");
    print_padded(flags_str, 7);
    vga::print_str(" ");
    vga::print_str(path);
    vga::print_str("\n");
}

fn inspect_task_vma(pid: usize, fg: vga::Color, bg: vga::Color) {
    unsafe {
        if pid >= MAX_TASKS || TASKS[pid].is_none() {
            vga::set_color(vga::Color::LightRed, bg);
            vga::print_str("Error: Process PID ");
            vga::print_u64(pid as u64);
            vga::print_str(" is not active\n");
            vga::set_color(vga::Color::LightGrey, bg);
            return;
        }

        let task = match TASKS[pid].as_ref() {
            Some(t) => t,
            None => return,
        };

        vga::set_color(fg, bg);
        vga::print_str("Process Virtual Memory Areas (PID ");
        vga::print_u64(task.id as u64);
        vga::print_str(": ");
        vga::print_str(task.name);
        vga::print_str("):\n");

        vga::set_color(vga::Color::Yellow, bg);
        vga::print_str("  START              END                SIZE      PERM  FLAGS   PATH\n");
        vga::set_color(fg, bg);

        let mut total_regions = 0usize;

        if !task.is_user {
            let stack_top = task.stack_addr + 16384;
            print_vma_line(task.stack_addr, stack_top, "rw-p", "private", "[kstack]");
            total_regions += 1;
        } else {
            // Text segment
            let text_start: u64 = 0x0040_0000;
            let text_end: u64 = text_start + 0x5000;
            print_vma_line(text_start, text_end, "r-xp", "private", task.name);
            total_regions += 1;

            // Data / BSS segment
            let data_start: u64 = text_end;
            let data_end: u64 = data_start + 0x1000;
            print_vma_line(data_start, data_end, "rw-p", "private", task.name);
            total_regions += 1;

            // Heap
            if task.program_break > task.program_break_start {
                print_vma_line(
                    task.program_break_start,
                    task.program_break,
                    "rw-p",
                    "private",
                    "[heap]",
                );
                total_regions += 1;
            }

            // Dynamic VMAs from VMA_TABLE
            if task.pml4_phys != 0 {
                let mut vmas = [keira_mem::Vma::empty(); MAX_VMAS];
                let count = get_vmas_for_pml4(task.pml4_phys, &mut vmas);
                for vma in &vmas[..count] {
                    let mut prot = [b'-'; 4];
                    if (vma.prot & PROT_READ) != 0 {
                        prot[0] = b'r';
                    }
                    if (vma.prot & PROT_WRITE) != 0 {
                        prot[1] = b'w';
                    }
                    if (vma.prot & PROT_EXEC) != 0 {
                        prot[2] = b'x';
                    }
                    prot[3] = if (vma.flags & MAP_SHARED) != 0 {
                        b's'
                    } else {
                        b'p'
                    };
                    let prot_str = core::str::from_utf8(&prot).unwrap_or("---p");
                    let flags_str = if (vma.flags & MAP_SHARED) != 0 {
                        "shared"
                    } else {
                        "private"
                    };
                    let path = vma.file_path_str().unwrap_or("[anon]");
                    print_vma_line(vma.start, vma.end, prot_str, flags_str, path);
                    total_regions += 1;
                }
            }

            // Stack
            #[cfg(target_arch = "x86")]
            let stack_top = 0x07FFF000u64;
            #[cfg(not(target_arch = "x86"))]
            let stack_top = 0x7FFFFFE00000u64.saturating_sub(task.stack_aslr_offset);
            let stack_bottom = stack_top.saturating_sub(0x20000);
            print_vma_line(stack_bottom, stack_top, "rw-p", "private", "[stack]");
            total_regions += 1;
        }

        vga::print_str("\n");
        vga::print_str("  PML4 Base    : ");
        vga::print_hex(task.pml4_phys);
        vga::print_str("\n");
        vga::print_str("  Active VMAs  : ");
        vga::print_u64(total_regions as u64);
        vga::print_str(" regions\n");
        vga::print_str("  VMA Capacity : ");
        vga::print_u64(total_active_vmas() as u64);
        vga::print_str(" / ");
        vga::print_u64(MAX_VMAS as u64);
        vga::print_str(" slots in kernel\n");
        vga::set_color(vga::Color::LightGrey, bg);
    }
}

/// Executes the 'vmainfo' / 'vma' supervisor diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);
    let bg = vga::Color::Black;
    let fg = vga::Color::White;

    if args.has_flag('h', "help") {
        vga::set_color(fg, bg);
        vga::print_str("Usage: vmainfo [-a] [-h] [pid]\n\n");
        vga::print_str(
            "Description:\n  Display Virtual Memory Area (VMA) layout, protection modes and address mappings.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str("  -a, --all      Display virtual memory layout for all active tasks\n");
        vga::print_str("  -h, --help     Show this help reference and exit\n");
        vga::print_str("  [pid]          Inspect virtual memory layout of specific process\n");
        vga::set_color(vga::Color::LightGrey, bg);
        return;
    }

    if args.has_flag('a', "all") {
        unsafe {
            for (i, slot) in TASKS.iter().enumerate() {
                if slot.is_some() {
                    inspect_task_vma(i, fg, bg);
                    vga::print_str("\n");
                }
            }
        }
        return;
    }

    let target_pid = if let Some(arg) = args.first_positional() {
        match arg.parse::<usize>() {
            Ok(p) => p,
            Err(_) => {
                vga::set_color(vga::Color::LightRed, bg);
                vga::print_str("Error: Invalid process identifier '");
                vga::print_str(arg);
                vga::print_str("'\n");
                vga::set_color(vga::Color::LightGrey, bg);
                return;
            }
        }
    } else {
        unsafe { CURRENT_TASK_IDX }
    };

    inspect_task_vma(target_pid, fg, bg);
}

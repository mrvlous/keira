// SPDX-License-Identifier: GPL-2.0-only

//! Trigger kernel stack frame unwinder backtrace.

use keira_io::vga;

#[inline(never)]
pub fn run(parts: &mut core::str::SplitWhitespace) {
    if let Some("-h") | Some("--help") = parts.next() {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: unwind [-h]\n\n");
        vga::print_str("Description:\n  Trigger kernel stack frame unwinder backtrace.\n\n");
        vga::print_str("Options:\n  -h, --help    Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_str("Kernel Unwinder & CPU Exception Telemetry:\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  Total Trapped Exceptions: ");
    vga::print_u64(keira_syscall::get_cpu_exception_count());
    vga::print_str("\n");

    #[cfg(target_os = "none")]
    keira_arch::unwind::unwind_stack();
}

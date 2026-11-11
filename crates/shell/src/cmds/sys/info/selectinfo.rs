// SPDX-License-Identifier: GPL-2.0-only

//! Implementation of the 'selectinfo' and 'select' supervisor shell commands.

use crate::args::CliArgs;
use keira_io::vga;

fn print_label_and_u64(label: &str, val: u64, suffix: &str) {
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str("  ");
    vga::print_str(label);
    let pad = 30usize.saturating_sub(label.len());
    for _ in 0..pad {
        vga::print_str(" ");
    }
    vga::print_str(": ");
    vga::set_color(vga::Color::White, vga::Color::Black);
    vga::print_u64(val);
    if !suffix.is_empty() {
        vga::print_str(" ");
        vga::print_str(suffix);
    }
    vga::print_str("\n");
}

/// Executes the 'selectinfo' / 'select' supervisor diagnostic command.
pub fn run(parts: &mut core::str::SplitWhitespace) {
    let args = CliArgs::parse(parts);

    if args.has_flag('h', "help") {
        vga::set_color(vga::Color::White, vga::Color::Black);
        vga::print_str("Usage: selectinfo [-h]\n\n");
        vga::print_str(
            "Description:\n  Display POSIX I/O multiplexing telemetry and sys_select invocation metrics.\n\n",
        );
        vga::print_str("Options:\n");
        vga::print_str("  -h, --help  Show this help message and exit\n");
        vga::set_color(vga::Color::LightGrey, vga::Color::Black);
        return;
    }

    let stats = keira_syscall::get_select_telemetry();

    vga::set_color(vga::Color::LightGreen, vga::Color::Black);
    vga::print_str("=== POSIX I/O Multiplexing Subsystem (SYS_SELECT) ===\n\n");
    vga::set_color(vga::Color::White, vga::Color::Black);

    print_label_and_u64("Total sys_select Invocations", stats.total_selects, "calls");
    print_label_and_u64(
        "Total Ready Event Signals",
        stats.total_ready_events,
        "events",
    );
    print_label_and_u64("Timeout Expirations", stats.total_timeouts, "events");
    print_label_and_u64("Bad Descriptor (EBADF)", stats.total_badf_events, "events");
    print_label_and_u64("Bitmask Capacity (FD_SETSIZE)", 1024, "descriptors / mask");

    vga::print_str("\n");
    vga::set_color(vga::Color::LightGrey, vga::Color::Black);
    vga::print_str(
        "  Supported Channels        : Anonymous Pipes, UNIX Sockets, TCP/UDP, TTY & VFS\n",
    );
    vga::print_str("  Standard Compliance       : POSIX.1-2008 / Single UNIX Specification v4\n");
}

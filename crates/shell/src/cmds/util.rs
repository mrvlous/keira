// SPDX-License-Identifier: GPL-2.0-only
//
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

//! Emergency diagnostic and help utilities.

pub mod misc {
    pub mod help {
        use keira_io::vga;

        pub fn run(_parts: &mut core::str::SplitWhitespace) {
            let bg = vga::Color::Black;
            let fg = vga::Color::White;

            vga::set_color(fg, bg);
            vga::print_str(concat!(
                "Keira Kernel Diagnostic Console (Ring 0 v",
                env!("CARGO_PKG_VERSION"),
                ")\n\n"
            ));
            vga::print_str("Diagnostics & Storage:\n");
            vga::print_str("  system         Display system summary, version and uptime\n");
            vga::print_str("  cpu, smp       Processor topology, features and SMP cores\n");
            vga::print_str("  memory, buddy  Physical memory, buddy orders and zone watermarks\n");
            vga::print_str("  slab, slabinfo Kernel slab object caches and CPU depot magazines\n");
            vga::print_str("  devices, pci   PCI hardware buses and peripheral devices\n");
            vga::print_str("  drivers        Registered kernel device drivers and bindings\n");
            vga::print_str("  lsmod, insmod  Inspect, load, query or unload kernel modules\n");
            vga::print_str("  rmmod, modinfo Manage dynamically loaded Ring 0 ELF modules\n");
            vga::print_str("  regs, irq      CPU control registers and interrupt counters\n");
            vga::print_str("  disk, mount    Block devices and VFS mount points\n");
            vga::print_str("  dcache, icache VFS dentry and inode cache telemetry\n\n");
            vga::print_str("Security & IPC:\n");
            vga::print_str("  pidinfo, procs Process hierarchy, states and PID metrics\n");
            vga::print_str("  secinfo, aslr  Hardware mitigations and ASLR entropy engine\n");
            vga::print_str("  rnginfo, rlimit Silicon entropy source and resource quotas\n");
            vga::print_str("  pipes, unixsock IPC anonymous pipes and UNIX domain sockets\n");
            vga::print_str("  vmainfo, poll  Virtual memory areas and POSIX poll tables\n");
            vga::print_str("  selectinfo     POSIX select fd_set multiplexing telemetry\n");
            vga::print_str("  waitqueue, wq  Kernel wait queues and condition synchronization\n\n");
            vga::print_str("Control & Lifecycle:\n");
            vga::print_str("  daemons, kthreads Active Ring 0 background kernel daemons\n");
            vga::print_str("  workqueue, kworker Deferred kernel workqueues and kworker pool\n");
            vga::print_str(
                "  unwind, watchpoint Stack unwinding and hardware DR0-DR3 watchpoints\n",
            );
            vga::print_str("  syslog, dmesg  Circular kernel logging buffer messages\n");
            vga::print_str("  sync, wipe     Flush dirty cache buffers and clear VGA console\n");
            vga::print_str("  sh, run <elf>  Launch Ring 3 shell or execute ELF binary\n");
            vga::print_str("  reset, power   ACPI reboot or power off system\n\n");
            vga::print_str("Type '<cmd> --help' for options. Press Tab for completion.\n");
        }
    }
    pub use self::help::*;
}

pub use self::misc::help;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_util_commands_help_invocation() {
        let mut args = "help --help".split_whitespace();
        args.next();
        help::run(&mut args);
    }
}

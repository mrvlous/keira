<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Syscall Router & Entry Routing

The master router arm dispatches the syscall number to specialized subsystem handlers:
* Filesystem handlers (`fs.rs`)
* Process & Task handlers (`proc.rs`)
* Network handlers (`net.rs`)
* IPC handlers (`ipc.rs`)
* System info handlers (`sys.rs`)

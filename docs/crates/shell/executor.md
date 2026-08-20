<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Shell Command Parser & Executor

Documentation for command execution in [`crates/shell/src/executor.rs`](../../../crates/shell/src/executor.rs).

## Features
- Splits input command line by whitespace into arguments.
- Handles user privilege verification (`please` command).
- Dispatches command token to corresponding handler in `crates/shell/src/cmds/`.

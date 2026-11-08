<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Keira Interactive Shell & Command Subsystem

The `shell` domain provides the kernel emergency supervisor console control plane and 32 built-in diagnostic commands. Standard user utilities (`ps`, `kill`, `hostname`, `dmesg`, `df`, `cat`, `ls`, `fetch`, `kcc`, `sh` and `init`) run exclusively in Ring 3 userspace (`/bin`).

---

## Shell Architecture

```mermaid
graph TD
    Shell["Interactive Shell"] --> Terminal["terminal/<br/>Prompt, Keyboard & Console Palette"]
    Shell --> Executor["executor/<br/>Command Dispatch, Autocomplete & CliArgs"]
    Shell --> Cmds["commands/<br/>32 Emergency Commands"]
```

---

## Submodule Index

| Submodule | Focus Area | Description |
| :--- | :--- | :--- |
| [`terminal/`](terminal/README.md) | Terminal UI | Prompt rendering, keyboard input and monochrome palette |
| [`executor/`](executor/README.md) | Command Execution | Argument parsing, pipe routing, autocomplete and execution dispatch |
| [`commands/`](commands/README.md) | Built-in Commands | Reference manuals for all 32 built-in emergency commands |

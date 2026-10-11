<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Shell Prompt & History Navigation

Keira implements a dual shell prompt architecture reflecting privilege separation:

1. **Ring 0 Supervisor Console**: Renders `ring0@<hostname>:<path># ` for emergency diagnostic and hardware recovery commands.
2. **Ring 3 Userspace Shell (`sh`)**: Renders `<hostname>:<path>$ ` (e.g. `keira:/$ `, `keira:/bin$ `) for unprivileged standard UNIX operations.

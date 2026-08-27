<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Interactive Fullscreen `kvi` Text Editor

This document specifies the fullscreen interactive `kvi` text editor built directly into Keira Kernel.

---

## Editor Architecture

* **Screen Buffer**: 80 $\times$ 25 character and color matrix.
* **Line Grid**: 128 editable lines $\times$ 256 characters per line.
* **Cut Buffer**: In-memory line clipboard.

---

## Keyboard Shortcuts

| Shortcut | Function | Description |
| :--- | :--- | :--- |
| `Ctrl+S` / `F2` | Save File | Commits current grid contents to storage via FAT16 VFS |
| `Ctrl+Q` / `F10` | Exit Editor | Closes editor buffer and returns to interactive shell prompt |
| `Ctrl+X` | Cut Line | Copies active line to clipboard and removes it from grid |
| `Ctrl+V` | Paste Line | Inserts clipboard line at cursor location |
| `Ctrl+F` | Find Text | Highlights search matches across the document |

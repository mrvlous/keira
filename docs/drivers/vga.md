# VGA Text Console Driver

This document details the VGA text mode driver, character cell structures, and hardware cursor programming in Keira Kernel.

## 1. VGA Text Memory Architecture
The VGA text-mode driver ([vga.c](../../drivers/vga/vga.c)) manages text rendering on an 80x25 character grid using a memory-mapped frame buffer.
*   **Physical Address**: The frame buffer is mapped at `0xB8000`.
*   **Total Capacity**: The display grid consists of 2000 character cells (80 columns × 25 rows), requiring a 4000-byte memory block.

### Character Cell Layout (16-bit Entry)
Each cell on the screen is represented by a 16-bit word:
*   **Bits 0-7**: The ASCII character byte.
*   **Bits 8-11**: The foreground text color (4 bits, 16 colors).
*   **Bits 12-15**: The background fill color (4 bits, 16 colors).

The attribute byte is compiled using the formula: `attribute = foreground | (background << 4)`.

---

## 2. Hardware Cursor Control
The blinking hardware cursor is programmed by writing to I/O ports `0x3D4` (Address Register) and `0x3D5` (Data Register).

### Positional Programming (`vga_update_cursor`)
To update the cursor to coordinate `(row, col)`:
1.  **Calculate Offset**: `position = row * 80 + col`.
2.  **Lower Byte**: Write register index `0x0F` to `0x3D4`, and write `position & 0xFF` to `0x3D5`.
3.  **Upper Byte**: Write register index `0x0E` to `0x3D4`, and write `(position >> 8) & 0xFF` to `0x3D5`.

### Cursor Enable and Scanline Adjustments
`vga_enable_cursor` configures the size and visibility of the hardware cursor:
*   Register index `0x0A` sets the starting scanline of the cursor (bits 0-4) and can disable the cursor (bit 5).
*   Register index `0x0B` sets the ending scanline of the cursor (bits 0-4).

---

## 3. Mouse Pointer Rendering in Text Mode
Since VGA text mode has no graphics capabilities, the mouse cursor is simulated by modifying character cell entries at the cursor coordinates.

### Rendering Steps
1.  **Hide Current Cursor**: If the mouse is visible, the driver reads the saved character entry and restores it at the old coordinates.
2.  **Store New Background**: The driver reads the 16-bit entry at the target coordinates `(x, y)` and saves it in `saved_mouse_entry`.
3.  **Draw Pointer Cell**:
    *   Reads the background color of the cell to avoid color clashes.
    *   If the background is light, it sets the mouse color to black; otherwise, it sets it to white.
    *   Overwrites the character to ASCII `24` (an up-arrow character) and applies the color attribute.
    *   Saves the coordinates and sets `mouse_is_visible = 1`.

---

## 4. PS/2 Keyboard Driver and Input Shortcuts
The PS/2 keyboard driver ([keyboard.c](../../drivers/keyboard/keyboard.c)) intercepts keyboard interrupts on IRQ 1 (PIC input line 1) and converts scan codes into ASCII character events dispatched to the shell.

### Keyboard Scan Code Translation
*   The driver reads scan codes from data port `0x60`.
*   Keeps track of modifier state (`shift_pressed`, `ctrl_pressed`).
*   Converts letters `a`-`z` to control characters `1`-`26` if `ctrl_pressed` is active, enabling standard terminal shortcuts.

### Global Shell Shortcuts
The shell handler ([shell.rs](../../kernel/src/shell.rs)) intercepts control codes to trigger system behaviors:
*   **`Ctrl+C` (ASCII 3)**: Aborts the current command input buffer, exits `please`/`login` password prompts, and reprints the active prompt on a new line.
*   **`Ctrl+L` (ASCII 12)**: Re-initializes the VGA console to clear the screen and moves the cursor to `(0, 0)`. Then, reprints the shell prompt and restores the user's current typed input buffer.

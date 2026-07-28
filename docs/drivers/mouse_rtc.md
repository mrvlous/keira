# Mouse and Real-Time Clock (RTC) Drivers

This document details the PS/2 Mouse and CMOS Real-Time Clock (RTC) drivers implemented in Keira Kernel.

## 1. PS/2 Mouse Driver
The mouse driver ([mouse.c](../../drivers/mouse/mouse.c)) initializes and decodes coordinate offsets from a standard PS/2 mouse.

### Controller Initialization (`mouse_init`)
1.  **Enable Port**: Sends the enable auxiliary device command (`0xA8`) to the PS/2 command port `0x64`.
2.  **Configure Controller**:
    *   Reads the PS/2 configuration byte by sending `0x20` to `0x64` and reading from `0x60`.
    *   Enables the auxiliary interrupt flag (bit 1) and clears the auxiliary disable flag (bit 5).
    *   Writes the modified configuration byte back by sending `0x60` to `0x64`, followed by the byte to `0x60`.
3.  **Command Sequence**:
    *   Sends the set defaults command (`0xF6`) to the auxiliary device.
    *   Sends the enable data reporting command (`0xF4`) to configure stream mode.
4.  **IRQ Activation**: Clears IRQ12 (auxiliary device) mask in the PIC controller.

### Interrupt Handler Packet Decoding (`mouse_handler`)
The hardware sends mouse movement updates in 3-byte packets via IRQ12:
*   **Byte 1**: Button states and sign flags.
    *   Bit 0: Left button pressed.
    *   Bit 1: Right button pressed.
    *   Bit 2: Middle button pressed.
    *   Bit 4: X sign bit (1 if offset is negative).
    *   Bit 5: Y sign bit (1 if offset is negative).
*   **Byte 2**: X displacement value.
    *   Combined with the X sign bit to calculate a signed 9-bit displacement.
*   **Byte 3**: Y displacement value.
    *   Combined with the Y sign bit to calculate a signed 9-bit displacement.

The driver decodes these deltas, scales them based on mouse sensitivity, updates the global `mouse_x` and `mouse_y` positions within the display resolution bounds, and updates the cursor on-screen.

---

## 2. CMOS Real-Time Clock (RTC) Driver
The RTC driver ([rtc.c](../../drivers/rtc/rtc.c)) queries the non-volatile CMOS memory chip to retrieve current system date and time details.

### Register Mapping and Access Methods
CMOS memory registers are accessed via I/O ports:
*   `CMOS_ADDRESS_PORT` (`0x70`): Port to select the target register index.
*   `CMOS_DATA_PORT` (`0x71`): Port to read or write data bytes.

### CMOS Read Sequence (`cmos_read`)
1.  Writes the target register index to port `0x70`.
2.  Executes `io_wait()` to ensure port stabilization.
3.  Reads the data byte from port `0x71`.

### Date/Time Retraction (`rtc_get_time`)
1.  **Update Validation**: Polls Status Register A (register `0x0A`) until the Update in Progress (UIP) bit (bit 7) is clear, ensuring the clock registers are stable.
2.  **Read Registers**:
    *   `0x00`: Seconds
    *   `0x02`: Minutes
    *   `0x04`: Hours
    *   `0x07`: Day of month
    *   `0x08`: Month
    *   `0x09`: Year
3.  **Data Decoding**: CMOS values are typically stored in Binary Coded Decimal (BCD) format. If the system uses BCD, the values are converted to binary integers using the formula: `binary = (bcd >> 4) * 10 + (bcd & 0x0F)`.

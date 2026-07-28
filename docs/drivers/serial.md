# Serial UART COM1 Driver

This document details the low-level 16550A UART serial driver used for kernel logging and diagnostic outputs.

## 1. Port Mapping and Hardware Registers
The serial driver ([serial.c](../../drivers/serial/serial.c)) configures the first serial port (COM1). The controller is mapped to the standard PC I/O port base address `0x3F8` and utilizes the following register offsets:

*   `COM1_DATA` (`0x3F8`): Data transmitter/receiver register (also Divisor LSB when DLAB=1).
*   `COM1_INT_ENABLE` (`0x3F9`): Interrupt Enable Register (also Divisor MSB when DLAB=1).
*   `COM1_FIFO_CTRL` (`0x3FA`): FIFO Control Register (write-only) for enabling and resetting buffers.
*   `COM1_LINE_CTRL` (`0x3FB`): Line Control Register (includes the Divisor Latch Access Bit - DLAB).
*   `COM1_MODEM_CTRL` (`0x3FC`): Modem Control Register.
*   `COM1_LINE_STATUS` (`0x3FD`): Line Status Register (provides transmitter and receiver status flags).

---

## 2. Initialization and Baud Rate Configuration
`serial_init` programs the UART controller for communication:
1.  **Disable Interrupts**: Write `0x00` to `COM1_INT_ENABLE` to disable UART interrupts during initialization.
2.  **Enable DLAB**: Write `0x80` to `COM1_LINE_CTRL` to enable the Divisor Latch Access Bit, allowing baud rate configuration.
3.  **Set Divisor (38400 Baud)**:
    *   Write `0x03` to `COM1_DIVISOR_LSB`.
    *   Write `0x00` to `COM1_DIVISOR_MSB`.
    *   The divisor value `3` divides the UART clock frequency (`115200 Hz`) down to `38400 Hz`.
4.  **Configure Data Format**: Write `0x03` to `COM1_LINE_CTRL` (disabling DLAB and setting 8 data bits, no parity, 1 stop bit).
5.  **Configure Buffers**: Write `0xC7` to `COM1_FIFO_CTRL` to enable FIFO buffers, clear them, and set the interrupt trigger threshold to 14 bytes.
6.  **Configure Modem**: Write `0x0B` to `COM1_MODEM_CTRL` to activate Data Terminal Ready (DTR), Request to Send (RTS), and Auxiliary Output 2 (which routes interrupts to the PIC).

---

## 3. Transmission Control
Character transmission utilizes polling of the Line Status Register (LSR).

### Transmitter Status Check (`serial_is_tx_ready`)
Before sending a byte, the driver reads `COM1_LINE_STATUS` and checks bit 5 (`LSR_TX_EMPTY` / value `0x20`). This bit is set to 1 by the hardware when the transmitter holding register is empty and ready to accept a new character.

### Writing Characters (`serial_putchar`)
*   If the input character is a newline (`\n`), the driver waits for the transmitter to be empty, sends a carriage return (`\r`), waits again, and then transmits the newline (`\n`) to ensure correct line termination on serial terminals.
*   Otherwise, it polls until `serial_is_tx_ready` returns true, then writes the character byte to the `COM1_DATA` port.

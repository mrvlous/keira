# Sound Programming Drivers

This document details the PC Speaker and Intel High Definition Audio (HDA) drivers implemented in Keira Kernel.

## 1. PC Speaker Driver (PIT Channel 2)
The PC Speaker driver ([sound.c](../../drivers/sound/sound.c)) generates tones by programming Channel 2 of the Programmable Interval Timer (PIT) to output a square-wave signal.

### Hardware Architecture
*   **PIT Mode Register** (`0x43`): Receives command bytes defining the timer channel, access mode, and operating mode.
*   **PIT Channel 2 Data Port** (`0x42`): Receives the 16-bit frequency divisor.
*   **System Control Port B** (`0x61`): Controls the gating of the PIT output to the PC Speaker.

### Programming Sequence (`sound_play`)
1.  **Calculate Divisor**: Calculates `divisor = PIT_BASE_FREQ / frequency` where `PIT_BASE_FREQ` is `1193182 Hz`.
2.  **Configure Channel**: Writes `0xB6` (Channel 2, access LSB/MSB, mode 3 - square wave, 16-bit binary) to the PIT Mode Register `0x43`.
3.  **Load Divisor**: Writes the lower byte (`divisor & 0xFF`), then the upper byte (`(divisor >> 8) & 0xFF`) to the Channel 2 Data Port `0x42`.
4.  **Activate Speaker**: Reads the value of System Control Port B `0x61`, sets bits 0 (enable PIT Channel 2 gate) and 1 (enable PC Speaker data line), and writes the modified value back.

### Stopping Sound (`sound_stop`)
Tones are stopped by clearing bits 0 and 1 of System Control Port B `0x61`, disabling output routing to the speaker.

---

## 2. Intel High Definition Audio (HDA)
The HDA driver ([hda.c](../../drivers/sound/hda.c)) manages PCI sound hardware using memory-mapped I/O (MMIO) registers and DMA transfers.

### Initialization Sequence
1.  **Global Controller Reset**: Sets the CRST bit (bit 0) of the Global Control Register (GCTL) to 0, waits for the hardware to clear the bit, delays, then sets CRST back to 1 to bring the controller out of reset.
2.  **Verb Transmission**: Commands are sent to the HDA codec by writing the verb command register (IC) and monitoring the Immediate Command Status register (ICS) and Immediate Response register (IR).
3.  **Codec Gating**: Verb sequences configure the DAC widget pin nodes to route audio to the output jacks.

### DMA Playback Setup (`hda_start_tone`)
1.  **Double Buffering**: Prepares two 4096-byte page buffers filled with synthesized stereo PCM square-wave frames.
2.  **BDL Setup**: Prepares a Buffer Descriptor List (BDL) containing two `hda_bdl_entry` structures pointing to the physical addresses of the page buffers.
3.  **Stream Programming**:
    *   Stops the output stream descriptor by clearing the run bit in `SD_CTL`.
    *   Writes the physical address of the BDL to the Stream Base Address registers (`SD_BDPL` and `SD_BDPU`).
    *   Sets the Cyclic Buffer Length (`SD_CBL` to 8192) and Last Valid Index (`SD_LVI` to 1).
    *   Programs the audio format (48kHz, 16-bit stereo) in `SD_FMTS`.
    *   Sets the stream tag to 1 and starts the stream by setting the RUN bit in `SD_CTL`.

# Intel High Definition Audio (HDA) DSP & WAV PCM Streaming Engine

This document details the Intel High Definition Audio (HDA) DSP, RIFF WAV PCM header parser, DMA stream ring buffers, and volume gain controls in Keira Kernel.

## 1. Intel HDA Subsystem Architecture
The Intel HDA DSP engine ([audio.rs](../../kernel/src/io/audio.rs)) manages audio hardware initialization, DMA ring buffer allocation, and master volume control (**Syscall 71: `sys_audio_dsp`**).

*   **PCI MMIO Base Address**: BAR0 physical MMIO register space.
*   **Sample Rate Support**: 44.1 kHz, 48.0 kHz, 96.0 kHz PCM 16-bit / 24-bit stereo audio channels.
*   **DMA Engine**: Direct Memory Access stream descriptors with circular ring buffer.

---

## 2. RIFF WAV Audio Header Structure
Keira Kernel parses 44-byte RIFF PCM WAV file headers directly from FAT16 disk storage:

```rust
#[repr(C, packed)]
pub struct WavHeader {
    pub riff_id: [u8; 4],        // "RIFF"
    pub chunk_size: u32,
    pub wave_id: [u8; 4],        // "WAVE"
    pub fmt_id: [u8; 4],         // "fmt "
    pub fmt_chunk_size: u32,     // 16 for PCM
    pub audio_format: u16,       // 1 for uncompressed PCM
    pub num_channels: u16,       // 1 = Mono, 2 = Stereo
    pub sample_rate: u32,        // e.g. 44100 or 48000 Hz
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,    // 8, 16, or 24 bits
    pub data_id: [u8; 4],        // "data"
    pub data_size: u32,
}
```

---

## 3. Shell Commands
*   **`aplay <file.wav>`**: Parses RIFF WAV header and streams PCM audio samples to the HDA controller.
*   **`alsamixer [volume 0..100]`**: Displays interactive audio control panel and adjusts master volume gain.

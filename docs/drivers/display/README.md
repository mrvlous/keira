<!-- SPDX-License-Identifier: GPL-2.0-only -->

# Video & Display Drivers

This directory details video display adapters, text-mode consoles, and linear framebuffer graphics rendering in Keira Kernel.

---

## Display Subsystem Architecture

```mermaid
graph TD
    KernelInit["Kernel Initialization"] --> DetectDisplay{"Multiboot2 Tag 8 Linear Framebuffer?"}
    DetectDisplay -->|Available| FBInit["Init VBE Linear Framebuffer (1024x768x32bpp)"]
    DetectDisplay -->|None| VGAInit["Init Hardware VGA Text Mode (80x25 @ 0xB8000)"]
    FBInit --> FontEngine["Bitmap Font Renderer (8x16 PSF Font)"]
    FontEngine --> GUI["VGA/FB Unified Console API (vga::putchar / print_str)"]
    VGAInit --> GUI
    GUI --> Terminal["Shell Terminal Screen (tty1)"]
```

---

## Display Driver Index

| Document | Display Architecture | Description |
| :--- | :--- | :--- |
| [`vga.md`](vga.md) | VGA Text Buffer | Legacy 80x25 text-mode console mapped at physical address `0xB8000` |
| [`framebuffer.md`](framebuffer.md) | VBE Linear Framebuffer | High-resolution 32bpp ARGB framebuffer graphics and bitmap font engine |

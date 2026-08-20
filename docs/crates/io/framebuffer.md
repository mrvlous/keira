<!-- SPDX-License-Identifier: GPL-2.0-only -->

# VBE Linear Framebuffer & Graphics

Documentation for graphical output in [`crates/io/src/framebuffer/`](../../../crates/io/src/framebuffer).

## Features
- **Linear Framebuffer (LFB)**: Automatically mapped via Multiboot2 framebuffer tags (e.g. 1280x800x32bpp or 1024x768x32bpp).
- **Embedded Font Renderer**: 8x16 bitmap font glyph rendering (`draw_char`, `draw_string`).
- **Hardware/Software Mouse Cursor**: Overlays a transparent arrow cursor with background save/restore.

// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Operating System Kernel
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

#![no_std]

//! Hardware I/O drivers (VGA/FrameBuffer, Serial, PCI/PCIe, Storage, USB, Audio, and TTY).

pub mod bus;
pub mod framebuffer;
pub mod serial;
pub mod sound;
pub mod storage;
pub mod tty;
pub mod usb;
pub mod vga;

pub use bus::pci::{
    self, init as pci_init, pci_class_to_str, pci_read_config_u32, pci_write_config_u32, PciDevice,
    PCI_DEVICES, PCI_DEVICE_COUNT,
};
pub use bus::pcie::{
    self, enable_msi, init as pcie_init, read_config_u32 as pcie_read_config_u32, PCIE_ECAM_BASE,
    PCIE_INITIALIZED,
};
pub use framebuffer::lfb::{
    draw_char, draw_mouse_cursor, draw_pixel, draw_rect, draw_string, fill_screen,
    set_resolution as fb_set_resolution, FB_ACTIVE, FB_ADDR, FB_BPP, FB_HEIGHT, FB_PITCH, FB_WIDTH,
};
pub use serial::uart::{
    print as serial_print, print_hex as serial_print_hex, print_str as serial_print_str,
    print_u64 as serial_print_u64, putchar as serial_putchar,
};
pub use sound::hda::{
    init as hda_init, play_tone as hda_play_tone, stop as hda_stop, HDA_INITIALIZED, HDA_PCI_FOUND,
};
pub use sound::speaker::{play_note, play_sound, sleep_ms, stop_sound};
pub use storage::ahci::{self, flush_dma_cache, init as ahci_init, AhciBlockDevice, AHCI_DEVICE};
pub use storage::block::{
    self, for_each_device, get_device, get_mounted_device, mount_device, register_device,
    BlockDevice,
};
pub use storage::ide::{
    self, identify as ide_identify, read_sector as ide_read_sector,
    write_sector as ide_write_sector, IdeBlockDevice, IDE_DEVICE,
};
pub use storage::nvme::{self, init as nvme_init, NvmeController, NVME_CONTROLLER};
pub use storage::ramdisk::{
    self, create_ramdisk, free_current_ramdisk, RamBlockDevice, RAM_DEVICE,
};
pub use tty::term::{get_active_tty, switch_tty, ACTIVE_TTY};
pub use usb::host::{
    init as usb_init, UsbControllerInfo, USB_CONTROLLERS, USB_CONTROLLER_COUNT, USB_INITIALIZED,
};
pub use usb::storage::{
    build_scsi_inquiry_cbw, build_scsi_read_capacity_cbw, eject_usb_storage, mount_usb_storage,
    sys_usb_device, CommandBlockWrapper, CommandStatusWrapper, USB_CMD_EJECT, USB_CMD_MOUNT,
    USB_CMD_SCAN, USB_CMD_STATUS, USB_STORAGE_MOUNTED,
};
pub use vga::color::Color as VgaColor;
pub use vga::console::{
    backspace as vga_backspace, clear_line_from as vga_clear_line_from,
    get_cursor_col as vga_get_cursor_col, get_cursor_row as vga_get_cursor_row,
    handle_timer_tick as vga_handle_timer_tick, init as vga_init_screen, print as vga_print,
    print_boot_log, print_hex as vga_print_hex, print_str as vga_print_str,
    print_u64 as vga_print_u64, putchar as vga_putchar, set_color as vga_set_color,
    set_cursor_pos as vga_set_cursor_pos, ACTIVE_BG_COLOR, ACTIVE_FG_COLOR, FRAMEBUFFER_ADDR,
    FRAMEBUFFER_BPP, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_MAPPED, FRAMEBUFFER_PITCH, FRAMEBUFFER_WIDTH,
    PIPE_ACTIVE, PIPE_BUFFER, PIPE_LEN, PIPE_READ_INDEX, REDIRECT_BUFFER, REDIRECT_LEN,
    REDIRECT_TO_FILE, VGA_BUSY,
};

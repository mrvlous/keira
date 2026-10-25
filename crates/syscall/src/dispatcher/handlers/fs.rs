// SPDX-License-Identifier: GPL-2.0-only
//
// Keira Kernel - Freestanding Kernel from Scratch
// Copyright (C) 2026 Moh. Ananda Firmansyah Putra
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; version 2 of the License.

//! File system and descriptor I/O system call handlers.

use keira_fs::vfs::{
    check_access, create_file, exists, get_file_size, get_metadata, is_dir, read_file,
    set_ownership, set_permissions, write_file, R_OK, W_OK,
};
use keira_io::vga;
use keira_mem::pmm;
use keira_task::scheduler::{
    get_current_egid, get_current_euid, get_current_uid, CURRENT_TASK_IDX, TASKS,
};
use keira_task::types::{FileDescriptor, MAX_FDS};

use crate::user_copy::{
    copy_from_user, copy_to_user, errno_to_ret, read_user_string, validate_user_ptr, EACCES,
    EAGAIN, EBADF, EBUSY, EEXIST, EFAULT, EINVAL, EIO, EISDIR, EMFILE, ENAMETOOLONG, ENODEV,
    ENOENT, ENOMEM, ENOSPC, ENOTDIR, ENOTEMPTY, EPERM, EPIPE, ESRCH,
};

/// Kernel representation of POSIX `struct stat` for userland ABI compatibility.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct KernelStat {
    /// Device ID containing file.
    pub st_dev: u64,
    /// Inode number.
    pub st_ino: u64,
    /// File type and mode bitmask.
    pub st_mode: u32,
    /// Number of hard links.
    pub st_nlink: u32,
    /// User ID of owner.
    pub st_uid: u32,
    /// Group ID of owner.
    pub st_gid: u32,
    /// Device ID (if special file).
    pub st_rdev: u64,
    /// Total file size in bytes.
    pub st_size: i64,
    /// Block size for filesystem I/O.
    pub st_blksize: u64,
    /// Number of 512B blocks allocated.
    pub st_blocks: u64,
    /// Time of last access.
    pub st_atime: i64,
    /// Time of last modification.
    pub st_mtime: i64,
    /// Time of last status change.
    pub st_ctime: i64,
}

/// Syscall 6: Open file or device node with POSIX flags.
pub fn handle_open(arg1: u64, arg2: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let flags = arg2 as u32;
    let write_mode = (flags & 0x0003) != 0 || flags == 1;
    let is_creat = (flags & 0x0040) != 0;
    let is_trunc = (flags & 0x0200) != 0;
    let is_append = (flags & 0x0400) != 0;

    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, target_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    let task_id = unsafe { CURRENT_TASK_IDX };
    let mac_mask = if write_mode {
        keira_task::security::MAC_WRITE
    } else {
        keira_task::security::MAC_READ
    };
    if !keira_task::security::check_path_access(task_id as u64, target_path, mac_mask) {
        return errno_to_ret(EACCES);
    }

    if write_mode && unsafe { keira_fs::lock::acquire_lock(target_path, task_id) }.is_err() {
        return errno_to_ret(EACCES);
    }

    let is_directory = (flags & 0x0001_0000) != 0;

    let current_euid = unsafe { get_current_euid() };
    let current_egid = unsafe { get_current_egid() };
    let desired_access = if (flags & 0x0003) == 2 {
        R_OK | W_OK
    } else if write_mode {
        W_OK
    } else {
        R_OK
    };

    let exists_val = exists(target_path);
    if !exists_val {
        if is_directory {
            return errno_to_ret(ENOENT);
        }
        if !write_mode && !is_creat {
            return errno_to_ret(ENOENT);
        }
        if create_file(target_path).is_err() {
            return errno_to_ret(EACCES);
        }
        let default_mode = if target_path.ends_with(".elf")
            || target_path.ends_with(".sh")
            || target_path.starts_with("/bin/")
        {
            0o755
        } else {
            0o644
        };
        let _ = set_ownership(target_path, current_euid, current_egid);
        let _ = set_permissions(target_path, default_mode);
    } else {
        if check_access(target_path, desired_access, current_euid, current_egid).is_err() {
            return errno_to_ret(EACCES);
        }
        let target_is_dir = is_dir(target_path);
        if write_mode && target_is_dir {
            return errno_to_ret(EISDIR);
        }
        if is_directory && !target_is_dir {
            return errno_to_ret(ENOTDIR);
        }
        if is_trunc {
            let _ = write_file(target_path, &[]);
        }
    }

    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            let mut fd_slot = None;
            for i in 0..MAX_FDS {
                if !t.fds[i].is_open {
                    fd_slot = Some(i);
                    break;
                }
            }
            if let Some(fd) = fd_slot {
                t.fds[fd].is_open = true;
                t.fds[fd].offset = if is_append {
                    keira_fs::vfs::get_file_size(target_path).unwrap_or(0) as u64
                } else {
                    0
                };
                t.fds[fd].write_mode = write_mode;
                t.fds[fd].path_len = target_len;
                t.fds[fd].path[..target_len].copy_from_slice(target_path.as_bytes());
                return fd as u64;
            }
        }
    }
    errno_to_ret(ENOMEM)
}

/// Syscall 7: Read from file descriptor into user buffer.
pub fn handle_read(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let fd = arg1 as usize;
    let buf_ptr = arg2;
    let len = arg3;
    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    if len == 0 {
        return 0;
    }
    if let Err(e) = unsafe { validate_user_ptr(buf_ptr, len, true) } {
        return errno_to_ret(e);
    }

    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if t.fds[fd].is_open {
                if t.fds[fd].is_pipe && !t.fds[fd].pipe_write {
                    let mut kernel_buf = [0u8; 1024];
                    let to_read = (len as usize).min(kernel_buf.len());
                    let pipe_id = t.fds[fd].pipe_id;
                    let res = if pipe_id != 0 {
                        keira_ipc::pipe::read_pipe_channel(pipe_id, &mut kernel_buf[..to_read])
                    } else {
                        Ok(keira_ipc::pipe::read_pipe(&mut kernel_buf[..to_read]))
                    };
                    match res {
                        Ok(bytes) => {
                            if bytes > 0 {
                                if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                    return bytes as u64;
                                }
                                return errno_to_ret(EFAULT);
                            }
                            return 0;
                        }
                        Err(code) => return code as u64,
                    }
                }

                if t.fds[fd].is_socket {
                    let mut kernel_buf = [0u8; 1024];
                    let to_read = (len as usize).min(kernel_buf.len());
                    match keira_net::socket::recv_socket(
                        t.fds[fd].socket_id as u64,
                        kernel_buf.as_mut_ptr(),
                        to_read,
                    ) {
                        Ok(bytes) => {
                            if bytes > 0 {
                                if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                    return bytes as u64;
                                }
                                return errno_to_ret(EFAULT);
                            }
                            return 0;
                        }
                        Err(_) => return errno_to_ret(EAGAIN),
                    }
                }

                let path_str = match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                    Ok(s) => s,
                    Err(_) => return errno_to_ret(EBADF),
                };

                if let Some(node_name) = path_str.strip_prefix("/dev/") {
                    let mut kernel_buf = [0u8; 512];
                    let to_read = (len as usize).min(kernel_buf.len());
                    if node_name == "tty" {
                        #[cfg(target_os = "none")]
                        loop {
                            if let Ok(bytes) =
                                keira_fs::dev::read_dev_node("tty", &mut kernel_buf[..to_read])
                            {
                                if bytes > 0 {
                                    if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                        return bytes as u64;
                                    }
                                    return errno_to_ret(EFAULT);
                                }
                                if t.fds[fd].nonblocking {
                                    return errno_to_ret(EAGAIN);
                                }
                            } else {
                                return errno_to_ret(EIO);
                            }
                            core::arch::asm!("sti; hlt");
                        }
                        #[cfg(not(target_os = "none"))]
                        {
                            if let Ok(bytes) =
                                keira_fs::dev::read_dev_node("tty", &mut kernel_buf[..to_read])
                            {
                                if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                    return bytes as u64;
                                }
                            }
                            return 0;
                        }
                    }

                    if let Ok(bytes) =
                        keira_fs::dev::read_dev_node(node_name, &mut kernel_buf[..to_read])
                    {
                        if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                            return bytes as u64;
                        }
                    }
                    return errno_to_ret(EFAULT);
                }

                let frame = match pmm::alloc_frame() {
                    Some(f) => f,
                    None => return errno_to_ret(ENOMEM),
                };
                let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                let bytes_read = match read_file(path_str, file_buf) {
                    Ok(b) => b,
                    Err(_) => {
                        pmm::free_frame(frame);
                        return errno_to_ret(EIO);
                    }
                };

                let offset = t.fds[fd].offset as usize;
                if offset >= bytes_read {
                    pmm::free_frame(frame);
                    return 0;
                }

                let avail = bytes_read - offset;
                let to_copy = (len as usize).min(avail);
                let slice_to_copy = &file_buf[offset..offset + to_copy];

                if let Err(e) = copy_to_user(buf_ptr, slice_to_copy) {
                    pmm::free_frame(frame);
                    return errno_to_ret(e);
                }

                t.fds[fd].offset += to_copy as u64;
                pmm::free_frame(frame);
                return to_copy as u64;
            } else if fd == 0 {
                let mut kernel_buf = [0u8; 128];
                let to_read = (len as usize).min(kernel_buf.len());
                #[cfg(target_os = "none")]
                loop {
                    if let Ok(bytes) =
                        keira_fs::dev::read_dev_node("tty", &mut kernel_buf[..to_read])
                    {
                        if bytes > 0 {
                            if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                return bytes as u64;
                            }
                            return errno_to_ret(EFAULT);
                        }
                    } else {
                        return errno_to_ret(EIO);
                    }
                    core::arch::asm!("sti; hlt");
                }
                #[cfg(not(target_os = "none"))]
                {
                    if let Ok(bytes) =
                        keira_fs::dev::read_dev_node("tty", &mut kernel_buf[..to_read])
                    {
                        if bytes > 0 {
                            if copy_to_user(buf_ptr, &kernel_buf[..bytes]).is_ok() {
                                return bytes as u64;
                            }
                            return errno_to_ret(EFAULT);
                        }
                    }
                    return 0;
                }
            }
        }
    }
    errno_to_ret(EBADF)
}

/// Syscall 8: Write to file descriptor from user buffer.
pub fn handle_write(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let fd = arg1 as usize;
    let buf_ptr = arg2;
    let len = arg3;
    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    if len == 0 {
        return 0;
    }
    if let Err(e) = unsafe { validate_user_ptr(buf_ptr, len, false) } {
        return errno_to_ret(e);
    }

    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if t.fds[fd].is_open && t.fds[fd].write_mode {
                if t.fds[fd].is_pipe && t.fds[fd].pipe_write {
                    let mut kernel_buf = [0u8; 1024];
                    let to_write = (len as usize).min(kernel_buf.len());
                    if copy_from_user(&mut kernel_buf[..to_write], buf_ptr).is_ok() {
                        let pipe_id = t.fds[fd].pipe_id;
                        let res = if pipe_id != 0 {
                            keira_ipc::pipe::write_pipe_channel(pipe_id, &kernel_buf[..to_write])
                        } else {
                            Ok(keira_ipc::pipe::write_pipe(&kernel_buf[..to_write]))
                        };
                        match res {
                            Ok(written) => return written as u64,
                            Err(code) => {
                                if code == -32 {
                                    let _ =
                                        keira_task::scheduler::send_signal(CURRENT_TASK_IDX, 13);
                                    return errno_to_ret(EPIPE);
                                }
                                return code as u64;
                            }
                        }
                    }
                    return errno_to_ret(EFAULT);
                }

                if t.fds[fd].is_socket {
                    let mut kernel_buf = [0u8; 1024];
                    let to_write = (len as usize).min(kernel_buf.len());
                    if copy_from_user(&mut kernel_buf[..to_write], buf_ptr).is_ok() {
                        match keira_net::socket::send_socket(
                            t.fds[fd].socket_id as u64,
                            kernel_buf.as_ptr(),
                            to_write,
                        ) {
                            Ok(bytes) => return bytes as u64,
                            Err(_) => return errno_to_ret(EIO),
                        }
                    }
                    return errno_to_ret(EFAULT);
                }

                let path_str = match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                    Ok(s) => s,
                    Err(_) => return errno_to_ret(EBADF),
                };

                if let Some(node_name) = path_str.strip_prefix("/dev/") {
                    let mut kernel_buf = [0u8; 512];
                    let to_write = (len as usize).min(kernel_buf.len());
                    if copy_from_user(&mut kernel_buf[..to_write], buf_ptr).is_ok() {
                        if let Ok(bytes) =
                            keira_fs::dev::write_dev_node(node_name, &kernel_buf[..to_write])
                        {
                            return bytes as u64;
                        }
                    }
                    return errno_to_ret(EFAULT);
                }

                let frame = match pmm::alloc_frame() {
                    Some(f) => f,
                    None => return errno_to_ret(ENOMEM),
                };
                let file_buf = core::slice::from_raw_parts_mut(frame as *mut u8, 4096);
                let mut current_size = read_file(path_str, file_buf).unwrap_or(0);

                let offset = t.fds[fd].offset as usize;
                if offset >= 4096 {
                    pmm::free_frame(frame);
                    return errno_to_ret(ENOSPC);
                }
                let to_write = (len as usize).min(4096 - offset);
                if to_write == 0 {
                    pmm::free_frame(frame);
                    return 0;
                }
                if let Err(e) = copy_from_user(&mut file_buf[offset..offset + to_write], buf_ptr) {
                    pmm::free_frame(frame);
                    return errno_to_ret(e);
                }

                if offset + to_write > current_size {
                    current_size = offset + to_write;
                }

                let write_res = write_file(path_str, &file_buf[..current_size]);
                pmm::free_frame(frame);

                if write_res.is_ok() {
                    t.fds[fd].offset += to_write as u64;
                    return to_write as u64;
                }
                return errno_to_ret(EIO);
            }
        }
    }

    if fd == 1 || fd == 2 {
        let mut chunk = [0u8; 128];
        let mut written = 0usize;
        while written < len as usize {
            let to_read = (len as usize - written).min(chunk.len());
            if unsafe { copy_from_user(&mut chunk[..to_read], buf_ptr + written as u64) }.is_err() {
                return if written > 0 {
                    written as u64
                } else {
                    errno_to_ret(EFAULT)
                };
            }
            if let Ok(s) = core::str::from_utf8(&chunk[..to_read]) {
                vga::print_str(s);
            } else {
                for &b in &chunk[..to_read] {
                    let single = [b];
                    if let Ok(s) = core::str::from_utf8(&single) {
                        vga::print_str(s);
                    }
                }
            }
            written += to_read;
        }
        return written as u64;
    }

    errno_to_ret(EBADF)
}

/// Syscall 9: Close open file descriptor.
pub fn handle_close(arg1: u64) -> u64 {
    let fd = arg1 as usize;
    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if t.fds[fd].is_open {
                if t.fds[fd].is_socket {
                    let _ = keira_net::socket::close_socket(t.fds[fd].socket_id as u64);
                } else if t.fds[fd].is_pipe {
                    if t.fds[fd].pipe_write {
                        keira_ipc::pipe::close_pipe_write(t.fds[fd].pipe_id);
                    } else {
                        keira_ipc::pipe::close_pipe_read(t.fds[fd].pipe_id);
                    }
                } else if t.fds[fd].write_mode {
                    let path_slice = &t.fds[fd].path[..t.fds[fd].path_len];
                    let other_open = (0..MAX_FDS).any(|i| {
                        i != fd
                            && t.fds[i].is_open
                            && t.fds[i].write_mode
                            && &t.fds[i].path[..t.fds[i].path_len] == path_slice
                    });
                    if !other_open {
                        if let Ok(path_str) = core::str::from_utf8(path_slice) {
                            let task_id = CURRENT_TASK_IDX;
                            keira_fs::lock::release_lock(path_str, task_id);
                        }
                    }
                }
                t.fds[fd] = FileDescriptor::new();
                return 0;
            }
        }
    }
    errno_to_ret(EBADF)
}

/// Syscall 10: Reposition read/write file offset.
pub fn handle_lseek(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let fd = arg1 as usize;
    let offset = arg2;
    let whence = arg3;
    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if t.fds[fd].is_open {
                match whence {
                    0 => t.fds[fd].offset = offset,
                    1 => t.fds[fd].offset = t.fds[fd].offset.saturating_add(offset),
                    _ => return errno_to_ret(EINVAL),
                }
                return t.fds[fd].offset;
            }
        }
    }
    errno_to_ret(EBADF)
}

/// Syscall 47: Splice data between file descriptors.
pub fn handle_splice(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match keira_ipc::pipe::sys_splice(arg1, arg2, arg3 as usize, 0) {
        Ok(len) => len as u64,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 48: Splice user pages into a pipe.
pub fn handle_vmsplice(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match keira_ipc::pipe::sys_vmsplice(arg1, arg2, arg3 as usize, 0) {
        Ok(bytes) => bytes as u64,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 70: Commit buffer cache to disk.
pub fn handle_sync() -> u64 {
    unsafe {
        let _ = keira_fs::cache::icache_flush_dirty();
        let _ = keira_fs::fat::flush_dirty_sectors();
        let _ = keira_io::storage::block::flush_mounted_device();
        keira_io::storage::ahci::flush_dma_cache();
    }
    0
}

/// Syscall 71: Synchronize a file's in-core state with storage device.
pub fn handle_fsync(arg1: u64) -> u64 {
    let fd = arg1 as usize;
    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    unsafe {
        let task = &TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if !t.fds[fd].is_open {
                return errno_to_ret(EBADF);
            }
        } else {
            return errno_to_ret(EBADF);
        }

        let _ = keira_fs::fat::flush_dirty_sectors();
        let _ = keira_io::storage::block::flush_mounted_device();
        keira_io::storage::ahci::flush_dma_cache();
    }
    0
}

/// Syscall 72: Manipulate file descriptor.
pub fn handle_fcntl(arg1: u64, arg2: u64) -> u64 {
    let fd = arg1 as usize;
    let cmd = arg2 as i32;
    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if !t.fds[fd].is_open {
                return errno_to_ret(EBADF);
            }
            match cmd {
                0 => {
                    for i in 0..MAX_FDS {
                        if !t.fds[i].is_open {
                            t.fds[i] = t.fds[fd];
                            return i as u64;
                        }
                    }
                    errno_to_ret(EMFILE)
                }
                1 => 0,
                2 => 0,
                3 => {
                    if t.fds[fd].write_mode {
                        1
                    } else {
                        0
                    }
                }
                4 => 0,
                _ => errno_to_ret(EINVAL),
            }
        } else {
            errno_to_ret(ESRCH)
        }
    }
}

/// Syscall 73: Control device / terminal ioctl.
pub fn handle_ioctl(arg2: u64, arg3: u64) -> u64 {
    let request = arg2;
    let argp = arg3;
    if argp == 0 {
        return errno_to_ret(EFAULT);
    }

    match request {
        0x5413 => {
            #[repr(C)]
            struct Winsize {
                ws_row: u16,
                ws_col: u16,
                ws_xpixel: u16,
                ws_ypixel: u16,
            }
            let ws = Winsize {
                ws_row: 25,
                ws_col: 80,
                ws_xpixel: 640,
                ws_ypixel: 400,
            };
            let ws_bytes = unsafe {
                core::slice::from_raw_parts(
                    &ws as *const _ as *const u8,
                    core::mem::size_of::<Winsize>(),
                )
            };
            if unsafe { copy_to_user(argp, ws_bytes) }.is_err() {
                return errno_to_ret(EFAULT);
            }
            0
        }
        0x5401 => {
            let term = keira_io::tty::get_termios();
            let term_bytes = unsafe {
                core::slice::from_raw_parts(
                    &term as *const _ as *const u8,
                    core::mem::size_of::<keira_io::tty::Termios>(),
                )
            };
            if unsafe { copy_to_user(argp, term_bytes) }.is_err() {
                return errno_to_ret(EFAULT);
            }
            0
        }
        0x5402..=0x5404 => {
            let mut term = keira_io::tty::Termios {
                c_iflag: 0,
                c_oflag: 0,
                c_cflag: 0,
                c_lflag: 0,
                c_line: 0,
                c_cc: [0u8; 32],
                c_ispeed: 0,
                c_ospeed: 0,
            };
            let term_bytes = unsafe {
                core::slice::from_raw_parts_mut(
                    &mut term as *mut _ as *mut u8,
                    core::mem::size_of::<keira_io::tty::Termios>(),
                )
            };
            if unsafe { copy_from_user(term_bytes, argp) }.is_err() {
                return errno_to_ret(EFAULT);
            }
            keira_io::tty::set_termios(&term);
            0
        }
        _ => errno_to_ret(EINVAL),
    }
}

/// Syscall 84: Duplicate open file descriptor.
pub fn handle_dup(arg1: u64) -> u64 {
    unsafe {
        let oldfd = arg1 as usize;
        if oldfd >= MAX_FDS {
            return errno_to_ret(EBADF);
        }
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if !t.fds[oldfd].is_open {
                if oldfd < 3 {
                    let mut desc = FileDescriptor::new();
                    desc.is_open = true;
                    desc.write_mode = oldfd != 0;
                    desc.path[..8].copy_from_slice(b"/dev/tty");
                    desc.path_len = 8;
                    t.fds[oldfd] = desc;
                } else {
                    return errno_to_ret(EBADF);
                }
            }
            for i in 0..MAX_FDS {
                if !t.fds[i].is_open {
                    t.fds[i] = t.fds[oldfd];
                    if t.fds[oldfd].is_pipe {
                        keira_ipc::pipe::dup_pipe_endpoint(
                            t.fds[oldfd].pipe_id,
                            t.fds[oldfd].pipe_write,
                        );
                    }
                    return i as u64;
                }
            }
            errno_to_ret(EMFILE)
        } else {
            errno_to_ret(EBADF)
        }
    }
}

/// Syscall 85: Duplicate open file descriptor to target fd.
pub fn handle_dup2(arg1: u64, arg2: u64) -> u64 {
    unsafe {
        let oldfd = arg1 as usize;
        let newfd = arg2 as usize;
        if oldfd >= MAX_FDS || newfd >= MAX_FDS {
            return errno_to_ret(EBADF);
        }
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if !t.fds[oldfd].is_open {
                if oldfd < 3 {
                    let mut desc = FileDescriptor::new();
                    desc.is_open = true;
                    desc.write_mode = oldfd != 0;
                    desc.path[..8].copy_from_slice(b"/dev/tty");
                    desc.path_len = 8;
                    t.fds[oldfd] = desc;
                } else {
                    return errno_to_ret(EBADF);
                }
            }
            if oldfd == newfd {
                return newfd as u64;
            }
            if t.fds[newfd].is_open {
                if t.fds[newfd].is_socket {
                    let _ = keira_net::socket::close_socket(t.fds[newfd].socket_id as u64);
                } else if t.fds[newfd].is_pipe {
                    if t.fds[newfd].pipe_write {
                        keira_ipc::pipe::close_pipe_write(t.fds[newfd].pipe_id);
                    } else {
                        keira_ipc::pipe::close_pipe_read(t.fds[newfd].pipe_id);
                    }
                } else if t.fds[newfd].write_mode {
                    let path_slice = &t.fds[newfd].path[..t.fds[newfd].path_len];
                    let other_open = (0..MAX_FDS).any(|i| {
                        i != newfd
                            && t.fds[i].is_open
                            && t.fds[i].write_mode
                            && &t.fds[i].path[..t.fds[i].path_len] == path_slice
                    });
                    if !other_open {
                        if let Ok(path_str) = core::str::from_utf8(path_slice) {
                            let task_id = CURRENT_TASK_IDX;
                            keira_fs::lock::release_lock(path_str, task_id);
                        }
                    }
                }
                t.fds[newfd] = FileDescriptor::new();
            }
            t.fds[newfd] = t.fds[oldfd];
            if t.fds[oldfd].is_pipe {
                keira_ipc::pipe::dup_pipe_endpoint(t.fds[oldfd].pipe_id, t.fds[oldfd].pipe_write);
            }
            newfd as u64
        } else {
            errno_to_ret(EBADF)
        }
    }
}

/// Directory entry descriptor populated by `SYS_GETDENTS` (Vector 86).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Dirent {
    pub d_ino: u64,
    pub d_off: i64,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
    pub _pad: [u8; 5],
}

/// Syscall 86: Read directory entries into user dirent buffer.
pub fn handle_getdents(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let fd = arg1 as usize;
    let buf_ptr = arg2;
    let count = arg3 as usize;

    if fd >= MAX_FDS {
        return errno_to_ret(EBADF);
    }
    if count < core::mem::size_of::<Dirent>() {
        return errno_to_ret(EINVAL);
    }
    if let Err(e) =
        unsafe { validate_user_ptr(buf_ptr, core::mem::size_of::<Dirent>() as u64, true) }
    {
        return errno_to_ret(e);
    }

    unsafe {
        let task = &mut TASKS[CURRENT_TASK_IDX];
        if let Some(t) = task {
            if !t.fds[fd].is_open {
                return errno_to_ret(EBADF);
            }

            let path_str = match core::str::from_utf8(&t.fds[fd].path[..t.fds[fd].path_len]) {
                Ok(s) => s,
                Err(_) => return errno_to_ret(EBADF),
            };

            let clean_path = path_str.trim_end_matches('/');
            let normalized_path = if clean_path.is_empty() {
                "/"
            } else {
                clean_path
            };

            if normalized_path == "/dev" || normalized_path == "dev" {
                let target_idx = t.fds[fd].offset as usize;
                if let Some(v_entry) = keira_fs::dev::get_dev_entry(target_idx) {
                    let mut d = Dirent {
                        d_ino: v_entry.ino,
                        d_off: (target_idx + 1) as i64,
                        d_reclen: core::mem::size_of::<Dirent>() as u16,
                        d_type: v_entry.d_type,
                        d_name: [0u8; 256],
                        _pad: [0u8; 5],
                    };
                    let name_bytes = v_entry.name.as_bytes();
                    let len = name_bytes.len().min(255);
                    d.d_name[..len].copy_from_slice(&name_bytes[..len]);

                    let entry_bytes = core::slice::from_raw_parts(
                        &d as *const Dirent as *const u8,
                        core::mem::size_of::<Dirent>(),
                    );
                    if copy_to_user(buf_ptr, entry_bytes).is_err() {
                        return errno_to_ret(EFAULT);
                    }
                    t.fds[fd].offset += 1;
                    return core::mem::size_of::<Dirent>() as u64;
                }
                return 0;
            }

            if normalized_path == "/proc" || normalized_path == "proc" {
                let target_idx = t.fds[fd].offset as usize;
                if let Some(v_entry) = keira_fs::proc::get_proc_entry(target_idx) {
                    let mut d = Dirent {
                        d_ino: v_entry.ino,
                        d_off: (target_idx + 1) as i64,
                        d_reclen: core::mem::size_of::<Dirent>() as u16,
                        d_type: v_entry.d_type,
                        d_name: [0u8; 256],
                        _pad: [0u8; 5],
                    };
                    let name_bytes = v_entry.name.as_bytes();
                    let len = name_bytes.len().min(255);
                    d.d_name[..len].copy_from_slice(&name_bytes[..len]);

                    let entry_bytes = core::slice::from_raw_parts(
                        &d as *const Dirent as *const u8,
                        core::mem::size_of::<Dirent>(),
                    );
                    if copy_to_user(buf_ptr, entry_bytes).is_err() {
                        return errno_to_ret(EFAULT);
                    }
                    t.fds[fd].offset += 1;
                    return core::mem::size_of::<Dirent>() as u64;
                }
                return 0;
            }

            let is_proc_pid_dir = if let Some(sub) = normalized_path.strip_prefix("/proc/") {
                sub == "self" || sub.parse::<usize>().is_ok()
            } else if let Some(sub) = normalized_path.strip_prefix("proc/") {
                sub == "self" || sub.parse::<usize>().is_ok()
            } else {
                false
            };

            if is_proc_pid_dir {
                let target_idx = t.fds[fd].offset as usize;
                if let Some(v_entry) = keira_fs::proc::get_pid_dir_entry(target_idx) {
                    let mut d = Dirent {
                        d_ino: v_entry.ino,
                        d_off: (target_idx + 1) as i64,
                        d_reclen: core::mem::size_of::<Dirent>() as u16,
                        d_type: v_entry.d_type,
                        d_name: [0u8; 256],
                        _pad: [0u8; 5],
                    };
                    let name_bytes = v_entry.name.as_bytes();
                    let len = name_bytes.len().min(255);
                    d.d_name[..len].copy_from_slice(&name_bytes[..len]);

                    let entry_bytes = core::slice::from_raw_parts(
                        &d as *const Dirent as *const u8,
                        core::mem::size_of::<Dirent>(),
                    );
                    if copy_to_user(buf_ptr, entry_bytes).is_err() {
                        return errno_to_ret(EFAULT);
                    }
                    t.fds[fd].offset += 1;
                    return core::mem::size_of::<Dirent>() as u64;
                }
                return 0;
            }

            let (_, fs_type) = keira_fs::vfs::route_path(normalized_path);
            if fs_type == keira_fs::vfs::FilesystemType::Tmpfs {
                let target_idx = t.fds[fd].offset as usize;
                if let Some((name_arr, name_len, d_type, ino)) =
                    keira_fs::vfs::get_tmpfs_entry(target_idx)
                {
                    let mut d = Dirent {
                        d_ino: ino,
                        d_off: (target_idx + 1) as i64,
                        d_reclen: core::mem::size_of::<Dirent>() as u16,
                        d_type,
                        d_name: [0u8; 256],
                        _pad: [0u8; 5],
                    };
                    let len = name_len.min(255);
                    d.d_name[..len].copy_from_slice(&name_arr[..len]);

                    let entry_bytes = core::slice::from_raw_parts(
                        &d as *const Dirent as *const u8,
                        core::mem::size_of::<Dirent>(),
                    );
                    if copy_to_user(buf_ptr, entry_bytes).is_err() {
                        return errno_to_ret(EFAULT);
                    }
                    t.fds[fd].offset += 1;
                    return core::mem::size_of::<Dirent>() as u64;
                }
                return 0;
            }

            let target_cluster = match keira_fs::fat::get_dir_cluster(normalized_path) {
                Ok(cluster) => cluster,
                Err(_) => return errno_to_ret(ENOTDIR),
            };

            let target_idx = t.fds[fd].offset as usize;
            let mut curr_idx = 0usize;
            let mut found_entry: Option<Dirent> = None;

            let _ = keira_fs::fat::for_each_directory_entry(target_cluster, |parsed| {
                if curr_idx == target_idx {
                    let mut d = Dirent {
                        d_ino: (parsed.sector as u64) * 16 + (parsed.index as u64) + 1,
                        d_off: (curr_idx + 1) as i64,
                        d_reclen: core::mem::size_of::<Dirent>() as u16,
                        d_type: if (parsed.entry.attr & 0x10) != 0 {
                            4
                        } else {
                            8
                        },
                        d_name: [0u8; 256],
                        _pad: [0u8; 5],
                    };
                    let len = parsed.name_len.min(255);
                    d.d_name[..len].copy_from_slice(&parsed.name[..len]);
                    found_entry = Some(d);
                    return Ok(false);
                }
                curr_idx += 1;
                Ok(true)
            });

            if let Some(entry) = found_entry {
                let entry_bytes = core::slice::from_raw_parts(
                    &entry as *const Dirent as *const u8,
                    core::mem::size_of::<Dirent>(),
                );
                if copy_to_user(buf_ptr, entry_bytes).is_err() {
                    return errno_to_ret(EFAULT);
                }
                t.fds[fd].offset += 1;
                return core::mem::size_of::<Dirent>() as u64;
            }

            0
        } else {
            errno_to_ret(EBADF)
        }
    }
}

/// Syscall 87: Unlink / remove file from virtual filesystem.
pub fn handle_unlink(arg1: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _target_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_path,
        keira_task::security::MAC_WRITE,
    ) {
        return errno_to_ret(EACCES);
    }

    match keira_fs::vfs::remove_entry(target_path) {
        Ok(_) => 0,
        Err(e) => {
            if e.contains("not empty") {
                errno_to_ret(ENOTEMPTY)
            } else if e.contains("read-only") || e.contains("cannot be removed") {
                errno_to_ret(EACCES)
            } else {
                errno_to_ret(ENOENT)
            }
        }
    }
}

/// Syscall 88: Create a directory node in the virtual filesystem.
pub fn handle_mkdir(arg1: u64, _arg2: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if path_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _target_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_path,
        keira_task::security::MAC_WRITE,
    ) {
        return errno_to_ret(EACCES);
    }

    if keira_fs::vfs::exists(target_path) {
        return errno_to_ret(EEXIST);
    }

    match keira_fs::vfs::create_dir(target_path) {
        Ok(_) => {
            let current_euid = unsafe { get_current_euid() };
            let current_egid = unsafe { get_current_egid() };
            let dir_mode = if _arg2 != 0 {
                (_arg2 as u32) & 0o777
            } else {
                0o755
            };
            let _ = set_ownership(target_path, current_euid, current_egid);
            let _ = set_permissions(target_path, dir_mode);
            0
        }
        Err(e) => {
            if e.contains("already exists") {
                errno_to_ret(EEXIST)
            } else if e.contains("read-only") {
                errno_to_ret(EACCES)
            } else {
                errno_to_ret(ENOENT)
            }
        }
    }
}

/// Syscall 92: Create a symbolic link pointing to a target pathname.
pub fn handle_symlink(arg1: u64, arg2: u64) -> u64 {
    let target_ptr = arg1 as *const u8;
    let link_ptr = arg2 as *const u8;

    let mut target_buf = [0u8; 128];
    let target_len = match unsafe { read_user_string(target_ptr, &mut target_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let target_str = match core::str::from_utf8(&target_buf[..target_len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    let mut link_buf = [0u8; 128];
    let link_len = match unsafe { read_user_string(link_ptr, &mut link_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let link_str = match core::str::from_utf8(&link_buf[..link_len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if target_str.is_empty() || link_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut resolved_link_buf = [0u8; 128];
    let (target_link_path, _norm_len) = if !link_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    link_str,
                    &mut resolved_link_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_link_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (link_str, link_len)
    };

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_link_path,
        keira_task::security::MAC_WRITE,
    ) {
        return errno_to_ret(EACCES);
    }

    match keira_fs::vfs::create_symlink(target_str, target_link_path) {
        Ok(()) => 0,
        Err(e) => {
            if e.contains("File exists") {
                errno_to_ret(EEXIST)
            } else if e.contains("too long") {
                errno_to_ret(ENAMETOOLONG)
            } else if e.contains("full") {
                errno_to_ret(ENOSPC)
            } else {
                errno_to_ret(EINVAL)
            }
        }
    }
}

/// Syscall 93: Read value of a symbolic link into user buffer.
pub fn handle_readlink(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let user_buf = arg2;
    let bufsiz = arg3 as usize;

    if bufsiz == 0 {
        return errno_to_ret(EINVAL);
    }

    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _norm_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_path,
        keira_task::security::MAC_READ,
    ) {
        return errno_to_ret(EACCES);
    }

    let mut kbuf = [0u8; 128];
    match keira_fs::vfs::read_symlink(target_path, &mut kbuf) {
        Ok(bytes_read) => {
            let to_copy = core::cmp::min(bufsiz, bytes_read);
            if let Err(e) = unsafe { copy_to_user(user_buf, &kbuf[..to_copy]) } {
                return errno_to_ret(e);
            }
            to_copy as u64
        }
        Err(e) => {
            if e.contains("not a symbolic link") {
                errno_to_ret(EINVAL)
            } else {
                errno_to_ret(ENOENT)
            }
        }
    }
}

/// Syscall 94: Mount a filesystem into the virtual directory hierarchy.
pub fn handle_mount(arg1: u64, arg2: u64, arg3: u64, arg4: u64, _arg5: u64) -> u64 {
    let source_ptr = arg1 as *const u8;
    let target_ptr = arg2 as *const u8;
    let fstype_ptr = arg3 as *const u8;
    let flags = arg4 as u32;

    if target_ptr.is_null() || fstype_ptr.is_null() {
        return errno_to_ret(EINVAL);
    }

    let uid = unsafe { get_current_uid() };
    if uid != 0 {
        return errno_to_ret(EPERM);
    }

    let mut source_buf = [0u8; 128];
    let source_str = if !source_ptr.is_null() {
        match unsafe { read_user_string(source_ptr, &mut source_buf) } {
            Ok(l) => match core::str::from_utf8(&source_buf[..l]) {
                Ok(s) => s,
                Err(_) => return errno_to_ret(EINVAL),
            },
            Err(e) => return errno_to_ret(e),
        }
    } else {
        "none"
    };

    let mut target_buf = [0u8; 128];
    let target_len = match unsafe { read_user_string(target_ptr, &mut target_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let target_str = match core::str::from_utf8(&target_buf[..target_len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if target_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut fstype_buf = [0u8; 32];
    let fstype_len = match unsafe { read_user_string(fstype_ptr, &mut fstype_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let fstype_str = match core::str::from_utf8(&fstype_buf[..fstype_len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    let fs_type = match keira_fs::vfs::FilesystemType::from_str(fstype_str) {
        Some(t) => t,
        None => return errno_to_ret(ENODEV),
    };

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _norm_len) = if !target_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    target_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (target_str, target_len)
    };

    match keira_fs::vfs::vfs_mount(source_str, target_path, fs_type, flags) {
        Ok(()) => 0,
        Err(e) => {
            if e.contains("already mounted") {
                errno_to_ret(EBUSY)
            } else if e.contains("full") {
                errno_to_ret(ENOSPC)
            } else {
                errno_to_ret(EINVAL)
            }
        }
    }
}

/// Syscall 95: Unmount a filesystem from the virtual directory hierarchy.
pub fn handle_umount(arg1: u64, arg2: u64) -> u64 {
    let target_ptr = arg1 as *const u8;
    let flags = arg2 as u32;

    if target_ptr.is_null() {
        return errno_to_ret(EINVAL);
    }

    let uid = unsafe { get_current_uid() };
    if uid != 0 {
        return errno_to_ret(EPERM);
    }

    let mut target_buf = [0u8; 128];
    let target_len = match unsafe { read_user_string(target_ptr, &mut target_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let target_str = match core::str::from_utf8(&target_buf[..target_len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if target_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _norm_len) = if !target_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    target_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (target_str, target_len)
    };

    match keira_fs::vfs::vfs_umount(target_path, flags) {
        Ok(()) => 0,
        Err(e) => {
            if e.contains("busy") || e.contains("root") {
                errno_to_ret(EBUSY)
            } else if e.contains("not found") {
                errno_to_ret(ENOENT)
            } else {
                errno_to_ret(EINVAL)
            }
        }
    }
}

/// Syscall 97: Change file mode permissions according to POSIX DAC rules.
pub fn handle_chmod(arg1: u64, arg2: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let mode = (arg2 as u32) & 0o7777;

    if path_ptr.is_null() {
        return errno_to_ret(EINVAL);
    }

    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if path_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _norm_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    if !exists(target_path) && !is_dir(target_path) {
        return errno_to_ret(ENOENT);
    }

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_path,
        keira_task::security::MAC_WRITE,
    ) {
        return errno_to_ret(EACCES);
    }

    let current_euid = unsafe { get_current_euid() };
    let (owner_uid, _, _) = get_metadata(target_path);

    // POSIX DAC check: caller must be superuser (EUID 0) or the file owner
    if current_euid != 0 && current_euid != owner_uid {
        return errno_to_ret(EPERM);
    }

    match set_permissions(target_path, mode) {
        Ok(()) => 0,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 98: Change file owner and group identity.
pub fn handle_chown(arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let uid = arg2 as u32;
    let gid = arg3 as u32;

    if path_ptr.is_null() {
        return errno_to_ret(EINVAL);
    }

    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if path_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _norm_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    if !exists(target_path) && !is_dir(target_path) {
        return errno_to_ret(ENOENT);
    }

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_path,
        keira_task::security::MAC_WRITE,
    ) {
        return errno_to_ret(EACCES);
    }

    let current_euid = unsafe { get_current_euid() };

    // POSIX DAC check: only superuser (EUID 0) may change file owner
    if current_euid != 0 {
        return errno_to_ret(EPERM);
    }

    match set_ownership(target_path, uid, gid) {
        Ok(()) => 0,
        Err(_) => errno_to_ret(EINVAL),
    }
}

/// Syscall 99: Query file status and metadata into userland stat buffer.
pub fn handle_stat(arg1: u64, arg2: u64) -> u64 {
    let path_ptr = arg1 as *const u8;
    let statbuf_ptr = arg2;

    if path_ptr.is_null() || statbuf_ptr == 0 {
        return errno_to_ret(EINVAL);
    }

    if let Err(e) =
        unsafe { validate_user_ptr(statbuf_ptr, core::mem::size_of::<KernelStat>() as u64, true) }
    {
        return errno_to_ret(e);
    }

    let mut path_buf = [0u8; 128];
    let len = match unsafe { read_user_string(path_ptr, &mut path_buf) } {
        Ok(l) => l,
        Err(e) => return errno_to_ret(e),
    };
    let path_str = match core::str::from_utf8(&path_buf[..len]) {
        Ok(s) => s,
        Err(_) => return errno_to_ret(EINVAL),
    };

    if path_str.is_empty() {
        return errno_to_ret(ENOENT);
    }

    let mut resolved_path_buf = [0u8; 128];
    let (target_path, _norm_len) = if !path_str.starts_with('/') {
        unsafe {
            if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
                let current_cwd = core::str::from_utf8(&t.cwd[..t.cwd_len]).unwrap_or("/");
                match crate::dispatcher::handlers::process::normalize_path(
                    current_cwd,
                    path_str,
                    &mut resolved_path_buf,
                ) {
                    Ok(norm_len) => (
                        core::str::from_utf8(&resolved_path_buf[..norm_len]).unwrap_or("/"),
                        norm_len,
                    ),
                    Err(e) => return errno_to_ret(e),
                }
            } else {
                ("/", 1)
            }
        }
    } else {
        (path_str, len)
    };

    let target_is_dir = is_dir(target_path);
    let target_exists = exists(target_path) || target_is_dir;

    if !target_exists {
        return errno_to_ret(ENOENT);
    }

    let task_id = unsafe { CURRENT_TASK_IDX };
    if !keira_task::security::check_path_access(
        task_id as u64,
        target_path,
        keira_task::security::MAC_READ,
    ) {
        return errno_to_ret(EACCES);
    }

    let (uid, gid, mode) = get_metadata(target_path);
    let file_size = if target_is_dir {
        4096
    } else {
        get_file_size(target_path).unwrap_or(0)
    };

    let stat = KernelStat {
        st_dev: 1,
        st_ino: 1,
        st_mode: mode,
        st_nlink: if target_is_dir { 2 } else { 1 },
        st_uid: uid,
        st_gid: gid,
        st_rdev: 0,
        st_size: file_size as i64,
        st_blksize: 4096,
        st_blocks: (file_size as u64 + 511) / 512,
        st_atime: 1700000000,
        st_mtime: 1700000000,
        st_ctime: 1700000000,
    };

    let stat_slice = unsafe {
        core::slice::from_raw_parts(
            &stat as *const KernelStat as *const u8,
            core::mem::size_of::<KernelStat>(),
        )
    };

    if let Err(e) = unsafe { copy_to_user(statbuf_ptr, stat_slice) } {
        return errno_to_ret(e);
    }

    0
}

/// Syscall 100: Query file descriptor status and metadata into userland stat buffer.
pub fn handle_fstat(arg1: u64, arg2: u64) -> u64 {
    let fd = arg1 as usize;
    let statbuf_ptr = arg2;

    if fd >= MAX_FDS || statbuf_ptr == 0 {
        return errno_to_ret(EBADF);
    }

    if let Err(e) =
        unsafe { validate_user_ptr(statbuf_ptr, core::mem::size_of::<KernelStat>() as u64, true) }
    {
        return errno_to_ret(e);
    }

    let (st_mode, st_size, st_ino, st_uid, st_gid, st_nlink, st_rdev) = unsafe {
        if let Some(ref t) = TASKS[CURRENT_TASK_IDX] {
            let f = &t.fds[fd];
            if !f.is_open {
                return errno_to_ret(EBADF);
            }

            if f.is_pipe {
                (
                    0o010666, // S_IFIFO | 0666
                    0i64,
                    1000 + f.pipe_id as u64,
                    get_current_uid(),
                    0,
                    1,
                    0u64,
                )
            } else if f.is_socket {
                (
                    0o140666, // S_IFSOCK | 0666
                    0i64,
                    2000 + f.socket_id as u64,
                    get_current_uid(),
                    0,
                    1,
                    0u64,
                )
            } else {
                let path_str = core::str::from_utf8(&f.path[..f.path_len]).unwrap_or("");
                if path_str.is_empty() {
                    // Standard streams (stdin, stdout and stderr)
                    (
                        0o020666, // S_IFCHR | 0666
                        0i64,
                        fd as u64 + 1,
                        get_current_uid(),
                        0,
                        1,
                        5u64,
                    )
                } else {
                    let target_is_dir = is_dir(path_str);
                    let (uid, gid, mode) = get_metadata(path_str);
                    let size = if target_is_dir {
                        4096
                    } else {
                        get_file_size(path_str).unwrap_or(0)
                    };
                    (
                        mode,
                        size as i64,
                        100 + fd as u64,
                        uid,
                        gid,
                        if target_is_dir { 2 } else { 1 },
                        0u64,
                    )
                }
            }
        } else {
            return errno_to_ret(ESRCH);
        }
    };

    let stat = KernelStat {
        st_dev: 1,
        st_ino,
        st_mode,
        st_nlink,
        st_uid,
        st_gid,
        st_rdev,
        st_size,
        st_blksize: 4096,
        st_blocks: (st_size as u64).div_ceil(512),
        st_atime: 1700000000,
        st_mtime: 1700000000,
        st_ctime: 1700000000,
    };

    let stat_slice = unsafe {
        core::slice::from_raw_parts(
            &stat as *const KernelStat as *const u8,
            core::mem::size_of::<KernelStat>(),
        )
    };

    if let Err(e) = unsafe { copy_to_user(statbuf_ptr, stat_slice) } {
        return errno_to_ret(e);
    }

    0
}

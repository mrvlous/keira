// SPDX-License-Identifier: GPL-2.0-only

//! Unit tests for task control block types and file descriptor descriptors.

use super::*;

#[test]
fn test_task_limits() {
    assert_eq!(MAX_TASKS, 64);
    assert_eq!(MAX_FDS, 32);
}

#[test]
fn test_file_descriptor_initialization() {
    let fd = FileDescriptor::new();
    assert!(!fd.is_open);
    assert_eq!(fd.offset, 0);
    assert!(!fd.write_mode);
    assert!(!fd.is_socket);
    assert!(!fd.is_pipe);

    let sock = FileDescriptor::new_socket(5, true);
    assert!(sock.is_open);
    assert!(sock.is_socket);
    assert_eq!(sock.socket_id, 5);
    assert!(sock.nonblocking);

    let pipe = FileDescriptor::new_pipe(1, true);
    assert!(pipe.is_open);
    assert!(pipe.is_pipe);
    assert_eq!(pipe.pipe_id, 1);
    assert!(pipe.pipe_write);
}

#[test]
fn test_task_state_equality() {
    assert_eq!(TaskState::Ready, TaskState::Ready);
    assert_ne!(TaskState::Running, TaskState::Blocked);
    assert_eq!(TaskState::Zombie(0), TaskState::Zombie(0));
    assert_ne!(TaskState::Zombie(0), TaskState::Zombie(1));
}

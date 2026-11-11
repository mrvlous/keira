// SPDX-License-Identifier: GPL-2.0-only

//! Real-Time Clock (RTC) and CMOS subsystem.

pub mod cmos;

pub use self::cmos::{get_time, init, RtcTime};

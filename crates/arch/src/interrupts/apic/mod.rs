// SPDX-License-Identifier: GPL-2.0-only

//! Local Advanced Programmable Interrupt Controller (LAPIC) subsystem.

pub mod lapic;

pub use lapic::{
    enable_lapic, eoi, get_current_lapic_id, read_reg, wait_icr_idle, write_reg,
    LAPIC_DEFAULT_BASE, LAPIC_EOI_REG, LAPIC_ICR_HIGH_REG, LAPIC_ICR_LOW_REG, LAPIC_ID_REG,
    LAPIC_SVR_REG, LAPIC_TIMER_CURR_CNT, LAPIC_TIMER_DIV_REG, LAPIC_TIMER_INIT_CNT,
    LAPIC_TIMER_LVT_REG, LAPIC_TPR_REG, LAPIC_VER_REG,
};

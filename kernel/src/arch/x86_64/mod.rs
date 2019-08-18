//**************************************************************************************************
// mod.rs                                                                                          *
// Copyright (c) 2019 Todd Berta-Oldham                                                            *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

#[macro_use]
#[allow(dead_code)]
#[allow(unused_macros)]
pub mod debug;
pub mod gdt;
pub mod idt;
pub mod tss;

use x86::interrupts;

#[no_mangle]
pub unsafe extern "C" fn main() {
    debug::init();

    crate::print_header();

    interrupts::disable();

    gdt::install();

    idt::install();

    interrupts::enable();

    crate::main_stage_2();
}

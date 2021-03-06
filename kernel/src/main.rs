//**************************************************************************************************
// main.rs                                                                                         *
// Copyright (c) 2018-2021 The Verdure Project                                                     *
// This code is made available under the MIT License.                                              *
//**************************************************************************************************

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(alloc_layout_extra)]
#![feature(alloc_error_handler)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(associated_type_bounds)]
#![feature(once_cell)]

extern crate alloc;

use crate::spinlock::Spinlock;
use core::alloc::Layout;
use core::panic::PanicInfo;
use kernel_interface::init::Args;

#[macro_use]
mod arch;
mod acpi_interface;
pub mod drivers;
mod frame;
mod heap;
pub mod icm;
mod pmm;
mod spinlock;
mod tasks;
pub mod tm;

pub use acpi_interface::*;

pub unsafe fn main(args: &'static Args) -> ! {
    loop {}
}

pub unsafe fn main_ap() -> ! {
    loop {}
}

#[alloc_error_handler]
fn on_oom(layout: Layout) -> ! {
    println!("Kernel heap has run out of memory.");
    println!("{:?}", layout);
    unsafe { arch::stall() }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel panic.");
    println!("{}", info);
    unsafe { arch::stall() }
}

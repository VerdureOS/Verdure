// *************************************************************************
// lib.rs
// Copyright 2019 Todd Berta-Oldham
// This code is made available under the MIT License.
// *************************************************************************

#![no_std]
#![feature(alloc)]
#![feature(alloc_layout_extra)]
#![feature(alloc_error_handler)]

extern crate alloc;

mod ffi;
mod drawing;
mod error;
pub mod memory;
pub mod graphics;
pub mod text_io;
pub mod storage;
pub mod string;
pub mod protocol;
pub mod system;

pub use self::system as uefi_system;
pub use self::ffi::{ SystemTable, Handle, Status };
pub use self::drawing::*;
pub use self::error::*;

use self::ffi::*;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::alloc::{ GlobalAlloc, Layout };

struct UEFIAllocator;

unsafe impl GlobalAlloc for UEFIAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 { 
        let system_table = &*uefi_system::system_table().expect("UEFI Core was not initialized before allocating memory.");

        if system_table.boot_services.is_null() {
            panic!("Boot services are not available. New alllocations should not be made.");
        }

        let boot_services = &*system_table.boot_services;

        let mut buffer = null_mut();
        let buffer_size = layout.size();

        ((boot_services.allocate_pool)(MemoryType::LoaderData, buffer_size, &mut buffer));

        return buffer as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let system_table = &*uefi_system::system_table().expect("UEFI Core was not initialized before freeing memory.");

        if system_table.boot_services.is_null() {
            return;
        }

        let boot_services = &*system_table.boot_services;

        (boot_services.free_pool)(ptr as *mut c_void);    
    }
}

#[global_allocator]
static ALLOCATOR : UEFIAllocator = UEFIAllocator;

#[alloc_error_handler]
fn on_oom(_layout: Layout) -> ! {
    loop {}
}
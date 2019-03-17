// *************************************************************************
// main.rs
// Copyright 2019 Todd Berta-Oldham
// This code is made available under the MIT License.
// *************************************************************************

#![no_std]
#![no_main]
#![feature(alloc)]
#![feature(alloc_layout_extra)]
#![feature(alloc_error_handler)]

extern crate alloc;

use uefi_core::{Handle, Status, SystemTable, printrln, uefi_system, ProtocolProvider, UefiError, UefiIoError };
use uefi_core::graphics::GraphicsOutputProvider;
use uefi_core::storage::VolumeProvider;
use core::fmt::Write;
use core::mem;
use alloc::vec::Vec;
use elf::{ ElfIdentityHeader, Elf64Header };

#[no_mangle]
pub unsafe extern "win64" fn efi_main(image_handle : Handle, system_table : *mut SystemTable) -> Status {
    uefi_system::init(image_handle, system_table).expect("Failed to initialize UEFI system.");
    main();
    Status::SUCCESS    
}

fn main() {    
    initialize_graphics_and_console();

    load_kernel();
    
    loop { }
}

fn initialize_graphics_and_console() {
    let provider = GraphicsOutputProvider::new().expect("Failed to create graphics output provider.");
    
    for output in provider.iter() {
        output.maximize(true).unwrap();
    }

    printrln!("Pet UEFI Boot Loader");
    printrln!("Copyright 2019 Todd Berta-Oldham");

    if cfg!(debug_assertions) {
        printrln!("This is a debug build.");
    }

    for id in 0..provider.len() {
        let output = provider.open(id).expect("Failed to open graphics output provider.");
        match output.framebuffer_address() {
            Some(address) => printrln!("Graphics output {} initialized at address {:#X} with {}x{} resolution.", id, address, output.width(), output.height()),
            None => printrln!("Graphics output {} could not be initialized with a linear framebuffer.", id)
        }
    }
}

fn load_kernel() {
    let kernel_buffer = read_kernel_from_disk().into_boxed_slice();
    
    unsafe {
        let kernel_buffer_pointer = kernel_buffer.as_ptr();
        let kernel_buffer_size = kernel_buffer.len();

        if kernel_buffer_size < mem::size_of::<Elf64Header>() {
            panic!("Kernel is only {} byte(s).", kernel_buffer_size);
        }
        
        let id_header = &*(kernel_buffer_pointer as *const ElfIdentityHeader);
        
        if !id_header.is_valid() {
            panic!("Kernel is not a valid ELF file. Header shows {:#X}, {:#X}, {:#X}, {:#X}.", id_header.magic_0, id_header.magic_1, id_header.magic_2, id_header.magic_3);
        }

        if !id_header.is_64bit() {
            panic!("Kernel is not 64 bit.");
        }

        printrln!("Kernel is a valid x86_64 ELF file.");
    }
}

fn read_kernel_from_disk() -> Vec<u8> {
    printrln!("Searching for kernel...");

    let provider = VolumeProvider::new().expect("Failed to create volume provider.");

    let mut kernel_buffer = Vec::new();

    for id in 0..provider.len()  {
        printrln!("Checking volume {}...", id);

        match provider.open(id).and_then(|volume| { volume.root_node() }) {
            Ok(root) => {
                match root.open_node("Boot\\Kernel", true, false) {
                    Ok(kernel_node) => { 
                        printrln!("Kernel found. Reading...");
                        kernel_node.read_to_end(&mut kernel_buffer).expect("Failed to read kernel from disk.");
                        printrln!("Read {} bytes from disk.", kernel_buffer.len());
                        return kernel_buffer;
                    }
                    Err(error) => {
                        if let UefiError::IoError(io_error) = &error {
                            if let UefiIoError::PathNonExistent(_) = io_error {
                                printrln!("Kernel not found.");
                                continue;
                            }
                        }
                        panic!("The error \"{}\" occured while trying to find and open kernel.", &error);
                    }
                }
            },
            Err(error) => { 
                printrln!("The error \"{}\" occured while trying to open volume {}. Skipping...", error, id) 
            }
        }
    }

    panic!("Failed to find and read kernel. It either doesn't exist or it's volume encountered an error while being opened.");
}
// *************************************************************************
// main.rs
// Copyright 2019 Todd Berta-Oldham
// This code is made available under the MIT License.
// *************************************************************************

#![no_std]
#![no_main]

use uefi_core::{Handle, Status, SystemTable, printrln, uefi_system, GraphicsOutputProvider, VolumeProvider };
use core::fmt::Write;
use core::panic::PanicInfo;

#[no_mangle]
pub unsafe extern "win64" fn efi_main(image_handle : Handle, system_table : *mut SystemTable) -> Status {
    uefi_system::init(image_handle, system_table);
    main();
    Status::Success    
}

fn main() {    
    {
        // Initialize graphics, print header, and then print graphics info.

        let provider = GraphicsOutputProvider::new().expect("Failed to create graphics output provider");
        
        for id in 0..provider.len() {
            provider.get(id).unwrap().maximize(true).unwrap();
        }

        printrln!("Pet UEFI Boot Loader").unwrap();
        printrln!("Copyright 2019 Todd Berta-Oldham").unwrap();

        if cfg!(debug_assertions) {
            printrln!("This is a debug build.").unwrap();
        }

        for id in 0..provider.len() {
            let output = provider.get(id).unwrap();
            match output.framebuffer_address() {
                Some(address) => printrln!("Graphics output {} initialized at address {:#X} with {}x{} resolution.", id, address, output.width(), output.height()).unwrap(),
                None => printrln!("Graphics output {} could not be initialized with a linear framebuffer.", id).unwrap()
            }
        }
    }

    {
        // Read kernel from disk.

        printrln!("Searching for kernel...").unwrap();

        let provider = VolumeProvider::new().expect("Failed to create volume provider");

        for id in 0..provider.len()  {
            let volume = provider.get(id).unwrap();
            let root = volume.root_node().unwrap();
        }
    }

    loop { }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
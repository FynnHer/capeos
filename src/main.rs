// src/main.rs

#![no_std]
#![no_main]

#![feature(abi_efiapi)]

use core::panic::PanicInfo;
use core::ffi::c_void;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "efiapi" fn efi_main(
    image: *mut c_void,
    system_table: *const c_void,
) -> usize {
    loop {}
}
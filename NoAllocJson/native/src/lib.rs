
#![no_std]

#[panic_handler]
#[cfg(not(test))]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
#[cfg(target_env = "msvc")]
mod msvc {
    #[link(name = "vcruntime")]
    extern "C" {}
    #[link(name = "msvcrt")]
    extern "C" {}

    #[no_mangle]
    pub static _fltused: i32 = 0;

    #[no_mangle]
    extern "system" fn __chkstk() {}

    #[no_mangle]
    extern "system" fn _DllMainCRTStartup(_: *const u8, reason: u32, _: *const u8) -> u32 {
        if reason == 1 {
            unsafe {
                crate::mono::mono_add_internal_call(
                    "NoAllocJson.Patch::SerializeInplace\0".as_ptr(),
                    crate::serialize_inplace as *const core::ffi::c_void,
                );
                //println!("Registered internal calls");
            }
        }
        1
    }
}

/*
#[no_mangle]
extern "system" fn DllMain(_: *const u8, reason: u32, _: *const u8) -> u32 {
    if reason == 1 {
        unsafe {
            crate::mono::initialize();
            crate::mono::mono_add_internal_call(
                "NoAllocJson.Patch::SerializeInplace\0".as_ptr(),
                crate::serialize_inplace as *const core::ffi::c_void,
            );
            println!("Registered internal calls");
        }
    }
    1
}
*/

mod mono;
mod ser;

use ser::*;
use mono::*;

#[no_mangle]
pub fn serialize_inplace(data: &'static MonoObject, dmem: &'static mut MonoString) -> usize {
    let mut ptr = &mut dmem.chars as *mut u16;
    let length = data.serialize_inplace(&mut ptr);
    dmem.length = length as i32;
    length
}

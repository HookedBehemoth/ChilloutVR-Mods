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

    #[no_mangle]
    pub static _fltused: i32 = 0;

    #[no_mangle]
    extern "system" fn __chkstk() {}

    #[no_mangle]
    extern "system" fn _DllMainCRTStartup(_: *const u8, _: u32, _: *const u8) -> u32 {
        1
    }
}

#[cfg(target_env = "gnu")]
mod mingw {
    #[no_mangle]
    extern "system" fn DllMainCRTStartup(_: *const u8, _: u32, _: *const u8) -> u32 {
        1
    }
}

/// # Safety
///
/// We have to trust the caller to supply valid ptr/sz pairs to the function.
#[no_mangle]
pub unsafe extern "C" fn decrypt(
    guid_ptr: *const u8,
    guid_len: usize,
    data_ptr: *const u8,
    data_len: usize,
    key_ptr: *const u8,
    key_len: usize,
    dst_ptr: *mut u8,
) {
    let guid = core::slice::from_raw_parts(guid_ptr, guid_len);
    let data = core::slice::from_raw_parts(data_ptr, data_len);
    let key = core::slice::from_raw_parts(key_ptr, key_len);
    let dst = core::slice::from_raw_parts_mut(dst_ptr, data_len + key_len);

    decrypt::decrypt_internal(&guid, data, key, dst);
}

mod decrypt;

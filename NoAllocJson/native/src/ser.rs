#[cfg(not(std))]
use core::{ptr, slice};
#[cfg(std)]
use std::{ptr, slice};

use utf16_lit::utf16;

use crate::mono::*;

pub trait Serializeable {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize;
}

impl<const N: usize> Serializeable for &[u16; N] {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), *dmem, self.len());
            *dmem = dmem.add(self.len());
        }
        self.len()
    }
}

impl Serializeable for &[u16] {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        unsafe {
            ptr::copy_nonoverlapping(self.as_ptr(), *dmem, self.len());
            *dmem = dmem.add(self.len());
        }
        self.len()
    }
}

impl Serializeable for crate::mono::MonoString {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        unsafe {
            **dmem = '"' as u16;
            *dmem = dmem.add(1);
            let length = if self.length > 0 {
                slice::from_raw_parts(&self.chars as *const u16, self.length as usize)
                    .serialize_inplace(dmem)
            } else {
                0
            };
            **dmem = '"' as u16;
            *dmem = dmem.add(1);
            length + 2
        }
    }
}

impl Serializeable for i64 {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        let chars: [u16; 10] = utf16!("0123456789");
        let mut buf: [u16; 20] = [0_u16; 20];
        let mut length: usize = 0;
        let mut tmp: usize = if self.is_negative() {
            unsafe {
                **dmem = utf16!("-")[0];
                *dmem = dmem.wrapping_add(1);
            }
            -(*self as i64) as usize
        } else {
            *self as usize
        };
        loop {
            unsafe {
                let rem = tmp.checked_rem(10).unwrap_unchecked();
                let index = buf.len() - 1 - length;
                *buf.get_unchecked_mut(index) = *chars.get_unchecked(rem);
                length = length.wrapping_add(1);
                tmp = tmp.wrapping_div(10);
                if tmp == 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr().add(buf.len() - length), *dmem, length);
            *dmem = dmem.add(length);
        }
        if self.is_negative() {
            length + 1
        } else {
            length
        }
    }
}

impl Serializeable for u64 {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        let chars: [u16; 10] = utf16!("0123456789");
        let mut buf: [u16; 20] = [0_u16; 20];
        let mut length: usize = 0;
        let mut tmp: usize = *self as usize;
        loop {
            unsafe {
                let rem = tmp.checked_rem(10).unwrap_unchecked();
                let index = buf.len() - 1 - length;
                *buf.get_unchecked_mut(index) = *chars.get_unchecked(rem);
                length = length.wrapping_add(1);
                tmp = tmp.wrapping_div(10);
                if tmp == 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr().add(buf.len() - length), *dmem, length);
            *dmem = dmem.add(length);
        }
        length
    }
}

trait Float {}
impl Float for f32 {}
impl Float for f64 {}

impl<T: Float + ryu::Float> Serializeable for T {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        let mut buffer = ryu::Buffer::new();
        let printed = buffer.format(*self);
        for c in printed.chars() {
            unsafe {
                **dmem = c as u16;
                *dmem = dmem.add(1);
            }
        }
        printed.len()
    }
}

impl Serializeable for bool {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        if *self {
            (&utf16!("true")).serialize_inplace(dmem)
        } else {
            (&utf16!("false")).serialize_inplace(dmem)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Serializeable;
    use utf16_lit::utf16;

    macro_rules! test {
        ($val:expr, $ty:ty) => {
            let a: $ty = $val;
            let mut buffer = [0_u16; stringify!($val).len()];
            let mut ptr = buffer.as_mut_ptr();
            let _ = a.serialize_inplace(&mut ptr);
            assert_eq!(buffer, utf16!(stringify!($val)));
        };
    }

    #[test]
    fn serialize_int() {
        macro_rules! test_signed {
            ($val:expr) => {
                test!($val, i64)
            };
        }
        test_signed!(123456789);
        test_signed!(-99999);
        test_signed!(0);
        test_signed!(-2147483648);
        test_signed!(2147483647);
    }
    #[test]
    fn serialize_uint() {
        macro_rules! test_unsigned {
            ($val:expr) => {
                test!($val, u64)
            };
        }
        test_unsigned!(123456789);
        test_unsigned!(0);
        test_unsigned!(18446744073709551615);
    }
    #[test]
    fn serialize_bool() {
        macro_rules! test_bool {
            ($val:expr) => {
                test!($val, bool)
            };
        }
        test_bool!(true);
        test_bool!(false);
    }
}

impl Serializeable for *const RawString {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        let mut length = 0;

        let mut it = *self as *const u8;
        unsafe {
            loop {
                let c = *it;
                if c == 0 {
                    break;
                }
                **dmem = c as u16;
                *dmem = dmem.add(1);
                it = it.add(1);
                length += 1;
            }
        }

        length
    }
}

impl Serializeable for char {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        unsafe {
            **dmem = *self as u16;
            *dmem = dmem.add(1);
        }
        1
    }
}

impl Serializeable for MonoChar {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        unsafe {
            *dmem.add(0) = '"' as u16;
            *dmem.add(1) = self.val;
            *dmem.add(2) = '"' as u16;
            *dmem = dmem.add(3);
        }
        3
    }
}

impl Serializeable for MonoArray {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        /* FIXME: This assumes all arrays are one dimensional */
        let mut length = 0;
        unsafe {
            let typ = &*mono_class_get_type(self.object.vtable.klass.eclass as _);
            let ptr = &self.vector as *const usize as *const core::ffi::c_void;
            let stride = mono_array_element_size(self.object.vtable.klass) as usize;

            length += '['.serialize_inplace(dmem);
            for i in 0..self.max_length {
                if i != 0 {
                    length += ','.serialize_inplace(dmem);
                }
                let ptr = ptr.add(i as usize * stride);
                length += serialize_value(typ, ptr as _, dmem)
            }
            length += ']'.serialize_inplace(dmem);
        }
        length
    }
}

#[repr(C)]
struct ManagedList {
    object: MonoObject,
    items: &'static MonoArray,
    #[cfg(feature = "sgen")]
    _sync: &'static MonoObject,
    size: i32,
    _version: i32,
    #[cfg(feature = "boehm")]
    _sync: &'static MonoObject,
}

#[test]
fn assert_size() {
    assert_eq!(core::mem::size_of::<ManagedList>(), 40);
}

impl Serializeable for ManagedList {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        let mut length = 0;
        unsafe {
            let typ = &*mono_class_get_type(self.items.object.vtable.klass.eclass as _);
            let ptr = &self.items.vector as *const usize as *const core::ffi::c_void;
            let stride = mono_array_element_size(self.items.object.vtable.klass) as usize;

            length += '['.serialize_inplace(dmem);
            for i in 0..self.size {
                if i != 0 {
                    length += ','.serialize_inplace(dmem);
                }
                let ptr = ptr.add(i as usize * stride);
                length += serialize_value(typ, ptr as _, dmem)
            }
            length += ']'.serialize_inplace(dmem);
        }
        length
    }
}

unsafe fn serialize_value(
    typ: &'static MonoType,
    ptr: *const core::ffi::c_void,
    dmem: &mut *mut u16,
) -> usize {
    macro_rules! push {
        ($val:expr) => {
            ($val).serialize_inplace(dmem)
        };
    }

    macro_rules! reference {
        ($ty:ty) => {
            ptr as *const $ty
        };
    }

    match (*typ).typ {
        /* END & VOID aren't valid field types */
        MonoTypeEnum::MONO_TYPE_BOOLEAN => {
            push!(*reference!(bool))
        }
        MonoTypeEnum::MONO_TYPE_CHAR => {
            push!(*reference!(MonoChar))
        }
        MonoTypeEnum::MONO_TYPE_I1 => {
            push!(*reference!(i8) as i64)
        }
        MonoTypeEnum::MONO_TYPE_I2 => {
            push!(*reference!(i16) as i64)
        }
        MonoTypeEnum::MONO_TYPE_I4 => {
            push!(*reference!(i32) as i64)
        }
        MonoTypeEnum::MONO_TYPE_I8 | MonoTypeEnum::MONO_TYPE_I => {
            push!(*reference!(i64))
        }
        MonoTypeEnum::MONO_TYPE_U1 => {
            push!(*reference!(u8) as u64)
        }
        MonoTypeEnum::MONO_TYPE_U2 => {
            push!(*reference!(u16) as u64)
        }
        MonoTypeEnum::MONO_TYPE_U4 => {
            push!(*reference!(u32) as u64)
        }
        MonoTypeEnum::MONO_TYPE_U8 | MonoTypeEnum::MONO_TYPE_U => {
            push!(*reference!(u64))
        }
        MonoTypeEnum::MONO_TYPE_R4 => {
            push!(*reference!(f32))
        }
        MonoTypeEnum::MONO_TYPE_R8 => {
            push!(*reference!(f64))
        }
        MonoTypeEnum::MONO_TYPE_STRING => {
            let ptr = *reference!(*const MonoString);
            if ptr.is_null() {
                push!(&utf16!(r#""""#))
            } else {
                push!(*ptr)
            }
        }
        /* Field can't be PTR or BYREF */
        MonoTypeEnum::MONO_TYPE_VALUETYPE => {
            // log_class(&*(*typ).klass);
            // 0
            serialize_object(
                &*(*typ).klass,
                reference!(core::ffi::c_void).wrapping_sub(0x10),
                dmem,
            )
        },
        MonoTypeEnum::MONO_TYPE_CLASS
        // | MonoTypeEnum::MONO_TYPE_GENERICINST
        | MonoTypeEnum::MONO_TYPE_OBJECT => {
            let ptr = *reference!(*const MonoObject);
            if ptr.is_null() {
                // println!("Object is null");
                push!(&utf16!("{}"))
            } else {
                // let typ = mono_class_get_type((*ptr).vtable.klass);
                let flags = mono_class_get_flags((*ptr).vtable.klass);
                // println!("0x{:X}", flags);
                /* Serializable */
                if (flags & 0x2000) == 0 {
                    // println!("Object isn't serializable");
                    push!(&utf16!("{}"))
                } else {
                    push!(*ptr)
                }
            }
        }
        MonoTypeEnum::MONO_TYPE_GENERICINST => {
            /* FIXME: Just assuming every generic class is a list */
            let arr = *reference!(*const ManagedList);
            if arr.is_null() {
                push!(&utf16!("[]"))
            } else {
                push!(*arr)
            }
        }
        /* VAR? */
        MonoTypeEnum::MONO_TYPE_ARRAY | MonoTypeEnum::MONO_TYPE_SZARRAY => {
            let arr = *reference!(*const MonoArray);
            if arr.is_null() {
                push!(&utf16!("[]"))
            } else {
                push!(*arr)
            }
        }
        _ => {
        //ty => {
            // println!("Failed to recognize: {:?}", ty);
            push!(&utf16!("null"))
        }
    }
}

unsafe fn serialize_object(
    klass: &'static MonoClass,
    ptr: *const core::ffi::c_void,
    dmem: &mut *mut u16,
) -> usize {
    let mut length = 0;

    macro_rules! push {
        ($val:expr) => {
            length += ($val).serialize_inplace(dmem);
        };
    }

    push!('{');

    let mut iter = ptr::null();
    loop {
        let field = mono_class_get_fields(klass, &mut iter as _);
        if field.is_null() {
            break;
        }
        let field = &*field;
        let typ = (*field).typ;
        /* STATIC | NOT_SERIALIZED */
        if (typ.attrs & 0x90) != 0 {
            continue;
        }
        let name = field.name;
        let offset = field.offset;

        if length > 1 {
            push!(',');
        }
        push!('"');
        push!(name);
        push!(&utf16!("\":"));

        // println!(
        //     "{:?}: offset: {}, {:?}",
        //     std::ffi::CStr::from_ptr(name as *const i8),
        //     offset,
        //     typ.typ
        // );

        length += serialize_value(typ, ptr.add(offset as usize), dmem)
    }
    push!('}');

    length
}

/*
unsafe fn log_class(
    klass: &'static MonoClass,
) {
    let mut iter = ptr::null();
    loop {
        let field = mono_class_get_fields(klass, &mut iter as _);
        if field.is_null() {
            break;
        }
        let field = &*field;
        let offset = field.offset;
        let name = field.name;
        let typ = (*field).typ;

        println!(
            "{:?}: offset: {}, {:?}",
            std::ffi::CStr::from_ptr(name as *const i8),
            offset,
            typ.typ
        );
    }
}
*/

impl Serializeable for MonoObject {
    fn serialize_inplace(&self, dmem: &mut *mut u16) -> usize {
        unsafe {
            serialize_object(
                self.vtable.klass,
                self as *const MonoObject as *const core::ffi::c_void,
                dmem,
            )
        }
    }
}

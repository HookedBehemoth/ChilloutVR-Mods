#![allow(improper_ctypes)]
#![allow(non_camel_case_types)]
#![allow(unused)]

use core::fmt::Write;
#[cfg(not(std))]
use core::{ffi, fmt};
#[cfg(std)]
use std::{ffi, fmt};

extern "C" {
    pub fn mono_add_internal_call(name: *const u8, method: *const ffi::c_void);
    pub fn mono_class_get_name(klass: *const MonoClass) -> *const RawString;
    pub fn mono_class_get_fields(klass: *const MonoClass, iter: *const *const ffi::c_void) -> *const MonoClassField;
    pub fn mono_class_get_type(klass: *const MonoClass) -> *const MonoType;
	pub fn mono_class_get_flags(klass: *const MonoClass) -> u32;
    pub fn mono_field_get_name(field: *const MonoClassField) -> *const RawString;
    pub fn mono_field_get_offset(field: *const MonoClassField) -> u32;
    pub fn mono_field_get_type(field: *const MonoClassField) -> *const MonoType;
    pub fn mono_array_element_size(klass: *const MonoClass) -> i32;
}

pub struct RawString {}

#[repr(C)]
pub struct MonoObject {
    pub vtable: &'static MonoVTable,
    sync: &'static MonoThreadsSync,
}

#[repr(C)]
pub struct MonoVTable {
    pub klass: &'static MonoClass,
}
#[repr(C)]
pub struct MonoThreadsSync {}
#[repr(C)]
pub struct MonoClass {
    pub eclass: &'static MonoClass,
}
#[repr(C)]
pub struct MonoClassField {
    pub typ: &'static MonoType,
    pub name: *const RawString,
    pub parent: &'static MonoClass,
    pub offset: i32,
}

#[repr(C)]
pub struct MonoType {
    pub klass: &'static MonoClass,
    pub attrs: u16,
    pub typ: MonoTypeEnum,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum MonoTypeEnum {
	MONO_TYPE_END        = 0x00,       /* End of List */
	MONO_TYPE_VOID       = 0x01,
	MONO_TYPE_BOOLEAN    = 0x02,
	MONO_TYPE_CHAR       = 0x03,
	MONO_TYPE_I1         = 0x04,
	MONO_TYPE_U1         = 0x05,
	MONO_TYPE_I2         = 0x06,
	MONO_TYPE_U2         = 0x07,
	MONO_TYPE_I4         = 0x08,
	MONO_TYPE_U4         = 0x09,
	MONO_TYPE_I8         = 0x0a,
	MONO_TYPE_U8         = 0x0b,
	MONO_TYPE_R4         = 0x0c,
	MONO_TYPE_R8         = 0x0d,
	MONO_TYPE_STRING     = 0x0e,
	MONO_TYPE_PTR        = 0x0f,       /* arg: <type> token */
	MONO_TYPE_BYREF      = 0x10,       /* arg: <type> token */
	MONO_TYPE_VALUETYPE  = 0x11,       /* arg: <type> token */
	MONO_TYPE_CLASS      = 0x12,       /* arg: <type> token */
	MONO_TYPE_VAR	     = 0x13,	   /* number */
	MONO_TYPE_ARRAY      = 0x14,       /* type, rank, boundsCount, bound1, loCount, lo1 */
	MONO_TYPE_GENERICINST= 0x15,	   /* <type> <type-arg-count> <type-1> \x{2026} <type-n> */
	MONO_TYPE_TYPEDBYREF = 0x16,
	MONO_TYPE_I          = 0x18,
	MONO_TYPE_U          = 0x19,
	MONO_TYPE_FNPTR      = 0x1b,	      /* arg: full method signature */
	MONO_TYPE_OBJECT     = 0x1c,
	MONO_TYPE_SZARRAY    = 0x1d,       /* 0-based one-dim-array */
	MONO_TYPE_MVAR	     = 0x1e,       /* number */
	MONO_TYPE_CMOD_REQD  = 0x1f,       /* arg: typedef or typeref token */
	MONO_TYPE_CMOD_OPT   = 0x20,       /* optional arg: typedef or typref token */
	MONO_TYPE_INTERNAL   = 0x21,       /* CLR internal type */

	MONO_TYPE_MODIFIER   = 0x40,       /* Or with the following types */
	MONO_TYPE_SENTINEL   = 0x41,       /* Sentinel for varargs method signature */
	MONO_TYPE_PINNED     = 0x45,       /* Local var that points to pinned object */

	MONO_TYPE_ENUM       = 0x55        /* an enumeration */
}

#[repr(C)]
pub struct MonoArray {
    pub object: MonoObject,
    pub bounds: &'static MonoArrayBounds,
    pub max_length: usize,
    pub vector: usize,
}

#[repr(C)]
pub struct MonoArrayBounds {}

#[repr(C)]
pub struct MonoString {
    object: MonoObject,
    pub length: i32,
    pub chars: u16,
}

#[repr(C)]
pub struct MonoChar {
    pub val: u16,
}

#[test]
fn test_sizes() {
    use core::mem::size_of;
    assert_eq!(size_of::<MonoObject>(), 0x10);
    assert_eq!(size_of::<MonoString>(), 0x18);
}

extern "C" {
	// pub i_enumerable_get: extern "C" fn()
}
pub fn initialize() {

}

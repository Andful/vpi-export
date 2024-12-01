#![no_std]
#![feature(const_mut_refs)]
#![feature(maybe_uninit_uninit_array)]

#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
pub mod vpi_user;
use core::{
    ffi::c_void,
    ptr::NonNull,
    str,
    sync::atomic::{AtomicPtr, Ordering},
};

pub use ctor::ctor;
pub use vpi_export_macro::vpi_export;
pub use vpi_user::vpi_printf;
pub use bitvec::prelude::BitArray;

pub struct FunctionCollections {
    head: AtomicPtr<FnWithChild>,
}

struct FnWithChild {
    f: fn(),
    next: Option<NonNull<FnWithChild>>,
}

impl FunctionCollections {
    const fn new() -> Self {
        Self {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    pub fn push(&self, f: fn()) {
        //SAFETY: Calling malloc, writing to pointer and then dereferencing.
        let (ptr, new_head) = unsafe {
            let ptr = libc::malloc(core::mem::size_of::<FnWithChild>()) as *mut FnWithChild;
            ptr.write(FnWithChild { f, next: None });
            let new_head = &mut *ptr;
            (ptr, new_head)
        };
        new_head.next = NonNull::new(self.head.swap(ptr, Ordering::SeqCst));
    }
}

pub static __FUNCTION_COLLECTIONS__: FunctionCollections = FunctionCollections::new();

pub extern "C" fn register_vpi_functions() {
    let mut head = NonNull::new(
        __FUNCTION_COLLECTIONS__
            .head
            .swap(core::ptr::null_mut(), Ordering::Relaxed),
    );
    while let Some(elem_ptr) = head {
        {
            let elem = unsafe { elem_ptr.as_ref() };
            (elem.f)();
            head = elem.next;
        }
        //SAFETY: Pointer is allocated with malloc and there are no dangling elements
        unsafe { libc::free(elem_ptr.as_ptr() as *mut c_void) };
    }
}

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static vlog_startup_routines: [Option<extern "C" fn()>; 2] =
    [Some(register_vpi_functions), None];

pub trait FromVpiHandle {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self;
}

impl <const N: usize> FromVpiHandle for bitvec::prelude::BitArray<[u32; N]> {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiVectorVal as i32,
            ..Default::default()
        };
        let size = vpi_user::vpi_get(vpi_user::vpiSize as i32, handle);
        let elem = (size-1) / 32 + 1;
        vpi_user::vpi_get_value(handle, &mut value as *mut vpi_user::t_vpi_value);
        value.value.vector;
        let mut data = [0u32; N];
        for i in 0..N.min(elem as usize) {
            data[i] = (*(value.value.vector.add(i))).aval as u32;
        }
        bitvec::prelude::BitArray::new(data)
    }
}

impl FromVpiHandle for i32 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiIntVal as i32,
            ..Default::default()
        };
        vpi_user::vpi_get_value(handle, &mut value as *mut vpi_user::t_vpi_value);
        value.value.integer
    }
}

impl FromVpiHandle for isize {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as isize
    }
}

impl FromVpiHandle for i64 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as i64
    }
}

impl FromVpiHandle for i16 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as i16
    }
}

impl FromVpiHandle for i8 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as i8
    }
}

impl FromVpiHandle for usize {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as usize
    }
}

impl FromVpiHandle for u64 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as u64
    }
}

impl FromVpiHandle for u16 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as u16
    }
}

impl FromVpiHandle for u8 {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        i32::from_vpi_handle(handle) as u8
    }
}

impl FromVpiHandle for &str {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        let s: &core::ffi::CStr = FromVpiHandle::from_vpi_handle(handle);
        s.to_str().unwrap()
    }
}

impl FromVpiHandle for &core::ffi::CStr {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Self {
        let mut value = vpi_user::t_vpi_value {
            format: vpi_user::vpiStringVal as i32,
            ..Default::default()
        };
        vpi_user::vpi_get_value(handle, &mut value as *mut vpi_user::t_vpi_value);
        core::ffi::CStr::from_ptr(value.value.str_)
    }
}

pub fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    unsafe { libc::abort() }
}

pub fn eh_personality() -> ! {
    unsafe { libc::abort() }
}

pub fn print(c: &core::ffi::CStr) {
    unsafe {
        vpi_user::vpi_printf(c.as_ptr() as *const core::ffi::c_char);
    }
}

pub fn println(c: &core::ffi::CStr) {
    print(c);
    print(&unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(b"\n\0") });
}

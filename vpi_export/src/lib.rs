#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![warn(missing_docs)]

//! Vpi-export
//!
//! Allows exporting of rust functions as VPI functions

extern crate alloc;

#[doc(hidden)]
pub mod __hidden__;
use core::{ffi::{c_void, CStr}, ptr::null_mut};

pub use vpi_user;
mod bitvec;
mod handle;
mod impls;
mod vpi_iter;
pub use bitvec::BitVector;
pub use vpi_export_macro::{bitvec, vpi_task};
pub use vpi_user::vpi_printf;

mod __private {
    pub trait Sealed {}
}

pub use handle::Handle;
pub use vpi_iter::VpiIter;

pub trait VpiTaskResult: __private::Sealed {
    fn into_vpi_result(self) -> Result<()>;
}

///Error due to conversion from verilog type to rust type
#[derive(Debug)]
pub enum VpiConversionError {
    ///String conversion error from verilog to rust
    Utf8Error(core::str::Utf8Error),
    ///Vector length missmat
    BitVectorMissMatch {
        ///Expected length
        expected: usize,
        ///Obtained length
        actual: usize,
    },
}

///Result to a conversion from verilog type to rust type
pub type Result<T> = core::result::Result<T, VpiConversionError>;

///Conversion trait from verilog to rust
pub trait FromVpiHandle: Sized {
    ///Conversion function from verilog to rust. In implementation, use the function
    /// [crate::vpi_user::vpi_get_value] to obtain the value to convert.
    /// # Safety
    /// handle must NOT be dangling or null
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self>;
}

///Conversion trait from rust to verilog
pub trait IntoVpiHandle: Sized {
    ///Conversion function from rust to verilog. In implementation, use the function
    /// [crate::vpi_user::vpi_put_value] to conver type to verilog.
    /// # Safety
    /// handle must NOT be dangling or null
    unsafe fn into_vpi_handle(&self, handle: vpi_user::vpiHandle) -> Result<()>;
}

unsafe extern "C" fn cb(data: *mut vpi_user::t_cb_data) -> i32 {

    let data = unsafe {&mut *data};
    let data = unsafe { &mut *(data.user_data as *mut CallbackData) };
    let f = unsafe {&mut *data.callback};
    f();
    0
}

struct CallbackData {
    handle: vpi_user::vpiHandle,
    raw_callback_pointer: *mut u8,
    callback: *mut dyn FnMut() -> (),
    callback_layout: alloc::alloc::Layout,
}

pub struct VpiCallbackHandle(vpi_user::vpiHandle);

///Register callback
pub fn on_value_change<'a, E: FromVpiHandle, F: FnMut() -> () + Sized + 'static>(value: &Handle<E>, f: F) -> VpiCallbackHandle {
    use alloc::alloc::{alloc, handle_alloc_error, Layout};
    let callback_layout = Layout::new::<F>();
    let raw_callback_pointer = unsafe { alloc(callback_layout) };
    let callback = raw_callback_pointer as *mut F;
    if callback.is_null() {
        handle_alloc_error(callback_layout);
    }
    unsafe {
        callback.write(f);
    }
    let data_layout = Layout::new::<CallbackData>();
    let data = unsafe { alloc(callback_layout) } as *mut CallbackData;
    unsafe {
        data.write(CallbackData {
            handle: null_mut(),
            raw_callback_pointer,
            callback,
            callback_layout,
        });
    }
    let cb_data_layout = Layout::new::<vpi_user::t_cb_data>();
    let cb_data = unsafe { alloc(cb_data_layout) } as *mut vpi_user::t_cb_data;
    if cb_data.is_null() {
        handle_alloc_error(cb_data_layout);
    }
    unsafe {
        cb_data.write(vpi_user::t_cb_data {
            reason: vpi_user::cbValueChange as i32,
            cb_rtn: Some(cb),
            obj: value.handle,
            user_data: data as *mut vpi_user::PLI_BYTE8,
            ..Default::default()
        });
    }
    VpiCallbackHandle(unsafe {
        vpi_user::vpi_register_cb(cb_data)
    })
}

pub fn remove_cb(cb_handle: VpiCallbackHandle) {
    //TODO free data
    unsafe { vpi_user::vpi_remove_cb(cb_handle.0) };
}

///Print function that internally will use the simulator's print function.
pub fn print(c: &core::ffi::CStr) {
    unsafe {
        vpi_user::vpi_printf(c.as_ptr() as *mut core::ffi::c_char);
    }
}

///Print function that internally will use the simulator's print function with an appended new line.
pub fn println(c: &core::ffi::CStr) {
    print(c);
    print(unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(b"\n\0") });
}

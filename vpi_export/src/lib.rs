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
use core::{ffi::CStr, ptr::NonNull};

pub use vpi_user;
mod bitvec;
mod clk;
mod handle;
mod impls;
mod vpi_iter;
pub use bitvec::BitVector;
pub use clk::Clk;
pub use vpi_export_macro::{bitvec, vpi_module, vpi_task};
pub use vpi_user::vpi_printf;
use vpi_user::{vpiSimTime, vpi_get_time};

mod __private {
    pub trait Sealed {}
}

pub use handle::Handle;
pub use vpi_iter::VpiIter;

/// Not null [vpi_user::vpiHandle]
pub type RawHandle = NonNull<vpi_user::PLI_UINT32>;

///Possible result of a vpi task
pub trait VpiTaskResult: __private::Sealed {
    ///Conversion into [VpiTaskResult]
    fn into_vpi_result(self) -> Result<()>;
}

///Errors relating to VPI
#[derive(Debug)]
#[non_exhaustive]
pub enum VpiError {
    ///String conversion error from verilog to rust
    Utf8Error(core::str::Utf8Error),
    ///Module was not found
    NoModule(&'static CStr),
    ///Vector length missmat
    BitVectorLengthMissMatch {
        ///Expected length
        expected: usize,
        ///Obtained length
        actual: usize,
    },
    ///Period too small
    PeriodTooSmall,
}

///Result relating to a vpi result
pub type Result<T> = core::result::Result<T, VpiError>;

///Conversion trait from verilog to rust
pub trait FromVpiHandle: Sized {
    ///Conversion function from verilog to rust. In implementation, use the function
    /// [crate::vpi_user::vpi_get_value] to obtain the value to convert.
    /// # Safety
    /// handle must NOT be dangling or null
    unsafe fn from_vpi_handle(handle: RawHandle) -> Result<Self>;
}

///Conversion trait from rust to verilog
pub trait StoreIntoVpiHandle: Sized {
    ///Conversion function from rust to verilog. In implementation, use the function
    /// [crate::vpi_user::vpi_put_value] to conver type to verilog.
    /// # Safety
    /// handle must NOT be dangling or null
    unsafe fn store_into_vpi_handle(&self, handle: RawHandle) -> Result<()>;
}

unsafe extern "C" fn cb(data: *mut vpi_user::t_cb_data) -> i32 {
    let data = unsafe { &mut *data };
    let data = unsafe { &mut *(data.user_data as *mut CallbackData) };
    let f = unsafe { &mut *data.callback };
    f();
    0
}

struct CallbackData {
    raw_callback_pointer: *mut u8,
    callback: *mut dyn FnMut(),
    callback_layout: alloc::alloc::Layout,
}

///Callback handle wrapper
pub struct VpiCallbackHandle(vpi_user::vpiHandle, *const CallbackData);

///Register callback for on value change
pub fn on_value_change<E: FromVpiHandle, F: FnMut() + Sized + 'static>(
    value: Handle<E>,
    f: F,
) -> VpiCallbackHandle {
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
    let data = unsafe { alloc(data_layout) } as *mut CallbackData;
    unsafe {
        data.write(CallbackData {
            raw_callback_pointer,
            callback,
            callback_layout,
        });
    }
    let mut cb_data = vpi_user::t_cb_data {
        reason: vpi_user::cbValueChange as i32,
        cb_rtn: Some(cb),
        obj: value.handle.as_ptr(),
        user_data: data as *mut vpi_user::PLI_BYTE8,
        ..Default::default()
    };
    VpiCallbackHandle(unsafe { vpi_user::vpi_register_cb(&mut cb_data) }, data)
}

fn on_delay_internal<F: FnMut() + Sized + 'static>(delay: u64, f: F) -> VpiCallbackHandle {
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
    let data = unsafe { alloc(data_layout) } as *mut CallbackData;
    unsafe {
        data.write(CallbackData {
            raw_callback_pointer,
            callback,
            callback_layout,
        });
    }

    let mut cb_data = vpi_user::t_cb_data {
        reason: vpi_user::cbAfterDelay as i32,
        cb_rtn: Some(cb),
        time: &mut vpi_user::t_vpi_time {
            type_: vpiSimTime as i32,
            high: (delay >> 32) as u32,
            low: (delay & ((!0u64) >> 32)) as u32,
            ..Default::default()
        },
        user_data: data as *mut vpi_user::PLI_BYTE8,
        ..Default::default()
    };
    VpiCallbackHandle(unsafe { vpi_user::vpi_register_cb(&mut cb_data) }, data)
}

///Callback after delay
pub fn on_delay<F: FnOnce() + Sized + 'static>(delay: u64, f: F) -> VpiCallbackHandle {
    let mut f = Some(f);
    on_delay_internal(delay, move || {
        if let Some(f) = core::mem::take(&mut f) {
            f();
        }
    })
}

///Obtain simulation time
pub fn get_time() -> u64 {
    let mut t = vpi_user::t_vpi_time {
        type_: vpiSimTime as i32,
        ..Default::default()
    };
    unsafe { vpi_get_time(core::ptr::null_mut(), &mut t) };
    let mut result = t.high as u64;
    result <<= 32;
    result |= t.low as u64;
    result
}

/// Equivalent to $finish
pub fn finish() {
    unsafe {
        vpi_user::vpi_control(vpi_user::vpiFinish as i32);
    }
}

///Remove callback handle
pub fn remove_cb(cb_handle: VpiCallbackHandle) {
    //TODO free data even though there does not seem to be a sensible way of doing it
    //Data is shared with the simulator
    let a = unsafe { &*cb_handle.1 };
    let _ = a.callback_layout;
    let _ = a.raw_callback_pointer;
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

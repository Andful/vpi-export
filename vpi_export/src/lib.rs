#![no_std]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![warn(missing_docs)]

//! Vpi-export
//!
//! Allows exporting of rust functions as VPI functions

#[doc(hidden)]
pub mod __hidden__;
pub use vpi_user;
mod bitvec;
mod impls;
mod marker;
pub use bitvec::BitVector;
pub use marker::{input::Input, input_output::InputOutput, output::Output};
pub use vpi_export_macro::{bitvec, vpi_task};
pub use vpi_user::vpi_printf;

mod __private {
    pub trait Sealed {}
}

/// Trait required for vpi task arguments
pub trait VpiTaskArg<'a>: __private::Sealed {
    ///Argument to keep in the stack when initializing VpiTaskArg.
    type Data;
    /// # Safety
    ///
    /// Handle must not be null or dangling
    unsafe fn initialize_data(handle: vpi_user::vpiHandle) -> Result<Self::Data>;
    ///Initializer
    fn new(e: &'a mut Self::Data) -> Self;
    /// # Safety
    ///
    /// Handle must not be null or dangling
    unsafe fn finalize_data(e: Self::Data, handle: vpi_user::vpiHandle) -> Result<()>;
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
    unsafe fn into_vpi_handle(self, handle: vpi_user::vpiHandle) -> Result<()>;
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

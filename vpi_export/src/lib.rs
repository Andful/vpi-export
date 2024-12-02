#![no_std]
#![feature(associated_type_defaults)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![warn(missing_docs)]

//! Vpi-export
//!
//! Allows exporting of rust functions as VPI functions

#[doc(hidden)]
#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
pub mod vpi_user;

#[doc(hidden)]
pub mod __hidden__;

mod bitvec;

mod impls;

use core::ops::{Deref, DerefMut};

pub use bitvec::BitVector;
pub use vpi_export_macro::{bitvec, vpi_task};
pub use vpi_user::vpi_printf;

/// Mark a value as `input`. Changes on this argument will have no effect on
/// the task. This is equivalent as using the argument `E`
pub struct Input<E>
where
    E: FromVpiHandle,
{
    elem: E,
}

impl<E> Deref for Input<E>
where
    E: FromVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}

impl<E> DerefMut for Input<E>
where
    E: FromVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elem
    }
}

impl<E> FromVpiHandle for Input<E>
where
    E: FromVpiHandle,
{
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self> {
        Ok(Self {
            elem: E::from_vpi_handle(handle)?,
        })
    }
}

/// Mark a value as `output`. Its value will default to [Default::default]. Changes
/// will update argument to the set value at the end of the function call.
pub struct Output<E>
where
    E: Default + IntoVpiHandle,
{
    handle: vpi_user::vpiHandle,
    elem: E,
}

impl<E> Deref for Output<E>
where
    E: Default + IntoVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}

impl<E> DerefMut for Output<E>
where
    E: Default + IntoVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elem
    }
}

impl<E> FromVpiHandle for Output<E>
where
    E: Default + IntoVpiHandle,
{
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self> {
        Ok(Self {
            handle,
            elem: E::default(),
        })
    }
}

impl<E> Drop for Output<E>
where
    E: Default + IntoVpiHandle,
{
    fn drop(&mut self) {
        let e = core::mem::take(&mut self.elem);
        unsafe { IntoVpiHandle::into_vpi_handle(e, self.handle) };
    }
}

/// Mark a value as `inout`. Changes
/// will update argument to the set value at the end of the function call.
pub struct InputOutput<E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    handle: vpi_user::vpiHandle,
    elem: E,
}

impl<E> Deref for InputOutput<E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.elem
    }
}

impl<E> DerefMut for InputOutput<E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elem
    }
}

impl<E> FromVpiHandle for InputOutput<E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self> {
        Ok(Self {
            handle,
            elem: E::from_vpi_handle(handle)?,
        })
    }
}

impl<E> Drop for InputOutput<E>
where
    E: IntoVpiHandle + FromVpiHandle,
{
    fn drop(&mut self) {
        //just to replace
        let replace = unsafe { FromVpiHandle::from_vpi_handle(self.handle) }.unwrap();
        let e = core::mem::replace(&mut self.elem, replace);
        unsafe { IntoVpiHandle::into_vpi_handle(e, self.handle) };
    }
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
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self>;
}

///Conversion trait from rust to verilog
pub trait IntoVpiHandle: Sized {
    ///Conversion function from rust to verilog. In implementation, use the function
    /// [crate::vpi_user::vpi_put_value] to conver type to verilog.
    unsafe fn into_vpi_handle(self, handle: vpi_user::vpiHandle);
}

///Print function that internally will use the simulator's print function.
pub fn print(c: &core::ffi::CStr) {
    unsafe {
        vpi_user::vpi_printf(c.as_ptr() as *const core::ffi::c_char);
    }
}

///Print function that internally will use the simulator's print function with an appended new line.
pub fn println(c: &core::ffi::CStr) {
    print(c);
    print(&unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(b"\n\0") });
}

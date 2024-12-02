#![no_std]
#![feature(associated_type_defaults)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

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

#[derive(Debug)]
pub enum VpiConversionError {
    Utf8Error(core::str::Utf8Error),
    BitVectorMissMatch { expected: usize, actual: usize },
}

pub type Result<T> = core::result::Result<T, VpiConversionError>;

pub trait FromVpiHandle: Sized {
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> Result<Self>;
}

pub trait IntoVpiHandle: Sized {
    unsafe fn into_vpi_handle(self, handle: vpi_user::vpiHandle);
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

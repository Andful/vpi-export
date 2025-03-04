use core::{
    ffi::CStr,
    ops::{Deref, DerefMut},
};

use crate::{FromVpiHandle, RawHandle, StoreIntoVpiHandle};

#[derive(Clone)]
///Vpi handle type to interact with verilog values
pub struct Handle<E>
where
    E: FromVpiHandle,
{
    pub(crate) handle: crate::RawHandle,
    pd: core::marker::PhantomData<E>,
}

impl<E> Handle<E>
where
    E: FromVpiHandle,
{
    /// Immutable borrow of wrapped value
    pub fn borrow(&self) -> crate::Result<HandleRef<'_, E>> {
        Ok(HandleRef(Default::default(), self.get_value()?))
    }

    /// Mutable borrow of wrapped value
    pub fn borrow_mut(&mut self) -> crate::Result<HandleMut<'_, E>>
    where
        E: StoreIntoVpiHandle,
    {
        Ok(HandleMut(self, self.get_value()?))
    }

    /// Obtain underling raw handle
    pub fn raw_handle(&self) -> RawHandle {
        self.handle
    }

    /// Consumes the [Handle], returning the wrapped value.
    pub fn get_value(&self) -> crate::Result<E> {
        unsafe { E::from_vpi_handle(self.handle) }
    }

    /// Get verilog name of handle
    pub fn raw_name(&self) -> *const core::ffi::c_char {
        unsafe { vpi_user::vpi_get_str(vpi_user::vpiName as i32, self.handle.as_ptr()) }
    }

    /// Get verilog name of handle
    pub fn name(&self) -> crate::Result<alloc::string::String> {
        use crate::alloc::string::ToString;

        unsafe { CStr::from_ptr(self.raw_name()) }
            .to_str()
            .map(|e| e.to_string())
            .map_err(crate::VpiError::Utf8Error)
    }
}

pub struct HandleRef<'a, E>(core::marker::PhantomData<&'a ()>, E)
where
    E: FromVpiHandle;

impl<E> Deref for HandleRef<'_, E>
where
    E: FromVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

pub struct HandleMut<'a, E>(&'a mut Handle<E>, E)
where
    E: FromVpiHandle + StoreIntoVpiHandle;

impl<E> Deref for HandleMut<'_, E>
where
    E: FromVpiHandle + StoreIntoVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<E> DerefMut for HandleMut<'_, E>
where
    E: FromVpiHandle + StoreIntoVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<E> Drop for HandleMut<'_, E>
where
    E: FromVpiHandle + StoreIntoVpiHandle,
{
    fn drop(&mut self) {
        unsafe { StoreIntoVpiHandle::store_into_vpi_handle(&self.1, self.0.handle) }.unwrap();
    }
}

impl<E> crate::__private::Sealed for Handle<E> where E: FromVpiHandle {}

impl<E> FromVpiHandle for Handle<E>
where
    E: FromVpiHandle,
{
    unsafe fn from_vpi_handle(handle: RawHandle) -> crate::Result<Self> {
        Ok(Self {
            handle,
            pd: Default::default(),
        })
    }
}

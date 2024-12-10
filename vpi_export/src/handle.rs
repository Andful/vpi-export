use core::ops::{Deref, DerefMut};

use crate::{FromVpiHandle, IntoVpiHandle};

///Handle
pub struct Handle<E>
where
    E: FromVpiHandle,
{
    pub(crate) handle: crate::vpi_user::vpiHandle,
    element: E,
    pd: core::marker::PhantomData<E>,
}

impl<E> Handle<E>
where
    E: FromVpiHandle,
{
    ///Get reference
    pub fn as_ref(&self) -> crate::Result<HandleRef<'_, E>> {
        Ok(HandleRef(self))
    }

    ///Get mutable reference
    pub fn as_mut(&mut self) -> crate::Result<HandleMut<'_, E>>
    where
        E: IntoVpiHandle,
    {
        Ok(HandleMut(self))
    }

    pub fn into_inner(self) -> E {
        self.element
    }
}

pub struct HandleRef<'a, E>(&'a Handle<E>)
where
    E: FromVpiHandle;

impl<'a, E> Deref for HandleRef<'a, E>
where
    E: FromVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.0.element
    }
}

pub struct HandleMut<'a, E>(&'a mut Handle<E>)
where
    E: FromVpiHandle + IntoVpiHandle;

impl<E> Deref for HandleMut<'_, E>
where
    E: FromVpiHandle + IntoVpiHandle,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.0.element
    }
}

impl<E> DerefMut for HandleMut<'_, E>
where
    E: FromVpiHandle + IntoVpiHandle,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.element
    }
}

impl<E> Drop for HandleMut<'_, E>
where
    E: FromVpiHandle + IntoVpiHandle,
{
    fn drop(&mut self) {
        unsafe { IntoVpiHandle::into_vpi_handle(&self.0.element, self.0.handle) }.unwrap();
    }
}

impl<E> crate::__private::Sealed for Handle<E> where E: FromVpiHandle {}

impl<E> FromVpiHandle for Handle<E>
where
    E: FromVpiHandle,
{
    unsafe fn from_vpi_handle(handle: vpi_user::vpiHandle) -> crate::Result<Self> {
        Ok(Self {
            handle,
            element: FromVpiHandle::from_vpi_handle(handle)?,
            pd: Default::default(),
        })
    }
}
